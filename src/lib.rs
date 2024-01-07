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
        let cos_angle = cordic::cos(angle);
        let sin_angle = cordic::sin(angle);

        // Clarke transform
        let orthogonal_current =
            park_clarke::clarke(park_clarke::ThreePhaseStationaryReferenceFrame {
                a: currents[0],
                b: currents[1],
                c: None,
            });

        // Park transform
        // Eq8
        let i_d = cos_angle * orthogonal_current.alpha + sin_angle * orthogonal_current.beta;
        // Eq9
        let i_q = cos_angle * orthogonal_current.beta - sin_angle * orthogonal_current.alpha;

        // Error to desired torque & flux currents
        let (error_i_d, error_i_q) = (-i_d, desired_torque - i_q);

        // PI controllers
        let v_d = self
            .flux_current_controller
            .update(error_i_d, I16F16::ZERO, dt);
        let v_q = self
            .torque_current_controller
            .update(error_i_q, desired_torque, dt);

        // Inverse Park transform
        // Eq10
        let v_alpha = cos_angle * v_d - sin_angle * v_q;
        // Eq11
        let v_beta = sin_angle * v_d + cos_angle * v_q;

        // Inverse Clark transform
        let voltages =
            park_clarke::inverse_clarke(park_clarke::TwoPhaseStationaryOrthogonalReferenceFrame {
                alpha: v_alpha,
                beta: v_beta,
            });

        [
            voltages.a,
            voltages.b,
            voltages.c.expect("inverse clarke filled out third value"),
        ]
    }
}
