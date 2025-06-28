use embassy_executor::Spawner;
use embassy_net::{DhcpConfig, Stack, StackResources};
use embassy_time::{Duration, Timer};
use esp_alloc as _;
use esp_backtrace as _;
use esp_println::println;
use esp_wifi::{
    wifi::{
        ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiMode,
        WifiStaDevice, WifiState,
    },
    EspWifiController,
};

use crate::mk_static;

const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("PASSWORD");

#[embassy_executor::task]
async fn connection_task(mut controller: WifiController<'static>) {
    println!("start connection task");
    println!("Device capabilities: {:?}", controller.capabilities());
    loop {
        if esp_wifi::wifi::wifi_state() == WifiState::StaConnected {
            // Wait until we're no longer connected
            controller.wait_for_event(WifiEvent::StaDisconnected).await;
            Timer::after(Duration::from_millis(5000)).await
        }
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = Configuration::Client(ClientConfiguration {
                ssid: SSID.try_into().unwrap(),
                password: PASSWORD.try_into().unwrap(),
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            println!("Starting wifi");
            controller.start_async().await.unwrap();
            println!("Wifi started!");
        }
        println!("About to connect...");

        match controller.connect_async().await {
            Ok(_) => println!("Wifi connected!"),
            Err(e) => {
                println!("Failed to connect to wifi: {e:?}");
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>) {
    stack.run().await
}

pub async fn start_wifi(
    esp_wifi_ctrl: &'static EspWifiController<'static>,
    wifi: esp_hal::peripherals::WIFI,
    mut rng: esp_hal::rng::Rng,
    spawner: Spawner,
) -> &Stack<WifiDevice<'static, WifiStaDevice>> {
    let (wifi_interface, controller) =
        esp_wifi::wifi::new_with_mode(&esp_wifi_ctrl, wifi, WifiStaDevice).unwrap();
    let net_seed = rng.random() as u64 | ((rng.random() as u64) << 32);

    let dhcp_config = DhcpConfig::default();

    let net_config = embassy_net::Config::dhcpv4(dhcp_config);

    let stack = &*mk_static!(
        Stack<WifiDevice<'_, WifiStaDevice>>,
        Stack::new(
            wifi_interface,
            net_config,
            mk_static!(StackResources<3>, StackResources::<3>::new()),
            net_seed
        )
    );

    spawner.spawn(connection_task(controller)).ok();
    spawner.spawn(net_task(stack)).ok();

    wait_for_connection(stack).await;

    stack
}

async fn wait_for_connection(stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>) {
    println!("Waiting for link to be up");
    loop {
        if stack.is_link_up() {
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    println!("Waiting to get IP address...");
    loop {
        if let Some(config) = stack.config_v4() {
            println!("Got IP: {}", config.address);
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }
}
