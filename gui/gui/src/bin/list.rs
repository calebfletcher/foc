fn main() {
    for device in nusb::list_devices().unwrap() {
        println!(
            "{:04X} {:04X} {:?}",
            device.vendor_id(),
            device.product_id(),
            device.product_string()
        );
    }
}
