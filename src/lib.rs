#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

use core::marker::PhantomData;

use fixed::types::I16F16;

pub mod park_clarke;
mod pid;
pub mod pwm;

const FRAC_1_SQRT_3: I16F16 = I16F16::lit("0.57735027");
const SQRT_3: I16F16 = I16F16::lit("1.7320508");

pub struct Foc<Modulator: pwm::Modulation, const PWM_RESOLUTION: u16> {
    flux_current_controller: pid::PIController,
    torque_current_controller: pid::PIController,
    _phantom: PhantomData<Modulator>,
}

impl<Modulator: pwm::Modulation, const PWM_RESOLUTION: u16> Foc<Modulator, PWM_RESOLUTION> {
    pub fn new(
        flux_current_controller: pid::PIController,
        torque_current_controller: pid::PIController,
    ) -> Self {
        Self {
            flux_current_controller,
            torque_current_controller,
            _phantom: PhantomData,
        }
    }

    /// Current in amps
    /// Angle in radians
    /// Returns the 3 PWM values
    pub fn update(
        &mut self,
        currents: [I16F16; 2],
        angle: I16F16,
        desired_torque: I16F16,
        dt: I16F16,
    ) -> [u16; 3] {
        let (sin_angle, cos_angle) = cordic::sin_cos(angle);

        // Clarke transform
        let orthogonal_current =
            park_clarke::clarke(park_clarke::ThreePhaseBalancedStationaryReferenceFrame {
                a: currents[0],
                b: currents[1],
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

        // Modulate the result to PWM values
        Modulator::as_compare_value::<PWM_RESOLUTION>(orthogonal_voltage)
    }
}
