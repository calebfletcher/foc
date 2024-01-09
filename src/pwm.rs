//! Algorithms to convert a value from a two-phase stationary orthogonal
//! reference frame to a value suitable to be used for PWM generation.
//!
//! The resulting waveforms of the PWM generation methods are shown below.
//! ![PWM Methods](https://raw.githubusercontent.com/calebfletcher/foc/main/docs/pwm_methods.png)

use crate::park_clarke::TwoPhaseStationaryOrthogonalReferenceFrame;

/// Generate PWM values based on a space-vector method.
///
/// This method results in a waveform that is more efficient than sinusoidal
/// PWM while having better current ripple than the other methods. However, it
/// comes at the expense of a more complex computation.
pub fn svpwm(value: TwoPhaseStationaryOrthogonalReferenceFrame) -> [f32; 3] {
    // Convert alpha/beta to x/y/z
    const SQRT_3: f32 = 1.7320508;
    let sqrt_3_alpha = SQRT_3 * value.alpha.to_num::<f32>();
    let beta = value.beta.to_num::<f32>();
    let x = beta;
    let y = (beta + sqrt_3_alpha) / 2.;
    let z = (beta - sqrt_3_alpha) / 2.;

    // Calculate which sector the value falls in
    let sector: u8 = match (
        x.is_sign_positive(),
        y.is_sign_positive(),
        z.is_sign_positive(),
    ) {
        (true, true, false) => 1,
        (_, true, true) => 2,
        (true, false, true) => 3,
        (false, false, true) => 4,
        (_, false, false) => 5,
        (false, true, false) => 6,
    };

    // Map a,b,c values to three phase
    let (ta, tb, tc);
    match sector {
        1 | 4 => {
            ta = x - z;
            tb = x + z;
            tc = -x + z;
        }
        2 | 5 => {
            ta = y - z;
            tb = y + z;
            tc = -y - z;
        }
        3 | 6 => {
            ta = y - x;
            tb = -y + x;
            tc = -y - x;
        }
        _ => unreachable!("invalid sector"),
    }

    [ta, tb, tc]
}

/// Generate PWM values based on a sinusoidal waveform.
///
/// While this method is very simple (and fast) it is less efficient than SVPWM
/// as it does not utilise the bus voltage as well.
pub fn spwm(value: TwoPhaseStationaryOrthogonalReferenceFrame) -> [f32; 3] {
    let voltages = crate::park_clarke::inverse_clarke(value);

    [
        voltages.a.to_num::<f32>(),
        voltages.b.to_num::<f32>(),
        voltages.c.unwrap().to_num::<f32>(),
    ]
}

/// Generate PWM values based on a trapezoidal wave.
///
/// Note that for this method to work properly, when the output is 0 the
/// resective channel should be disabled/set as high impedance.
pub fn trapezoidal(value: TwoPhaseStationaryOrthogonalReferenceFrame) -> [f32; 3] {
    let voltages = crate::park_clarke::inverse_clarke(value);

    [
        (voltages.a * 2).round_to_zero().signum().to_num::<f32>(),
        (voltages.b * 2).round_to_zero().signum().to_num::<f32>(),
        (voltages.c.unwrap() * 2)
            .round_to_zero()
            .signum()
            .to_num::<f32>(),
    ]
}

/// Generate PWM values based on a square wave.
pub fn square(value: TwoPhaseStationaryOrthogonalReferenceFrame) -> [f32; 3] {
    let voltages = crate::park_clarke::inverse_clarke(value);

    [
        voltages.a.signum().to_num::<f32>(),
        voltages.b.signum().to_num::<f32>(),
        voltages.c.unwrap().signum().to_num::<f32>(),
    ]
}
