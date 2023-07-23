
use embedded_svc::wifi::{Configuration, ClientConfiguration, AuthMethod};
use esp_idf_hal::{peripheral::Peripheral, prelude::Peripherals};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::{EspNvsPartition, NvsDefault, EspDefaultNvsPartition}, timer::{EspTimerService, Task, EspTaskTimerService}, wifi::{AsyncWifi, EspWifi}, ping::EspPing, http::server::EspHttpServer};
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use log::*;


const SSID: &str = env!("RUST_ESP32_STD_DEMO_WIFI_SSID");
const PASS: &str = env!("RUST_ESP32_STD_DEMO_WIFI_PASS");

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Hello, world!");

    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take().unwrap();
    let timer_service = EspTaskTimerService::new().unwrap();
    let _wifi = wifi(peripherals.modem, sysloop,Some(EspDefaultNvsPartition::take().unwrap()),timer_service).unwrap();

    let mut server = EspHttpServer::new(&Default::default()).unwrap();

    server.fn_handler("/", embedded_svc::http::Method::Get, move |req| {
        let mut response = req.into_ok_response().unwrap();
        response.write("Hello from Esp32-c3".as_bytes()).unwrap();
        Ok(())
    }).unwrap();
}

    pub fn wifi(
        modem: impl Peripheral<P = esp_idf_hal::modem::Modem> + 'static,
        sysloop: EspSystemEventLoop,
        nvs: Option<EspNvsPartition<NvsDefault>>,
        timer_service: EspTimerService<Task>,
    ) -> anyhow::Result<AsyncWifi<EspWifi<'static>>> {
        use futures::executor::block_on;
        let mut wifi = AsyncWifi::wrap(
            EspWifi::new(modem, sysloop.clone(), nvs)?,
            sysloop,
            timer_service.clone(),
        )?;
    
        block_on(connect_wifi(&mut wifi))?;
    
        let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    
        println!("Wifi DHCP info: {:?}", ip_info);
        
        EspPing::default().ping(ip_info.subnet.gateway, &embedded_svc::ping::Configuration::default())?;
        Ok(wifi)
    
    }

    async fn connect_wifi(wifi: &mut AsyncWifi<EspWifi<'static>>) -> anyhow::Result<()> {
        let wifi_configuration: Configuration = Configuration::Client(ClientConfiguration {
            ssid: SSID.into(),
            bssid: None,
            auth_method: AuthMethod::WPA2Personal,
            password: PASS.into(),
            channel: None,
        });
    
        wifi.set_configuration(&wifi_configuration)?;
    
        wifi.start().await?;
        info!("Wifi started");
    
        wifi.connect().await?;
        info!("Wifi connected");
    
        wifi.wait_netif_up().await?;
        info!("Wifi netif up");
    
        Ok(())
    }