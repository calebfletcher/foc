# GUI and STM32 Firmware for FOC Tuning


The three crates in this folder are:
- `icd`: Interface that defines the `postcard-rpc` endpoints used to communicate between the GUI and the microcontroller.
- `gui`: Graphical interface for talking to the microcontroller and allows tuning of the motor controller parameters.
- `firmware`: STM32 firmware that implements a motor controller and uses the ICD to communicate over RTT.