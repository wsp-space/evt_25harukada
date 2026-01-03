use esp_idf_hal::delay::BLOCK;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::prelude::*;
use esp_idf_hal::uart::*;
use esp_idf_hal::uart::config::*;
use std::time::Duration;
use std::thread;

fn main() -> anyhow::Result<()>{
    let mut MODE = false;
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    //Configure Pin
    let peripherals = Peripherals::take()?;
    let pins = peripherals.pins;
    let mut m0 = PinDriver::output(pins.gpio19)?;
    let mut m1 = PinDriver::output(pins.gpio21)?;
    let mut led = PinDriver::output(pins.gpio2)?;

    //Initialize UART
    let mut config = UartConfig::new().baudrate(9600.Hz());
    config.parity = Parity::ParityNone;

    let uart = UartDriver::new(
        peripherals.uart2,
        pins.gpio17,
        pins.gpio16,
        Option::<esp_idf_hal::gpio::Gpio0>::None,
        Option::<esp_idf_hal::gpio::Gpio0>::None,
        &config
    )?;

    println!("--- LoRa E220 Driver System Start ---");
    thread::sleep(Duration::from_millis(1000));

    //Config Mode
    m0.set_high()?;
    m1.set_high()?;
    thread::sleep(Duration::from_millis(500));
    let fix_cmd = [
        0xC2, 0x00, 0x08, 0x00, 0x00, 0x62, 0x01, 0x00, 0x00, 0x00, 0x00
    ];
    uart.write(&fix_cmd)?;
    thread::sleep(Duration::from_millis(1000));
    let mut buf = [0u8; 128];
    if let Ok(size) = uart.read(&mut buf, 100) {
        println!(">> Config Response: {:?}", &buf[..size]);
    }

    //Normal Mode
    m0.set_low()?;
    m1.set_low()?;
    thread::sleep(Duration::from_millis(500));
    let mut counter = 0;
    loop {
        if(MODE){
            if counter % 30 == 0 {
                let msg = format!("sendmsg");
                uart.write(msg.as_bytes())?;
            }
            counter += 1;
            thread::sleep(Duration::from_millis(100));
        }else{
            let mut rx_buf = [0u8; 256];
            match uart.read(&mut rx_buf, 10) {
                Ok(size) if size > 0 => {
                    if let Ok(s) = std::str::from_utf8(&rx_buf[..size]) {
                        if(s == "sendmsg"){
                            led.set_high()?;
                            thread::sleep(Duration::from_millis(100));
                        }
                    } else {
                        println!("Received (Hex): {:?}", &rx_buf[..size]);
                    }
                }
                _ => {}
            }
        }
        led.set_low()?;
    }
}
