use embassy_net::{
    dns::DnsSocket,
    driver::Driver,
    tcp::client::{TcpClient, TcpClientState},
    Stack,
};
use esp_wifi::wifi::{WifiDevice, WifiStaDevice};
use heapless::{String, Vec};
use log::info;
use reqwless::{
    client::{HttpClient, TlsConfig},
    TlsReference,
};
use serde::Deserialize;
use serde_repr::Deserialize_repr;

const API_URL: &str = "https://api.open-meteo.com/v1/forecast?latitude=52.4315&longitude=21.0321&hourly=temperature_2m,wind_speed_10m,wind_direction_10m,wind_gusts_10m,temperature_80m&current=temperature_2m,wind_speed_10m,wind_gusts_10m,wind_direction_10m,rain";

pub struct OpenMeteoApi {
    net_stack: Stack<'static>,
    // url: String<120>,
}

impl OpenMeteoApi {
    pub fn new(net_stack: Stack<'static>) -> Self {
        Self { net_stack }
    }
    pub async fn fetch_data(&self, tls_reference: TlsReference<'_>) {
        let dns = DnsSocket::new(self.net_stack);
        let tcp_state = TcpClientState::<1, 4096, 4096>::new();
        let tcp = TcpClient::new(self.net_stack, &tcp_state);
        let tls_config = TlsConfig::new(
            reqwless::TlsVersion::Tls1_2,
            reqwless::Certificates {
                ca_chain: reqwless::X509::pem(
                    concat!(include_str!("../ca_cert.pem"), "\0").as_bytes(),
                )
                .ok(),
                ..Default::default()
            },
            tls_reference,
        );

        let mut client = HttpClient::new_with_tls(&tcp, &dns, tls_config);
        let mut buffer = [0u8; 4096];
        let mut http_req = client
            .request(reqwless::request::Method::GET, API_URL)
            .await
            .unwrap();
        let response = http_req.send(&mut buffer).await.unwrap();

        info!("Got response");
        let res = response.body().read_to_end().await.unwrap();

        // let (data, _): (WeatherData, _) = serde_json_core::de::from_slice(res).unwrap();
        // data
    }
}
