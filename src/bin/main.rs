#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::timer::systimer::SystemTimer;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::{clock::CpuClock, delay::Delay};
use esp_println as _;
use lilygo_epd47::{pin_config, Display, DrawMode};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // generator version: 0.4.0

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 64 * 1024);

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    info!("Embassy initialized!");

    let rng = esp_hal::rng::Rng::new(peripherals.RNG);
    let timer1 = TimerGroup::new(peripherals.TIMG0);
    let wifi_init = esp_wifi::init(timer1.timer0, rng, peripherals.RADIO_CLK)
        .expect("Failed to initialize WIFI/BLE controller");
    let (mut _wifi_controller, _interfaces) = esp_wifi::wifi::new(&wifi_init, peripherals.WIFI)
        .expect("Failed to initialize WIFI controller");

    // TODO: Spawn some tasks
    let _ = spawner;
    // Initialize display
    //
    esp_alloc::psram_allocator!(peripherals.PSRAM, esp_hal::psram);
    let mut display = Display::new(
        pin_config!(peripherals),
        peripherals.DMA_CH0,
        peripherals.LCD_CAM,
        peripherals.RMT,
    )
    .expect("Failed to initialize display");

    info!("Hello world0!");
    // let delay = Delay::new();

    // delay.delay_millis(100);
    // display.power_on();
    // delay.delay_millis(10);
    // display.clear().unwrap();

    // Draw test circle on the display
    use embedded_graphics::pixelcolor::{Gray4, GrayColor};
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

    loop {
        info!("Hello world!");
        Timer::after(Duration::from_secs(1)).await;
    }
    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-beta.1/examples/src/bin
}
