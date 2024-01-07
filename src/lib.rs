#![no_std]
#![forbid(unsafe_code)]

use fixed::types::I16F16;
use fixed_macro::types::I16F16;

mod park_clarke;
mod pid;

const FRAC_1_SQRT_3: I16F16 = I16F16!(0.57735027);
const SQRT_3: I16F16 = I16F16!(1.7320508);

pub struct Foc {
    flux_current_controller: pid::PIController,
    torque_current_controller: pid::PIController,
}

impl Foc {
    /// Current in amps
    /// Angle in radians
    /// Returns the 3 PWM values
    pub fn update(
        &mut self,
        currents: [I16F16; 2],
        angle: I16F16,
        desired_torque: I16F16,
        dt: I16F16,
    ) -> [I16F16; 3] {
        let (sin_angle, cos_angle) = cordic::sin_cos(angle);

        // Clarke transform
        let orthogonal_current =
            park_clarke::clarke(park_clarke::ThreePhaseStationaryReferenceFrame {
                a: currents[0],
                b: currents[1],
                c: None,
            });

        // Park transform
        let rotating_current = park_clarke::park(cos_angle, sin_angle, orthogonal_current);

        // Current PI controllers
        let v_d = self
            .flux_current_controller
            .update(rotating_current.d, I16F16::ZERO, dt);
        let v_q = self
            .torque_current_controller
            .update(rotating_current.q, desired_torque, dt);

        // Inverse Park transform
        let orthogonal_voltage = park_clarke::inverse_park(
            cos_angle,
            sin_angle,
            park_clarke::MovingReferenceFrame { d: v_d, q: v_q },
        );

        // Inverse Clark transform
        let voltages = park_clarke::inverse_clarke(orthogonal_voltage);

        [
            voltages.a,
            voltages.b,
            voltages.c.expect("inverse clarke filled out third value"),
        ]
    }
}
