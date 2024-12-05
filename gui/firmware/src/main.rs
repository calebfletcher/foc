#![no_main]
#![no_std]
#![allow(non_snake_case)]

use panic_rtt_target as _;
use rtt_target::{DownChannel, UpChannel};

pub struct RpcChannel {
    up: UpChannel,
    down: DownChannel,
}

#[rtic::app(device = stm32f1xx_hal::pac, dispatchers = [SPI1])]
mod app {
    use cobs::CobsDecoder;
    use icd::Endpoint;
    use rtt_target::{rtt_init, ChannelMode};
    use stm32f1xx_hal::{
        gpio::{Output, PinState, PushPull, PC13},
        pac,
        prelude::*,
        timer::{CounterMs, Event},
    };

    use crate::RpcChannel;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        rpc_channel: RpcChannel,
        led: PC13<Output<PushPull>>,
        timer_handler: CounterMs<pac::TIM1>,
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

        let mut flash = cx.device.FLASH.constrain();
        let rcc = cx.device.RCC.constrain();

        // let clocks = rcc
        //     .cfgr
        //     .use_hse(8.MHz())
        //     .sysclk(48.MHz())
        //     .pclk1(24.MHz())
        //     .freeze(&mut flash.acr);

        let clocks = rcc.cfgr.freeze(&mut flash.acr);
        let mut gpioc = cx.device.GPIOC.split();
        let led = gpioc
            .pc13
            .into_push_pull_output_with_state(&mut gpioc.crh, PinState::High);

        // Configure the syst timer to trigger an update every second and enables interrupt
        let mut timer = cx.device.TIM1.counter_ms(&clocks);
        timer.start(1.secs()).unwrap();
        timer.listen(Event::Update);

        rtt_rpc::spawn().unwrap();

        (
            Shared {},
            Local {
                rpc_channel,
                led,
                timer_handler: timer,
            },
        )
    }

    #[task(binds = TIM1_UP, priority = 1, local = [led, timer_handler, led_state: bool = false])]
    fn tick(cx: tick::Context) {
        if *cx.local.led_state {
            cx.local.led.set_high();
            *cx.local.led_state = false;
        } else {
            cx.local.led.set_low();
            *cx.local.led_state = true;
        }
        cx.local.timer_handler.clear_interrupt(Event::Update);
    }

    #[task(local = [rpc_channel])]
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
