use anyhow::bail;
use cobs::CobsDecoder;
use probe_rs::{
    probe::{list::Lister, DebugProbeInfo},
    rtt::{DownChannel, Rtt, UpChannel},
    Permissions, Session,
};
use serde::{Deserialize, Serialize};

pub fn list_all() -> Vec<DebugProbeInfo> {
    let lister = Lister::new();
    lister.list_all()
}

pub struct Device {
    pub probe_info: DebugProbeInfo,
    rpc_channel: RpcChannel,
}

impl Device {
    pub fn from_probe_info(probe_info: DebugProbeInfo) -> Result<Self, anyhow::Error> {
        let probe = probe_info.open()?;
        let session = probe.attach("STM32G474VETx", Permissions::default())?;
        Self::from_session(session, probe_info)
    }

    pub fn from_session(
        mut session: Session,
        probe_info: DebugProbeInfo,
    ) -> Result<Self, anyhow::Error> {
        let mut rtt = Rtt::attach(&mut session.core(0)?)?;

        let rpc_channel = RpcChannel {
            session,
            down_channel: rtt.down_channels.swap_remove(0),
            up_channel: rtt.up_channels.swap_remove(0),
        };
        Ok(Self {
            rpc_channel,
            probe_info,
        })
    }

    pub fn ping(&mut self) -> Result<(), anyhow::Error> {
        self.rpc_channel.call::<icd::PingEndpoint>(())
    }

    pub fn read_value(&mut self) -> Result<u32, anyhow::Error> {
        self.rpc_channel.call::<icd::ReadEndpoint>(())
    }

    pub fn write_value(&mut self, value: u32) -> Result<(), anyhow::Error> {
        self.rpc_channel.call::<icd::WriteEndpoint>(value)
    }

    pub fn sin_cos(&mut self, value: f32) -> Result<(f32, f32), anyhow::Error> {
        self.rpc_channel.call::<icd::SinCosEndpoint>(value)
    }

    pub fn encoder_angle(&mut self) -> Result<u16, anyhow::Error> {
        self.rpc_channel.call::<icd::EncoderAngle>(())
    }

    pub fn motor_currents(&mut self) -> Result<[u16; 2], anyhow::Error> {
        self.rpc_channel.call::<icd::MotorPhaseCurrents>(())
    }
}

struct RpcChannel {
    session: Session,
    down_channel: DownChannel,
    up_channel: UpChannel,
}

impl RpcChannel {
    fn call<E: icd::Endpoint>(&mut self, req: E::Request) -> Result<E::Response, anyhow::Error> {
        let mut core = self.session.core(0)?;
        self.down_channel.write(
            &mut core,
            &postcard::to_stdvec_cobs(&Frame {
                endpoint: E::ID,
                msg: req,
            })?,
        )?;
        loop {
            let mut rx_buffer_raw = [0; 64];
            let mut rx_buffer_frame = [0; 64];
            let mut decoder = CobsDecoder::new(&mut rx_buffer_frame);

            let bytes_read = self.up_channel.read(&mut core, &mut rx_buffer_raw)?;
            for byte in &rx_buffer_raw[..bytes_read] {
                match decoder.feed(*byte) {
                    Ok(Some(decoded_size)) => {
                        // Handle received frame
                        let frame_bytes = &rx_buffer_frame[..decoded_size];
                        let frame = postcard::from_bytes::<Frame<E::Response>>(frame_bytes)?;
                        if frame.endpoint != E::ID {
                            bail!("incorrect response endpoint");
                        }
                        return Ok(frame.msg);
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

#[derive(Serialize, Deserialize)]
struct Frame<T> {
    endpoint: u8,
    msg: T,
}
