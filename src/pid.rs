#![allow(dead_code)]

use simba::scalar::FixedI16F16;

pub struct PIController {
    k_p: FixedI16F16,
    integral: IntegralComponent,
}

impl PIController {
    pub fn update(
        &mut self,
        measurement: FixedI16F16,
        setpoint: FixedI16F16,
        dt: FixedI16F16,
    ) -> FixedI16F16 {
        let error = measurement - setpoint;
        self.k_p * error + self.integral.update(error, dt)
    }
}

pub struct PIDController {
    k_p: FixedI16F16,
    integral: IntegralComponent,
    derivative: DerivativeComponent,
}

impl PIDController {
    pub fn update(
        &mut self,
        measurement: FixedI16F16,
        setpoint: FixedI16F16,
        dt: FixedI16F16,
    ) -> FixedI16F16 {
        let error = measurement - setpoint;
        self.k_p * error + self.integral.update(error, dt) + self.derivative.update(measurement, dt)
    }
}

struct IntegralComponent {
    k_i: FixedI16F16,
    integral: FixedI16F16,
}

impl IntegralComponent {
    fn update(&mut self, error: FixedI16F16, dt: FixedI16F16) -> FixedI16F16 {
        self.integral += error * dt;
        self.k_i * self.integral
    }
}

struct DerivativeComponent {
    k_d: FixedI16F16,
    last_measurement: Option<FixedI16F16>,
}

impl DerivativeComponent {
    fn update(&mut self, measurement: FixedI16F16, dt: FixedI16F16) -> FixedI16F16 {
        let derivative = self
            .last_measurement
            .map(|last| (measurement - last) / dt)
            .unwrap_or(FixedI16F16::from_num(0));

        self.last_measurement = Some(measurement);

        self.k_d * derivative
    }
}
