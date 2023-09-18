use anyhow::{bail, Result};
use embedded_svc::wifi::{
    AuthMethod, ClientConfiguration, Configuration,
};
use esp_idf_hal::peripheral;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    nvs::EspDefaultNvsPartition,
    wifi::{EspWifi, BlockingWifi},
    //netif::{EspNetif, BlockingNetif},
};
use log::info;
use std::time::Duration;
// use std::net::Ipv4Addr;


pub struct WifiConfig {
    pub ssid: &'static str,
    pub psk: &'static str,
}

pub fn wifi(
    ssid: &str,
    pass: &str,
    modem: impl peripheral::Peripheral<P = esp_idf_hal::modem::Modem> + 'static,
    sysloop: EspSystemEventLoop,
    storage: EspDefaultNvsPartition,
) -> Result<BlockingWifi<EspWifi<'static>>> {
    log::info!("connecting to wifi");
    let mut auth_method = AuthMethod::WPA3Personal;
    if ssid.is_empty() {
        bail!("Missing WiFi name")
    }
    if pass.is_empty() {
        auth_method = AuthMethod::None;
        info!("Wifi password is empty");
    }
    let wifi = EspWifi::new(modem, sysloop.clone(), Some(storage))?;
    let mut wifi = BlockingWifi::wrap(wifi, sysloop).expect("failed to create blocking wifi");


    wifi.set_configuration(&Configuration::Client(
        ClientConfiguration {
            ssid: ssid.into(),
            password: pass.into(),
            auth_method,
            ..Default::default()
        }
    ))?;

    info!("Starting wifi...");
    wifi.start()?;
    info!("wifi started: {:?}", wifi.is_started());


    //wifi.wifi_wait_while(|| wifi.is_started(), Some(Duration::from_secs(21)))?;

    info!("wifi created");
    //info!("Wifi created, about to scan");

    std::thread::sleep(std::time::Duration::from_secs(5));

    // let ap_infos = wifi.scan()?;

    // let ours = ap_infos.into_iter().find(|a| a.ssid == ssid);

    // let channel = if let Some(ours) = ours {
    //     info!(
    //         "Found configured access point {} on channel {}",
    //         ssid, ours.channel
    //     );
    //     Some(ours.channel)
    // } else {
    //     info!(
    //         "Configured access point {} not found during scanning, will go with unknown channel",
    //         ssid
    //     );
    //     None
    // };

    info!("Connecting wifi...");

    wifi.connect()?;
    info!("wifi connected: {:?}", wifi.is_connected());
    std::thread::sleep(Duration::from_secs(5));

    // wifi.ip_wait_while(
    //     || {
    //         //wifi.is_connected().and_then(|_| wifi.wifi().sta_netif().get_ip_info().unwrap().ip != Ipv4Addr::new(0, 0, 0, 0))
    //         wifi.is_connected()
    //     },
    //     Some(Duration::from_secs(20))
    // )?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;

    info!("Wifi DHCP info: {:?}", ip_info);

    Ok(wifi)
}
