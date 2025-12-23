use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::gpio::Pull;
use esp_idf_hal::peripherals::Peripherals;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    //test
    let peripherals = Peripherals::take().unwrap();
    let mut input = PinDriver::input(peripherals.pins.gpio32)?;
    let mut output = PinDriver::input_output(peripherals.pins.gpio22)?;
    input.set_pull(Pull::Up)?;
    loop {
        if input.is_high() {
            output.set_high()?;
        } else {
            output.set_low()?;
        }
    }
}
