use esp_idf_hal::prelude::*;
use esp_idf_hal::uart;
use esp_idf_hal::gpio::AnyIOPin;
use anyhow::Result;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::PinDriver;

const MODE: i32 = 1;

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;
    let config = uart::config::Config::default().baudrate(Hertz(9_600));
    let uart  = uart::UartDriver::new(
        peripherals.uart2,
        pins.gpio17,
        pins.gpio16,
        Option::<AnyIOPin>::None,
        Option::<AnyIOPin>::None,
        &config
    ).unwrap();
    let mut buf = [0u8; 100]; // バッファサイズは適宜調整
    loop{
        if MODE == 0 {
            println!("[Lo-Ra]Sent the message");
            uart.write(&[0x00, 0x0A, 0x05, 0x68, 0x65, 0x6C, 0x6C, 0x6F]).unwrap();
            //FreeRtos::delay_ms(1000);
        }else{
            match uart.read(&mut buf, 100) { // 100 tick 待機 (環境によるが約1秒)
                        Ok(size) if size > 0 => {
                            // 受信データがあれば表示
                            println!("Received {} bytes: {:?}", size, &buf[..size]);

                            // 文字列として表示したい場合
                            if let Ok(s) = std::str::from_utf8(&buf[..size]) {
                                println!("String: {}", s);
                            }
                        }
                        Ok(_) => {
                            // タイムアウトでデータなし。何もしない
                        }
                        Err(e) => {
                            println!("UART Read Error: {:?}", e);
                        }
                    }
        }

    }
}
