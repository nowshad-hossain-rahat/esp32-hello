use anyhow::Error;
use esp_idf_svc::wifi::{AuthMethod, BlockingWifi, Configuration, EspWifi};

pub fn connect(
    ssid: &str,
    password: &str,
    wifi: &mut BlockingWifi<&mut EspWifi<'static>>,
) -> Result<(), Error> {
    log::info!("[+] Trying to connect to the WiFi: {}", ssid);

    let cfg: Configuration = Configuration::Client(esp_idf_svc::wifi::ClientConfiguration {
        ssid: ssid.try_into().unwrap(),
        bssid: None,
        auth_method: AuthMethod::WPA2Personal,
        password: password.try_into().unwrap(),
        channel: None,
        ..Default::default()
    });

    wifi.set_configuration(&cfg)?;
    log::info!("[+] Configuration is set");

    wifi.start()?;
    log::info!("[+] Wifi started");

    wifi.connect()?;
    log::info!("[+] Wifi connected");

    wifi.wait_netif_up()?;
    log::info!("[+] Wifi network interface is up");

    Ok(())
}
