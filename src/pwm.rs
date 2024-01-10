//! Algorithms to convert a value from a two-phase stationary orthogonal
//! reference frame to a value suitable to be used for PWM generation.
//!
//! The resulting waveforms of the PWM generation methods are shown below.
//! ![PWM Methods](https://raw.githubusercontent.com/calebfletcher/foc/main/docs/pwm_methods.png)

use fixed::types::I16F16;

use crate::park_clarke::TwoPhaseReferenceFrame;

pub trait Modulation {
    fn modulate(value: TwoPhaseReferenceFrame) -> [I16F16; 3];

    /// Module the value, returning the result as a value between 0 and the specified
    /// maximum value inclusive.
    fn as_compare_value<const MAX: u16>(value: TwoPhaseReferenceFrame) -> [u16; 3] {
        Self::modulate(value).map(|val| {
            (((val + I16F16::from_num(1)) * (MAX as i32 + 1)) / 2)
                .round()
                .saturating_to_num::<u16>()
                .clamp(0, MAX)
        })
    }
}

/// Generate PWM values based on a space-vector method.
///
/// This method results in a waveform that is more efficient than sinusoidal
/// PWM while having better current ripple than the other methods. However, it
/// comes at the expense of a more complex computation.
///
/// Returns a value between -1 and 1 for each channel.
pub struct SpaceVector;

impl Modulation for SpaceVector {
    fn modulate(value: TwoPhaseReferenceFrame) -> [I16F16; 3] {
        // Convert alpha/beta to x/y/z
        let sqrt_3_alpha = I16F16::SQRT_3 * value.alpha;
        let beta = value.beta;
        let x = beta;
        let y = (beta + sqrt_3_alpha) / 2;
        let z = (beta - sqrt_3_alpha) / 2;

        // Calculate which sector the value falls in
        let sector: u8 = match (x.is_positive(), y.is_positive(), z.is_positive()) {
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
}

/// Generate PWM values based on a sinusoidal waveform.
///
/// While this method is very simple (and fast) it is less efficient than SVPWM
/// as it does not utilise the bus voltage as well.
///
/// Returns a value between -1 and 1 for each channel.
pub struct Sinusoidal;

impl Modulation for Sinusoidal {
    fn modulate(value: TwoPhaseReferenceFrame) -> [I16F16; 3] {
        let voltages = crate::park_clarke::inverse_clarke(value);

        [voltages.a, voltages.b, voltages.c]
    }
}

/// Generate PWM values based on a trapezoidal wave.
///
/// Note that for this method to work properly, when the output is 0 the
/// resective channel should be disabled/set as high impedance.
///
/// Returns a value between -1 and 1 for each channel.
pub struct Trapezoidal;

impl Modulation for Trapezoidal {
    fn modulate(value: TwoPhaseReferenceFrame) -> [I16F16; 3] {
        let voltages = crate::park_clarke::inverse_clarke(value);

        [
            (voltages.a * 2).round_to_zero().signum(),
            (voltages.b * 2).round_to_zero().signum(),
            (voltages.c * 2).round_to_zero().signum(),
        ]
    }
}

/// Generate PWM values based on a square wave.
///
/// Returns a value between -1 and 1 for each channel.
pub struct Square;

impl Modulation for Square {
    fn modulate(value: TwoPhaseReferenceFrame) -> [I16F16; 3] {
        let voltages = crate::park_clarke::inverse_clarke(value);

        [
            voltages.a.signum(),
            voltages.b.signum(),
            voltages.c.signum(),
        ]
    }
}
