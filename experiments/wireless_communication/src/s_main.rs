use esp_idf_hal::delay::BLOCK;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::prelude::*;
use esp_idf_hal::uart::*;
use std::time::Duration;
use std::thread;

// --- 設定値 ---
// 自分のアドレス (0x0001)
const MY_ADDR_H: u8 = 0x00;
const MY_ADDR_L: u8 = 0x01;
// 自分のチャンネル (0ch)
const MY_CHAN: u8 = 0x00;

// 送信先の設定 (0x0002の相手へ送る)
const TARGET_ADDR_H: u8 = 0x00;
const TARGET_ADDR_L: u8 = 0x02;
const TARGET_CHAN: u8 = 0x00;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let pins = peripherals.pins;

    // 1. ピンの準備
    let mut m0 = PinDriver::output(pins.gpio19)?;
    let mut m1 = PinDriver::output(pins.gpio21)?;

    // UART設定 (E220デフォルトは9600bps)
    let config = UartConfig::new().baudrate(9600.Hz());
    let uart = UartDriver::new(
        peripherals.uart2,
        pins.gpio17, // TX -> RX
        pins.gpio16, // RX -> TX
        Option::<esp_idf_hal::gpio::Gpio0>::None,
        Option::<esp_idf_hal::gpio::Gpio0>::None,
        &config
    )?;

    println!("--- Initializing E220 ---");

    // ==========================================
    // ステップ1: 設定モード (Mode 3) に入る
    // ==========================================
    // M0=1, M1=1
    m0.set_high()?;
    m1.set_high()?;
    println!("Switched to Config Mode (M0=1, M1=1)");

    // モード切替後の安定待ち (重要)
    thread::sleep(Duration::from_millis(100));

    // ==========================================
    // ステップ2: 設定コマンドを送信する
    // ==========================================
    // データシートに基づく設定コマンド (C0: 一時保存, C2: 永続保存)
    // ここでは電源を切ると消える "C0" (一時設定) を使います。
    // 永続保存したい場合は先頭を 0xC2 に変えてください。

    let cfg_cmd = [
        0xC0, // Command: Set Register (RAM)
        0x00, // Start Register Address
        0x09, // Length (設定するバイト数)

        // --- ここから中身 ---
        MY_ADDR_H, // Addr High
        MY_ADDR_L, // Addr Low
        0x62,      // SPED: 8N1, 9600bps, AirRate 2.4k (デフォルト推奨)
        0x40,      // OPTION: ビット6が"1"だと定点送信モード (0x40 = 01000000)
        MY_CHAN,   // CHAN: チャンネル
        0x00,      // CRYPT_H (暗号化キーH)
        0x00,      // CRYPT_L (暗号化キーL)
    ];

    uart.write(&cfg_cmd)?;
    println!("Configuration command sent.");

    // 設定完了待ち & レスポンス読み捨て
    thread::sleep(Duration::from_millis(200));
    let mut temp_buf = [0u8; 100];
    // E220は設定コマンドを受け取ると、設定された値をエコーバックしてきます。
    // バッファに残っていると邪魔なので読み捨てます。
    if let Ok(len) = uart.read(&mut temp_buf, 100) {
        println!("Config Response (Discarded): {:?}", &temp_buf[..len]);
    }

    // ==========================================
    // ステップ3: ノーマルモード (Mode 0) に戻す
    // ==========================================
    // M0=0, M1=0
    m0.set_low()?;
    m1.set_low()?;
    println!("Switched to Normal Mode (M0=0, M1=0)");

    // モード切替後の安定待ち
    thread::sleep(Duration::from_millis(100));

    // ==========================================
    // ステップ4: 定点送信を行うループ
    // ==========================================
    let mut counter = 0;

    loop {
        let message = format!("Hello Target! cnt:{}", counter);
        let msg_bytes = message.as_bytes();

        // 定点送信パケットの作成
        // [宛先High, 宛先Low, 宛先Ch, データ...]
        let mut packet = Vec::new();
        packet.push(TARGET_ADDR_H);
        packet.push(TARGET_ADDR_L);
        packet.push(TARGET_CHAN);
        packet.extend_from_slice(msg_bytes);

        // 送信
        uart.write(&packet)?;
        println!("Sent to {:02X}{:02X} (Ch{}): {}", TARGET_ADDR_H, TARGET_ADDR_L, TARGET_CHAN, message);

        counter += 1;
        thread::sleep(Duration::from_secs(3));
    }
}
