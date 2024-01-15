# FOC

An implementation of [Field Oriented Control](https://en.wikipedia.org/wiki/Field_Oriented_Control) algorithms in Rust, designed for use in embedded systems.

## Goals
- Modular and extendable implementation of FOC algorithms.
- Exclusively use fixed-point math for all FOC calculations, using the [`fixed`](https://crates.io/crates/fixed) crate.
- Support for microcontrollers across the entire embedded Rust ecosystem.
- Support for microcontroller-specific accelerators (e.g. STM32G4/STM32H7 CORDIC peripheral for trig functions, STM32 FMAC peripheral for filters).
- Generic over angle sensors, current sensors, and PWM drivers.
- Straightforward to add custom algorithms.
- No heap allocations anywhere.