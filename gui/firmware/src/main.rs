#![no_main]
#![no_std]
#![allow(non_snake_case)]

use panic_rtt_target as _;
use rtt_target::{DownChannel, UpChannel};

pub struct RpcChannel {
    up: UpChannel,
    down: DownChannel,
}

#[rtic::app(device = stm32g4xx_hal::stm32, dispatchers = [SPI1])]
mod app {
    use as5048a::AS5048A;
    use cobs::CobsDecoder;
    use fixed::types::I1F31;
    use icd::Endpoint;
    use rtt_target::{rtt_init, ChannelMode};
    use stm32g4xx_hal::{
        adc::{
            config::{Continuous, SampleTime, Sequence},
            Adc, AdcClaim, ClockSource, Configured,
        },
        cordic::{
            self,
            func::dynamic::{Any, Mode as _},
            prec::P20,
            types::Q31,
            Cordic, Ext,
        },
        delay::SYSTDelayExt as _,
        gpio::{
            gpioc::{PC10, PC11, PC12, PC15, PC9},
            Alternate, GpioExt as _, Output, PushPull, AF6,
        },
        prelude::OutputPin as _,
        pwr::PwrExt as _,
        rcc::{Config, PllMDiv, PllNMul, PllRDiv, PllSrc, RccExt as _},
        spi::{Spi, SpiExt as _},
        stm32::{ADC1, SPI3},
        time::{ExtU32 as _, RateExtU32 as _},
        timer::{CountDownTimer, Event, Timer},
    };

    use crate::RpcChannel;

    type DynamicCordic = Cordic<Q31, Q31, Any, P20>;
    type EncoderSpi = Spi<
        SPI3,
        (
            PC10<Alternate<AF6>>,
            PC11<Alternate<AF6>>,
            PC12<Alternate<AF6>>,
        ),
    >;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        rpc_channel: RpcChannel,
        led: PC15<Output<PushPull>>,
        timer_handler: CountDownTimer<stm32g4xx_hal::stm32::TIM2>,
        cordic: DynamicCordic,
        encoder: AS5048A<EncoderSpi, PC9<Output<PushPull>>>,
        adc: Option<Adc<ADC1, Configured>>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let channels = rtt_init! {
            up: {
                0: {
                    size: 1024,
                    mode: ChannelMode::BlockIfFull,
                    name: "Terminal"
                }
            }
            down: {
                0: {
                    size: 1024,
                    mode: ChannelMode::BlockIfFull,
                    name: "Terminal"
                }
            }
        };

        let rpc_channel = RpcChannel {
            up: channels.up.0,
            down: channels.down.0,
        };

        let pwr = cx.device.PWR.constrain().freeze();
        let mut rcc = cx.device.RCC.freeze(
            Config::pll().pll_cfg(stm32g4xx_hal::rcc::PllConfig {
                mux: PllSrc::HSI,
                m: PllMDiv::DIV_1,
                n: PllNMul::MUL_18,
                r: Some(PllRDiv::DIV_2),
                q: None,
                p: None,
            }),
            pwr,
        );

        let cordic = cx.device.CORDIC.constrain(&mut rcc).into_dynamic();

        let gpioa = cx.device.GPIOA.split(&mut rcc);
        let gpioc = cx.device.GPIOC.split(&mut rcc);

        let cs0 = gpioc.pc9.into_push_pull_output();
        //let cs1 = gpioc.pc8.into_push_pull_output();
        let sclk = gpioc.pc10.into_alternate::<AF6>();
        let miso = gpioc.pc11.into_alternate::<AF6>();
        let mosi = gpioc.pc12.into_alternate::<AF6>();

        let spi = cx.device.SPI3.spi(
            (sclk, miso, mosi),
            stm32g4xx_hal::spi::Mode {
                polarity: stm32g4xx_hal::spi::Polarity::IdleHigh,
                phase: stm32g4xx_hal::spi::Phase::CaptureOnFirstTransition,
            },
            1u32.MHz(),
            &mut rcc,
        );

        let encoder = AS5048A::new(spi, cs0);

        // These don't implement drop, so it should be fine to drop them but
        // still use the ADC
        let motor_current_a = gpioa.pa0.into_analog();
        let motor_current_b = gpioa.pa1.into_analog();

        let mut delay = cx.core.SYST.delay(&rcc.clocks);
        let mut adc = cx
            .device
            .ADC1
            .claim(ClockSource::SystemClock, &rcc, &mut delay, false);

        adc.set_continuous(Continuous::Single);
        adc.reset_sequence();
        adc.configure_channel(&motor_current_a, Sequence::One, SampleTime::Cycles_640_5);
        adc.configure_channel(&motor_current_b, Sequence::Two, SampleTime::Cycles_640_5);
        let adc = adc.enable();

        let led = gpioc.pc15.into_push_pull_output();

        let timer2 = Timer::new(cx.device.TIM2, &rcc.clocks);
        let mut timer2 = timer2.start_count_down(1000u32.millis());
        timer2.clear_interrupt(Event::TimeOut);
        timer2.listen(Event::TimeOut);

        rtt_rpc::spawn().unwrap();

        (
            Shared {},
            Local {
                rpc_channel,
                led,
                timer_handler: timer2,
                cordic,
                encoder,
                adc: Some(adc),
            },
        )
    }

    #[task(binds = TIM2, priority = 1, local = [led, timer_handler, led_state: bool = false])]
    fn tick(cx: tick::Context) {
        if *cx.local.led_state {
            let _ = cx.local.led.set_high();
            *cx.local.led_state = false;
        } else {
            let _ = cx.local.led.set_low();
            *cx.local.led_state = true;
        }
        cx.local.timer_handler.clear_interrupt(Event::TimeOut);
    }

    #[task(local = [rpc_channel, cordic, encoder, adc])]
    async fn rtt_rpc(cx: rtt_rpc::Context) {
        let mut rx_buffer_raw = [0; 64];
        let mut rx_buffer_frame = [0; 64];
        let mut tx_buffer = [0; 64];
        let mut decoder = CobsDecoder::new(&mut rx_buffer_frame);

        #[derive(serde::Serialize, serde::Deserialize)]
        struct Frame<T> {
            endpoint: u8,
            msg: T,
        }

        let ping = |_: ()| {};
        let mut stored_value = 0;

        let mut handle_frame = |frame: &[u8], tx_buffer: &mut [u8]| -> Result<usize, ()> {
            if frame.is_empty() {
                return Err(());
            }

            icd::generate_endpoint_handler! {
                frame, tx_buffer,
                (icd::PingEndpoint, ping)
                (icd::ReadEndpoint, |_: ()| { stored_value })
                (icd::WriteEndpoint, |value: u32| { stored_value = value; })
                (icd::SinCosEndpoint, |value: f32| {
                    let (sin, cos) = cx.local.cordic.run::<cordic::func::SinCos>(I1F31::from_num(value));
                    (sin.to_num::<f32>(), cos.to_num::<f32>())
                })
                (icd::EncoderAngle, |_: ()| {
                    cx.local.encoder.angle().unwrap()
                })
                (icd::MotorPhaseCurrents, |_: ()| {
                    let adc = cx.local.adc.take().unwrap().start_conversion();

                    let adc = adc.wait_for_conversion_sequence().unwrap_active();
                    let current_a_mv = adc.sample_to_millivolts(adc.current_sample());
                    let adc = adc.wait_for_conversion_sequence().unwrap_stopped();
                    let current_b_mv = adc.sample_to_millivolts(adc.current_sample());

                    *cx.local.adc = Some(adc);

                    [current_a_mv, current_b_mv]
                })
            }
        };

        loop {
            // Read from channel to COBS decoder
            let bytes_read = cx.local.rpc_channel.down.read(&mut rx_buffer_raw);
            for byte in &rx_buffer_raw[..bytes_read] {
                match decoder.feed(*byte) {
                    Ok(Some(decoded_size)) => {
                        // Handle received frame
                        let frame = &rx_buffer_frame[..decoded_size];
                        let encoded_resp_len = handle_frame(frame, &mut tx_buffer).unwrap();
                        let encoded_resp = &tx_buffer[..encoded_resp_len];

                        // Send a response
                        let mut bytes_written = 0;
                        while bytes_written < encoded_resp.len() {
                            bytes_written += cx
                                .local
                                .rpc_channel
                                .up
                                .write(&encoded_resp[bytes_written..]);
                        }

                        decoder = CobsDecoder::new(&mut rx_buffer_frame);
                    }
                    Ok(None) => {
                        // not a full packet
                        continue;
                    }
                    Err(_count) => {
                        // decode error
                        continue;
                    }
                }
            }
        }
    }
}
