use esp_idf_hal::delay::BLOCK;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::prelude::*;
use esp_idf_hal::uart::*;
use esp_idf_hal::uart::config::*; // Parityを使うために必須
use std::time::Duration;
use std::thread;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let pins = peripherals.pins;

    // --- 1. ピン設定 ---
    let mut m0 = PinDriver::output(pins.gpio19)?;
    let mut m1 = PinDriver::output(pins.gpio21)?;

    // AUXピンは使いません（時間制御で管理します）

    // --- 2. UART初期化 (9600bps, 8N1) ---
    // ここでパリティを None に指定します
    let mut config = UartConfig::new().baudrate(9600.Hz());
    config.parity = Parity::ParityNone;

    let uart = UartDriver::new(
        peripherals.uart2,
        pins.gpio17, // TX
        pins.gpio16, // RX
        Option::<esp_idf_hal::gpio::Gpio0>::None,
        Option::<esp_idf_hal::gpio::Gpio0>::None,
        &config
    )?;

    println!("--- LoRa E220 Driver System Start ---");
    thread::sleep(Duration::from_millis(1000));

    // ================================================================
    // Phase 1: 設定モードで設定を強制上書き (Config)
    // ================================================================
    println!(">> 1. Enter Config Mode (M0=1, M1=1)");
    m0.set_high()?;
    m1.set_high()?;

    // モード切替安定待ち (AUX監視の代わりに500ms待つ)
    thread::sleep(Duration::from_millis(500));

    // 修復コマンド (保存, 9600bps, 8N1, 透過モード)
    // これを起動時に毎回送ることで、常に正常な状態を保ちます
    let fix_cmd = [
        0xC2, 0x00, 0x08, 0x00, 0x00, 0x62, 0x01, 0x00, 0x00, 0x00, 0x00
    ];

    println!(">> 2. Sending Configuration...");
    uart.write(&fix_cmd)?;

    // 書き込み処理待ち (念の為1秒待つ)
    thread::sleep(Duration::from_millis(1000));

    // 応答を確認（読み捨てるだけでもOK）
    let mut buf = [0u8; 128];
    if let Ok(size) = uart.read(&mut buf, 100) {
        println!(">> Config Response: {:?}", &buf[..size]);
        // [..., 98(0x62), ...] が見えたら設定完了
    }

    // ================================================================
    // Phase 2: ノーマルモードへ移行 (Comm)
    // ================================================================
    println!(">> 3. Switch to Normal Mode (M0=0, M1=0)");
    m0.set_low()?;
    m1.set_low()?;

    // 切り替え待ち
    thread::sleep(Duration::from_millis(500));

    println!("--- Ready to Chat ---");
    let mut counter = 0;

    loop {
        // ==========================================
        //  受信処理
        // ==========================================
        let mut rx_buf = [0u8; 256];
        match uart.read(&mut rx_buf, 10) {
            Ok(size) if size > 0 => {
                if let Ok(s) = std::str::from_utf8(&rx_buf[..size]) {
                    println!("Received: {}", s);
                } else {
                    println!("Received (Hex): {:?}", &rx_buf[..size]);
                }
            }
            _ => {}
        }

        // ==========================================
        //  送信処理
        // ==========================================
        if counter % 30 == 0 {
            let msg = format!("Rust Loop: {}\r\n", counter / 30);
            uart.write(msg.as_bytes())?;
            println!("Sent: {}", msg.trim());
        }

        counter += 1;
        thread::sleep(Duration::from_millis(100));
    }
}
