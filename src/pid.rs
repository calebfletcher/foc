#![allow(dead_code)]

use fixed::types::I16F16;

pub struct PIController {
    k_p: I16F16,
    integral: IntegralComponent,
}

impl PIController {
    pub fn new(k_p: I16F16, k_i: I16F16) -> Self {
        Self {
            k_p,
            integral: IntegralComponent {
                k_i,
                integral: I16F16::ZERO,
            },
        }
    }

    pub fn update(&mut self, measurement: I16F16, setpoint: I16F16, dt: I16F16) -> I16F16 {
        let error = measurement - setpoint;
        self.k_p * error + self.integral.update(error, dt)
    }
}

pub struct PIDController {
    k_p: I16F16,
    integral: IntegralComponent,
    derivative: DerivativeComponent,
}

impl PIDController {
    pub fn new(k_p: I16F16, k_i: I16F16, k_d: I16F16) -> Self {
        Self {
            k_p,
            integral: IntegralComponent {
                k_i,
                integral: I16F16::ZERO,
            },
            derivative: DerivativeComponent {
                k_d,
                last_measurement: None,
            },
        }
    }

    pub fn update(&mut self, measurement: I16F16, setpoint: I16F16, dt: I16F16) -> I16F16 {
        let error = measurement - setpoint;
        self.k_p * error + self.integral.update(error, dt) + self.derivative.update(measurement, dt)
    }
}

struct IntegralComponent {
    k_i: I16F16,
    integral: I16F16,
}

impl IntegralComponent {
    fn update(&mut self, error: I16F16, dt: I16F16) -> I16F16 {
        self.integral += self.k_i * error * dt;
        self.integral
    }
}

struct DerivativeComponent {
    k_d: I16F16,
    last_measurement: Option<I16F16>,
}

impl DerivativeComponent {
    fn update(&mut self, measurement: I16F16, dt: I16F16) -> I16F16 {
        let derivative = self
            .last_measurement
            .map(|last| (measurement - last) / dt)
            .unwrap_or(I16F16::ZERO);

        self.last_measurement = Some(measurement);

        self.k_d * derivative
    }
}
