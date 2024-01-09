//! Algorithms to convert a value from a two-phase stationary orthogonal
//! reference frame to a value suitable to be used for PWM generation.

use crate::park_clarke::TwoPhaseStationaryOrthogonalReferenceFrame;

/// Returns the three PWM values in the range 0-255, one for each motor coil.
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
    let t = 1.;
    let (ta, tb, tc);
    match sector {
        1 | 4 => {
            ta = t + x - z;
            tb = t + x + z;
            tc = t - x + z;
        }
        2 | 5 => {
            ta = t + y - z;
            tb = t + y + z;
            tc = t - y - z;
        }
        3 | 6 => {
            ta = t + y - x;
            tb = t - y + x;
            tc = t - y - x;
        }
        _ => unreachable!("invalid sector"),
    }

    [ta / 2., tb / 2., tc / 2.]
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
