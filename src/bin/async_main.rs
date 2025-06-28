#![no_std]
#![no_main]

use embedded_graphics::pixelcolor::Gray4;
use esp_backtrace as _;
use esp_eink_weather::{mk_static, open_meteo::OpenMeteoApi};
use esp_hal::{
    delay::Delay,
    peripheral::Peripheral,
    peripherals::{RSA, SHA},
    prelude::*,
};
use esp_wifi::EspWifiController;
use lilygo_epd47::{pin_config, Display, DrawMode};
use log::info;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

extern crate alloc;

#[main]
async fn main(spawner: Spawner) {
    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });

    esp_alloc::heap_allocator!(3 * 72 * 1024);

    esp_println::logger::init_logger_from_env();

    let timer0 = esp_hal::timer::systimer::SystemTimer::new(peripherals.SYSTIMER)
        .split::<esp_hal::timer::systimer::Target>();
    esp_hal_embassy::init(timer0.alarm0);

    let _ = spawner;
    info!("Embassy initialized!");

    // Initialize wifi
    let timer1 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    let rng = esp_hal::rng::Rng::new(peripherals.RNG);
    let wifi_init = esp_wifi::init(timer1.timer0, rng, peripherals.RADIO_CLK).unwrap();
    let esp_wifi_ctrl = &*mk_static!(EspWifiController<'static>, wifi_init);

    let _wifi_stack =
        esp_eink_weather::wifi::start_wifi(esp_wifi_ctrl, peripherals.WIFI, rng, &spawner).await;

    // Create PSRAM allocator
    esp_alloc::psram_allocator!(peripherals.PSRAM, esp_hal::psram);

    let mut display = Display::new(
        pin_config!(peripherals),
        peripherals.DMA,
        peripherals.LCD_CAM,
        peripherals.RMT,
    )
    .expect("Failed to initialize display");
    info!("EPD47 initialized!");

    let delay = Delay::new();

    delay.delay_millis(100);
    display.power_on();
    delay.delay_millis(10);
    display.clear().unwrap();

    use embedded_graphics::{
        prelude::*,
        primitives::{Circle, PrimitiveStyle},
    };

    Circle::new(display.bounding_box().center() - Point::new(100, 100), 200)
        .into_styled(PrimitiveStyle::with_stroke(Gray4::BLACK, 3))
        .draw(&mut display)
        .unwrap();
    display.flush(DrawMode::BlackOnWhite).unwrap();
    display.power_off();

    spawner
        .spawn(fetch_data(_wifi_stack, peripherals.RSA, peripherals.SHA))
        .ok();

    loop {
        info!("Hello world!");
        Timer::after(Duration::from_secs(1)).await;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/v0.22.0/examples/src/bin
}
#[embassy_executor::task]

async fn fetch_data(stack: embassy_net::Stack<'static>, rsa: RSA, sha: SHA) {
    use esp_mbedtls::Tls;
    let api = OpenMeteoApi::new(stack);

    let tls = Tls::new(sha)
        .expect("TLS::new with peripherals.SHA failed")
        .with_hardware_rsa(rsa);
    api.fetch_data(tls.reference()).await;
}
