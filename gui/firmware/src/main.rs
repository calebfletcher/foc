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
    use cobs::CobsDecoder;
    use fixed::types::I1F31;
    use icd::Endpoint;
    use rtt_target::{rtt_init, ChannelMode};
    use stm32g4xx_hal::{
        cordic::{
            self,
            func::dynamic::{Any, Mode as _},
            prec::P20,
            types::Q31,
            Cordic, Ext,
        },
        gpio::{gpioc::PC15, Output, PushPull},
        prelude::*,
        pwr::PwrExt as _,
        rcc::Config,
        time::ExtU32 as _,
        timer::{CountDownTimer, Event, Timer},
    };

    use crate::RpcChannel;

    type DynamicCordic = Cordic<Q31, Q31, Any, P20>;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        rpc_channel: RpcChannel,
        led: PC15<Output<PushPull>>,
        timer_handler: CountDownTimer<stm32g4xx_hal::stm32::TIM2>,
        cordic: DynamicCordic,
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
        let mut rcc = cx.device.RCC.freeze(Config::hsi(), pwr);

        let cordic = cx.device.CORDIC.constrain(&mut rcc).into_dynamic();

        let gpioc = cx.device.GPIOC.split(&mut rcc);
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

    #[task(local = [rpc_channel, cordic])]
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
                (icd::ReadEndpoint, |_: ()| -> u32 { stored_value })
                (icd::WriteEndpoint, |value: u32| { stored_value = value; })
                (icd::SinCosEndpoint, |value: f32| {
                    let (sin, cos) = cx.local.cordic.run::<cordic::func::SinCos>(I1F31::from_num(value));
                    (sin.to_num::<f32>(), cos.to_num::<f32>())
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
