use std::{thread::sleep, time::Duration, sync::{Mutex, Arc}, str::from_utf8};

use embedded_svc::{wifi::{Configuration, ClientConfiguration, AuthMethod}, http::Method::Post, io::Read};
use esp_idf_hal::{peripheral::Peripheral, prelude::Peripherals, gpio::PinDriver, ledc::{LedcTimerDriver, config::TimerConfig, LedcDriver}};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::{EspNvsPartition, NvsDefault, EspDefaultNvsPartition}, timer::{EspTimerService, Task, EspTaskTimerService}, wifi::{AsyncWifi, EspWifi}, ping::EspPing, http::server::EspHttpServer};
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use log::*;
use esp_idf_hal::units::*;



const SSID: &str = env!("RUST_ESP32_STD_DEMO_WIFI_SSID");
const PASS: &str = env!("RUST_ESP32_STD_DEMO_WIFI_PASS");

#[derive(Debug)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl TryFrom<&str> for Color {
    type Error = anyhow::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        Ok(Color {
            r:  u8::from_str_radix(&input[0..2], 16)?,
            g:  u8::from_str_radix(&input[2..4], 16)?,
            b:  u8::from_str_radix(&input[4..6], 16)?,
        })
    }
}

   

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

    let led_timer = peripherals.ledc.timer0;
    let led_timer_driver = LedcTimerDriver::new(led_timer, &TimerConfig::new().frequency(1000.Hz())).unwrap();

    let red_channel = Arc::new(Mutex::new(LedcDriver::new(peripherals.ledc.channel0, &led_timer_driver, peripherals.pins.gpio3).unwrap()));
    let green_channel = Arc::new(Mutex::new(LedcDriver::new(peripherals.ledc.channel1, &led_timer_driver, peripherals.pins.gpio4).unwrap()));
    let blue_channel = Arc::new(Mutex::new(LedcDriver::new(peripherals.ledc.channel2, &led_timer_driver, peripherals.pins.gpio5).unwrap()));


    server.fn_handler("/color", Post, move |mut req| {
        let mut buffer = [0_u8; 6];
        req.read_exact(&mut buffer)?;
        let color: Color = from_utf8(&buffer)?.try_into()?;

        println!("Color: {:?}",color);

        let mut response = req.into_ok_response()?;
        response.write("Hello from Esp32-c3".as_bytes())?;
        red_channel.lock().unwrap().set_duty(color.r as u32).unwrap();
        green_channel.lock().unwrap().set_duty(color.g as u32).unwrap();
        blue_channel.lock().unwrap().set_duty(color.b as u32).unwrap();
        Ok(())
    }).unwrap();

    loop {
        sleep(Duration::from_secs(1));
    }
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