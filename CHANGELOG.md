# Changelog

## [Unreleased]
### Added
- `pwm::Modulation` trait that all modulation types implement.
### Changed
- Convert most uses of floats to fixed-point math.
- PWM modulation methods are now structs.
### Removed
- Remove `fixed_macro` dependency.

## [0.2.0] - 2024-01-09
### Added
- Add Park & Clarke transformations and their inverses.
- Add PWM calculation methods.
### Changed
- Removed nalgebra, and changed existing API that used it.

## [0.1.0] - 2024-01-06
### Added
- Added FOC struct with basic method (that doesn't work).