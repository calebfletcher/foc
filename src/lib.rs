#![no_std]
#![forbid(unsafe_code)]

use core::f32::consts::FRAC_1_SQRT_2;

use nalgebra::Matrix3;

const SQRT_FRAC_2_3: f32 = 0.81649658; // sqrt(2/3)
const SQRT_3_FRAC_2: f32 = 0.8660254; // sqrt(3)/2

pub fn clarke() -> Matrix3<f32> {
    #[rustfmt::skip]
    let mat = Matrix3::new(
        1., -0.5, -0.5,
        0., SQRT_3_FRAC_2, 0.,
        FRAC_1_SQRT_2, FRAC_1_SQRT_2, FRAC_1_SQRT_2
    );

    SQRT_FRAC_2_3 * mat
}
