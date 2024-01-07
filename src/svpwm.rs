use fixed::{traits::LossyFrom as _, types::I16F16};

/// Get the zero-indexed sector based on an angle given in radians.
///
/// Supplied angle must be within the range of 0 <= angle < 2*pi, values outside
/// of this range will be clamped.
///
/// The returned value will be between 0 and 5 inclusive.
pub fn sector(angle: I16F16) -> u8 {
    let raw = I16F16::lossy_from(fixed::consts::FRAC_1_TAU)
        .saturating_mul(angle)
        .saturating_mul_int(6);
    let val: u8 = raw.saturating_to_num();
    val.min(5)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sector_calculations() {
        assert_eq!(sector(I16F16::from_num(0f32.to_radians())), 0);
        assert_eq!(sector(I16F16::from_num(10f32.to_radians())), 0);
        assert_eq!(sector(I16F16::from_num(45f32.to_radians())), 0);
        assert_eq!(sector(I16F16::from_num(60.1f32.to_radians())), 1);
        assert_eq!(sector(I16F16::from_num(90f32.to_radians())), 1);
        assert_eq!(sector(I16F16::from_num(110f32.to_radians())), 1);
        assert_eq!(sector(I16F16::from_num(120.1f32.to_radians())), 2);
        assert_eq!(sector(I16F16::from_num(179f32.to_radians())), 2);
        assert_eq!(sector(I16F16::from_num(180.1f32.to_radians())), 3);
        assert_eq!(sector(I16F16::from_num(210f32.to_radians())), 3);
        assert_eq!(sector(I16F16::from_num(235f32.to_radians())), 3);
        assert_eq!(sector(I16F16::from_num(240.1f32.to_radians())), 4);
        assert_eq!(sector(I16F16::from_num(270f32.to_radians())), 4);
        assert_eq!(sector(I16F16::from_num(299f32.to_radians())), 4);
        assert_eq!(sector(I16F16::from_num(300.1f32.to_radians())), 5);
        assert_eq!(sector(I16F16::from_num(340f32.to_radians())), 5);
        assert_eq!(sector(I16F16::from_num(359.9f32.to_radians())), 5);
    }

    #[test]
    fn sector_clamp() {
        assert_eq!(sector(I16F16::from_num(-10f32.to_radians())), 0);
        assert_eq!(sector(I16F16::from_num(-200f32.to_radians())), 0);
        assert_eq!(sector(I16F16::from_num(-600f32.to_radians())), 0);
        assert_eq!(sector(I16F16::from_num(360f32.to_radians())), 5);
        assert_eq!(sector(I16F16::from_num(400f32.to_radians())), 5);
        assert_eq!(sector(I16F16::from_num(1000f32.to_radians())), 5);
    }
}
