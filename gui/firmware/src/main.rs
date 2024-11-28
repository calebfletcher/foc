//! CDC-ACM serial port example using cortex-m-rtic.
//! Target board: Blue Pill
#![no_main]
#![no_std]
#![allow(non_snake_case)]

use panic_rtt_target as _;

#[rtic::app(device = stm32f1xx_hal::pac)]
mod app {
    use cortex_m::asm::delay;
    use rtt_target::rtt_init_default;
    use stm32f1xx_hal::prelude::*;
    use stm32f1xx_hal::usb::{Peripheral, UsbBus, UsbBusType};
    use usb_device::prelude::*;
    use usb_device::test_class::TestClass;

    #[shared]
    struct Shared {
        usb_dev: UsbDevice<'static, UsbBusType>,
        //class: usb_device::test_class::TestClass<'static, UsbBusType>,
        //serial: usbd_serial::SerialPort<'static, UsbBusType>,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        rtt_init_default!();

        static mut USB_BUS: Option<usb_device::bus::UsbBusAllocator<UsbBusType>> = None;

        let mut flash = cx.device.FLASH.constrain();
        let rcc = cx.device.RCC.constrain();

        let clocks = rcc
            .cfgr
            .use_hse(8.MHz())
            .sysclk(48.MHz())
            .pclk1(24.MHz())
            .freeze(&mut flash.acr);

        assert!(clocks.usbclk_valid());

        let mut gpioa = cx.device.GPIOA.split();

        // BluePill board has a pull-up resistor on the D+ line.
        // Pull the D+ pin down to send a RESET condition to the USB bus.
        // This forced reset is needed only for development, without it host
        // will not reset your device when you upload new firmware.
        let mut usb_dp = gpioa.pa12.into_push_pull_output(&mut gpioa.crh);
        usb_dp.set_low();
        delay(clocks.sysclk().raw() / 100);

        let usb_dm = gpioa.pa11;
        let usb_dp = usb_dp.into_floating_input(&mut gpioa.crh);

        let usb = Peripheral {
            usb: cx.device.USB,
            pin_dm: usb_dm,
            pin_dp: usb_dp,
        };

        unsafe {
            USB_BUS.replace(UsbBus::new(usb));
        }

        //let class = TestClass::new(unsafe { USB_BUS.as_ref().unwrap() });
        //let serial = usbd_serial::SerialPort::new(unsafe { USB_BUS.as_ref().unwrap() });

        let usb_dev = UsbDeviceBuilder::new(
            unsafe { USB_BUS.as_ref().unwrap() },
            UsbVidPid(0x16c0, 0x27d8),
        )
        .manufacturer("github.com/calebfletcher/foc")
        .product("Motor Controller")
        .device_class(0xFF)
        .build();
        //let usb_dev = class.make_device(unsafe { USB_BUS.as_ref().unwrap() });

        (Shared { usb_dev }, Local {})
    }

    #[task(binds = USB_HP_CAN_TX, shared = [usb_dev])]
    fn usb_tx(cx: usb_tx::Context) {
        let mut usb_dev = cx.shared.usb_dev;
        //let mut class = cx.shared.class;

        (&mut usb_dev,).lock(|usb_dev| {
            super::usb_poll(usb_dev);
        });
    }

    #[task(binds = USB_LP_CAN_RX0, shared = [usb_dev])]
    fn usb_rx0(cx: usb_rx0::Context) {
        let mut usb_dev = cx.shared.usb_dev;
        //let mut class = cx.shared.class;

        (&mut usb_dev,).lock(|usb_dev| {
            super::usb_poll(usb_dev);
        });
    }
}

fn usb_poll<B: usb_device::bus::UsbBus>(
    usb_dev: &mut usb_device::prelude::UsbDevice<'static, B>,
    //class2: &mut usb_device::test_class::TestClass<'static, B>,
    //serial: &mut usbd_serial::SerialPort<'static, B>,
) {
    if !usb_dev.poll(&mut [&mut class()]) {
        return;
    }
}

use usbd_microsoft_os::{os_20, utf16_lit, utf16_null_le_bytes, MsOsUsbClass, WindowsVersion};

const DESCRIPTOR_SET: os_20::DescriptorSet = os_20::DescriptorSet {
    version: WindowsVersion::MINIMAL,
    features: &[],
    configurations: &[os_20::ConfigurationSubset {
        configuration: 0,
        features: &[],
        functions: &[os_20::FunctionSubset {
            first_interface: 3,
            features: &[
                os_20::FeatureDescriptor::CompatibleId {
                    id: b"WINUSB\0\0",
                    sub_id: b"\0\0\0\0\0\0\0\0",
                },
                os_20::FeatureDescriptor::RegistryProperty {
                    data_type: os_20::PropertyDataType::RegMutliSz,
                    name: &utf16_lit::utf16_null!("DeviceInterfaceGUIDs"),
                    data: &utf16_null_le_bytes!("{6b09aac4-333f-4467-9e23-f88b9e9d95f7}\0"),
                },
            ],
        }],
    }],
};

const CAPABILITIES: os_20::Capabilities = os_20::Capabilities {
    infos: &[os_20::CapabilityInfo {
        descriptors: &DESCRIPTOR_SET,
        alt_enum_cmd: os_20::ALT_ENUM_CODE_NOT_SUPPORTED,
    }],
};

const DESCRIPTOR_SET_BYTES: [u8; DESCRIPTOR_SET.size()] = DESCRIPTOR_SET.descriptor();
const CAPABILITIES_BYTES: [u8; CAPABILITIES.data_len()] = CAPABILITIES.descriptor_data();

pub const fn class() -> MsOsUsbClass {
    MsOsUsbClass {
        os_20_capabilities_data: &CAPABILITIES_BYTES,
        os_20_descriptor_sets: &[&DESCRIPTOR_SET_BYTES],
    }
}
