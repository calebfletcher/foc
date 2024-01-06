#![no_std]
#![forbid(unsafe_code)]

use nalgebra::ComplexField as _;
use simba::scalar::FixedI16F16;

mod pid;

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
        currents: [FixedI16F16; 2],
        angle: FixedI16F16,
        desired_torque: FixedI16F16,
        dt: FixedI16F16,
    ) -> [FixedI16F16; 3] {
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();
        let frac_1_sqrt_3 = FixedI16F16::from_num(0.57735027f32); // 1/sqrt(3)
        let sqrt_3 = FixedI16F16::from_num(1.7320508f32); // sqrt(3)

        // Clarke transform
        // Eq3
        let i_alpha = currents[0];
        // Eq4
        let i_beta = frac_1_sqrt_3 * (currents[0] + FixedI16F16::from_num(2) * currents[1]);

        // Park transform
        // Eq8
        let i_d = cos_angle * i_alpha + sin_angle * i_beta;
        // Eq9
        let i_q = cos_angle * i_beta - sin_angle * i_alpha;

        // Error to desired torque & flux currents
        let (error_i_d, error_i_q) = (-i_d, desired_torque - i_q);

        // PI controllers
        let v_d = self
            .flux_current_controller
            .update(error_i_d, FixedI16F16::from_num(0), dt);
        let v_q = self
            .torque_current_controller
            .update(error_i_q, desired_torque, dt);

        // Inverse Park transform
        // Eq10
        let v_alpha = cos_angle * v_d - sin_angle * v_q;
        // Eq11
        let v_beta = sin_angle * v_d + cos_angle * v_q;

        // Inverse Clark transform
        // Eq5
        let v_a = v_alpha;
        // Eq6
        let v_b = -(-v_alpha + sqrt_3 * v_beta) / FixedI16F16::from_num(2);
        // Eq7
        let v_c = -(-v_alpha - sqrt_3 * v_beta) / FixedI16F16::from_num(2);

        [v_a, v_b, v_c]
    }
}
