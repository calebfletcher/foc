use std::{collections::BTreeMap, f32::consts::TAU, fs::File, io::BufWriter, sync::Arc};

use fixed::types::I16F16;
use foc::{park_clarke, pwm::Modulation};
use serde::Serialize;

#[derive(Serialize)]
struct Values {
    time_ns: u64,
    angle_rad: f32,
    orthogonal_voltage_alpha: f32,
    orthogonal_voltage_beta: f32,
    orthogonal_atan: f32,
    svpwm: [f32; 3],
    spwm: [f32; 3],
    trapezoidal: [f32; 3],
    square: [f32; 3],
}

fn main() -> Result<(), anyhow::Error> {
    let mut writer = mcap::Writer::new(BufWriter::new(File::create("out.mcap")?))?;
    let my_channel = mcap::Channel {
        topic: String::from("foc"),
        schema: Some(Arc::new(mcap::Schema {
            name: "".to_owned(),
            encoding: "".to_owned(),
            data: std::borrow::Cow::default(),
        })),
        message_encoding: "cbor".to_owned(),
        metadata: BTreeMap::default(),
    };
    let channel_id = writer.add_channel(&my_channel)?;

    let mut time_ns = 0;
    let dt_ns = 1_000_000;
    let mut angle_rad: f32 = 0.;
    let velocity_rad_per_sec = 1.;

    while time_ns <= 10_000_000_000 {
        // Calc motor values
        let angle_rad_norm = angle_rad % std::f32::consts::TAU;

        let (sin_angle, cos_angle) = cordic::sin_cos(I16F16::from_num(angle_rad_norm));
        let orthogonal_voltage = park_clarke::inverse_park(
            cos_angle,
            sin_angle,
            park_clarke::RotatingReferenceFrame {
                d: I16F16::ZERO,
                q: I16F16::ONE,
            },
        );

        let svpwm = foc::pwm::SpaceVector::modulate(orthogonal_voltage.clone());
        let spwm = foc::pwm::Sinusoidal::modulate(orthogonal_voltage.clone());
        let trapezoidal = foc::pwm::Trapezoidal::modulate(orthogonal_voltage.clone());
        let square = foc::pwm::Square::modulate(orthogonal_voltage.clone());

        let orthogonal_atan = orthogonal_voltage
            .beta
            .to_num::<f32>()
            .atan2(orthogonal_voltage.alpha.to_num::<f32>());

        // Write to file
        let mut buffer = Vec::with_capacity(128);
        ciborium::into_writer(
            &Values {
                time_ns,
                angle_rad: angle_rad_norm,
                orthogonal_voltage_alpha: orthogonal_voltage.alpha.to_num(),
                orthogonal_voltage_beta: orthogonal_voltage.beta.to_num(),
                orthogonal_atan: orthogonal_atan.rem_euclid(TAU),
                svpwm: svpwm.map(|v| v.to_num()),
                spwm: spwm.map(|v| v.to_num()),
                trapezoidal: trapezoidal.map(|v| v.to_num()),
                square: square.map(|v| v.to_num()),
            },
            &mut buffer,
        )
        .unwrap();
        writer
            .write_to_known_channel(
                &mcap::records::MessageHeader {
                    channel_id,
                    sequence: 0,
                    log_time: time_ns,
                    publish_time: time_ns,
                },
                &buffer,
            )
            .unwrap();

        // Update state
        angle_rad += velocity_rad_per_sec * (dt_ns as f32 / 1e9);
        time_ns += dt_ns;
    }

    writer.finish().unwrap();

    Ok(())
}
