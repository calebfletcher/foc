#![no_std]
#![forbid(unsafe_code)]

use nalgebra::Matrix3;
use simba::scalar::FixedI16F16;

pub fn clarke() -> Matrix3<FixedI16F16> {
    let sqrt_frac_2_3: FixedI16F16 = FixedI16F16::from_num(0.81649658); // sqrt(2/3)
    let sqrt_3_frac_2 = FixedI16F16::from_num(0.8660254); // sqrt(3)/2
    let frac_1_sqrt_2: FixedI16F16 = FixedI16F16::from_num(core::f32::consts::FRAC_1_SQRT_2); // 1/sqrt(2)

    #[rustfmt::skip]
    let mat = Matrix3::new(
        FixedI16F16::from_num(1), FixedI16F16::from_num(-0.5), FixedI16F16::from_num(-0.5),
        FixedI16F16::from_num(0), sqrt_3_frac_2, FixedI16F16::from_num(0),
        frac_1_sqrt_2, frac_1_sqrt_2, frac_1_sqrt_2
    );

    mat * sqrt_frac_2_3
}

// pub fn park(theta: f32) -> Matrix3<f32> {
//     #[rustfmt::skip]
//     let mat = Matrix3::new(
//         theta.cos(), -0.5, -0.5,
//         0., SQRT_3_FRAC_2, 0.,
//         FRAC_1_SQRT_2, FRAC_1_SQRT_2, FRAC_1_SQRT_2
//     );

//     mat
// }
