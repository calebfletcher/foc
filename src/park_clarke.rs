//! Park and Clarke transformations (along with their inverses).
//!
//! The algorithms implemented here are based on [Microsemi's suggested implementation](https://www.microsemi.com/document-portal/doc_view/132799-park-inverse-park-and-clarke-inverse-clarke-transformations-mss-software-implementation-user-guide)

use crate::{FRAC_1_SQRT_3, SQRT_3};

use fixed::types::I16F16;

// pub struct MovingReferenceFrame {
//     pub d: I16F16,
//     pub q: I16F16,
// }

pub struct TwoPhaseStationaryOrthogonalReferenceFrame {
    pub alpha: I16F16,
    pub beta: I16F16,
}

pub struct ThreePhaseStationaryReferenceFrame {
    pub a: I16F16,
    pub b: I16F16,
    /// C is optional if a + b + c equals zero.
    pub c: Option<I16F16>,
}

pub fn clarke(
    inputs: ThreePhaseStationaryReferenceFrame,
) -> TwoPhaseStationaryOrthogonalReferenceFrame {
    if let Some(c) = inputs.c {
        TwoPhaseStationaryOrthogonalReferenceFrame {
            // Eq1
            alpha: (2 * inputs.a) / 3 + (inputs.b - c) / 3,
            // Eq2
            beta: 2 * FRAC_1_SQRT_3 * (inputs.b - c),
        }
    } else {
        TwoPhaseStationaryOrthogonalReferenceFrame {
            // Eq3
            alpha: inputs.a,
            // Eq4
            beta: FRAC_1_SQRT_3 * (inputs.a + 2 * inputs.b),
        }
    }
}

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
