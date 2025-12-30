use esp_idf_hal::delay::BLOCK;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::prelude::*;
use esp_idf_hal::uart::*;
use std::time::Duration;
use std::thread;

// --- 受信側の設定 ---
// 送信側が「0x0002」宛に送っているので、自分のアドレスを「0x0002」にします
const MY_ADDR_H: u8 = 0x00;
const MY_ADDR_L: u8 = 0x02;

// チャンネル (送信側と同じにする必要があります)
const MY_CHAN: u8 = 0x00;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let pins = peripherals.pins;

    // 1. ピン準備
    let mut m0 = PinDriver::output(pins.gpio19)?;
    let mut m1 = PinDriver::output(pins.gpio21)?;

    // UART設定 (E220デフォルト9600bps)
    let config = UartConfig::new().baudrate(9600.Hz());
    let uart = UartDriver::new(
        peripherals.uart2,
        pins.gpio17, // TX -> RX
        pins.gpio16, // RX -> TX
        Option::<esp_idf_hal::gpio::Gpio0>::None,
        Option::<esp_idf_hal::gpio::Gpio0>::None,
        &config
    )?;

    println!("--- Initializing E220 Receiver ---");

    // ==========================================
    // ステップ1: 設定モード (Mode 3) へ
    // ==========================================
    m0.set_high()?;
    m1.set_high()?;
    println!("Switched to Config Mode");
    thread::sleep(Duration::from_millis(100));

    // ==========================================
    // ステップ2: 自分自身の設定を書き込む
    // ==========================================
    // アドレス: 0x0002, 定点送信モード(0x40): ON
    let cfg_cmd = [
        0xC0,      // 一時保存 (電源断で消える)
        0x00,      // 開始アドレス
        0x09,      // 長さ

        MY_ADDR_H, // Addr High (0x00)
        MY_ADDR_L, // Addr Low  (0x02)
        0x62,      // SPED: 8N1, 9600bps, AirRate 2.4k
        0x40,      // OPTION: 定点送信モード有効 (ビット6=1)
        MY_CHAN,   // CHAN: 00ch
        0x00,      // CRYPT_H
        0x00,      // CRYPT_L
    ];

    uart.write(&cfg_cmd)?;
    println!("Configuration sent: Addr {:02X}{:02X}", MY_ADDR_H, MY_ADDR_L);

    // 設定レスポンスの読み捨て
    thread::sleep(Duration::from_millis(200));
    let mut temp_buf = [0u8; 100];
    if let Ok(len) = uart.read(&mut temp_buf, 100) {
        // デバッグ用にレスポンスを見たい場合はコメントアウトを外す
        // println!("Config Response: {:?}", &temp_buf[..len]);
    }

    // ==========================================
    // ステップ3: ノーマルモード (Mode 0) へ戻る
    // ==========================================
    m0.set_low()?;
    m1.set_low()?;
    println!("Switched to Normal Mode. Waiting for data...");
    thread::sleep(Duration::from_millis(100));

    // ==========================================
    // ステップ4: 受信ループ
    // ==========================================
    let mut buf = [0u8; 256];

    loop {
        // データが来るまでブロック待機 (BLOCK)
        match uart.read(&mut buf, BLOCK) {
            Ok(size) if size > 0 => {
                println!("Received {} bytes", size);

                // バイト列を表示
                // println!("Raw: {:?}", &buf[..size]);

                // 文字列として表示
                if let Ok(s) = std::str::from_utf8(&buf[..size]) {
                    println!("Message: {}", s);
                } else {
                    println!("(Binary Data)");
                }
            }
            Ok(_) => {}, // サイズ0の場合は何もしない
            Err(e) => {
                println!("Error: {:?}", e);
                // エラー時は少し待つ
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
}
