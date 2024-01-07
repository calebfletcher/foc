//! Park and Clarke transformations (along with their inverses).
//!
//! The algorithms implemented here are based on [Microsemi's suggested implementation](https://www.microsemi.com/document-portal/doc_view/132799-park-inverse-park-and-clarke-inverse-clarke-transformations-mss-software-implementation-user-guide)

use crate::{FRAC_1_SQRT_3, SQRT_3};

use fixed::types::I16F16;

#[derive(Debug, Clone)]
pub struct MovingReferenceFrame {
    pub d: I16F16,
    pub q: I16F16,
}

#[derive(Debug, Clone)]
pub struct TwoPhaseStationaryOrthogonalReferenceFrame {
    pub alpha: I16F16,
    pub beta: I16F16,
}

#[derive(Debug, Clone)]
pub struct ThreePhaseStationaryReferenceFrame {
    pub a: I16F16,
    pub b: I16F16,
    /// C is optional if a + b + c equals zero.
    pub c: Option<I16F16>,
}

/// Clarke transform
///
/// Implements equations 1-4 from the Microsemi guide.
pub fn clarke(
    inputs: ThreePhaseStationaryReferenceFrame,
) -> TwoPhaseStationaryOrthogonalReferenceFrame {
    if let Some(c) = inputs.c {
        let _ = TwoPhaseStationaryOrthogonalReferenceFrame {
            // Eq1
            alpha: (2 * inputs.a) / 3 - (inputs.b - c) / 3,
            // Eq2
            beta: 2 * FRAC_1_SQRT_3 * (inputs.b - c),
        };
        unimplemented!("this isn't giving correct results for some reason");
    } else {
        TwoPhaseStationaryOrthogonalReferenceFrame {
            // Eq3
            alpha: inputs.a,
            // Eq4
            beta: FRAC_1_SQRT_3 * (inputs.a + 2 * inputs.b),
        }
    }
}

/// Inverse Clarke transform
///
/// Implements equations 5-7 from the Microsemi guide.
pub fn inverse_clarke(
    inputs: TwoPhaseStationaryOrthogonalReferenceFrame,
) -> ThreePhaseStationaryReferenceFrame {
    ThreePhaseStationaryReferenceFrame {
        // Eq5
        a: inputs.alpha,
        // Eq6
        b: (-inputs.alpha + SQRT_3 * inputs.beta) / 2,
        // Eq7
        c: Some((-inputs.alpha - SQRT_3 * inputs.beta) / 2),
    }
}

/// Park transform
///
/// Implements equations 8 and 9 from the Microsemi guide.
pub fn park(
    cos_angle: I16F16,
    sin_angle: I16F16,
    inputs: TwoPhaseStationaryOrthogonalReferenceFrame,
) -> MovingReferenceFrame {
    MovingReferenceFrame {
        // Eq8
        d: cos_angle * inputs.alpha + sin_angle * inputs.beta,
        // Eq9
        q: cos_angle * inputs.beta - sin_angle * inputs.alpha,
    }
}

/// Inverse Park transform
///
/// Implements equations 10 and 11 from the Microsemi guide.
pub fn inverse_park(
    cos_angle: I16F16,
    sin_angle: I16F16,
    inputs: MovingReferenceFrame,
) -> TwoPhaseStationaryOrthogonalReferenceFrame {
    TwoPhaseStationaryOrthogonalReferenceFrame {
        // Eq10
        alpha: cos_angle * inputs.d - sin_angle * inputs.q,
        // Eq11
        beta: sin_angle * inputs.d + cos_angle * inputs.q,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fixed_macro::types::I16F16;

    #[track_caller]
    fn clark_e_round_trip(a: f32, b: f32, c: Option<f32>) {
        let input = ThreePhaseStationaryReferenceFrame {
            a: I16F16::from_num(a),
            b: I16F16::from_num(b),
            c: c.map(I16F16::from_num),
        };
        let two_phase = clarke(input.clone());
        dbg!(&two_phase);
        let result = inverse_clarke(two_phase);

        dbg!(&result);

        assert!(result.a.abs_diff(input.a) < 0.0001);
        assert!(result.b.abs_diff(input.b) < 0.0001);
        if let Some(c) = input.c {
            assert!(result.c.unwrap().abs_diff(c) < 0.0001);
        } else {
            assert!((result.a + result.b + result.c.unwrap()) < 0.0001);
        }
    }

    #[test]
    fn clarke_round_trip_zero() {
        clark_e_round_trip(0., 0., Some(0.));
        clark_e_round_trip(0., 0., None);
    }

    #[test]
    fn clarke_round_trip_two_inputs() {
        clark_e_round_trip(0., 1., None);
        clark_e_round_trip(1., 0., None);
        clark_e_round_trip(-0.5, -0.5, None);
        clark_e_round_trip(-0.1, -0.2, None);
        clark_e_round_trip(13., 21., None);

        // let i_alpha = (2. / 3.) * i_a -
    }

    #[test]
    fn clarke_round_trip_three_inputs() {
        clark_e_round_trip(0., 1., Some(-1.));
        clark_e_round_trip(1., 0., Some(-1.));
        clark_e_round_trip(-0.5, -0.5, Some(1.));
        clark_e_round_trip(-0.1, -0.2, Some(0.3));
        clark_e_round_trip(13., 21., Some(-34.));
    }

    #[test]
    fn park_round_trip() {
        let angle = I16F16!(0.82);
        let (sin_angle, cos_angle) = cordic::sin_cos(angle);

        let input = TwoPhaseStationaryOrthogonalReferenceFrame {
            alpha: I16F16!(2),
            beta: I16F16!(3),
        };
        let moving_reference = park(cos_angle, sin_angle, input.clone());
        dbg!(&moving_reference);
        let result = inverse_park(cos_angle, sin_angle, moving_reference);

        dbg!(&result);

        assert!(result.alpha.abs_diff(input.alpha) < 0.001);
        assert!(result.beta.abs_diff(input.beta) < 0.001);
    }
}
