use std::sync::{Arc, Mutex};

use anyhow::Error;
use esp_idf_hal::{delay::FreeRtos, gpio::PinDriver};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::peripherals::Peripherals,
    nvs::EspDefaultNvsPartition,
    wifi::{BlockingWifi, EspWifi},
};

mod http_server;
mod types;
mod utils;

const WIFI_SSID: &str = "nhrdev";
const WIFI_PASS: &str = "1234_Nhr";

fn main() -> Result<(), Error> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    let sys_loop = EspSystemEventLoop::take()?;

    let nvs = EspDefaultNvsPartition::take()?;

    let led_pin = PinDriver::output(peripherals.pins.gpio2)?;

    let shared_state = Arc::new(Mutex::new(types::state::State {
        led_on: false,
        led_pin,
    }));

    let mut esp_wifi = EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs.clone()))?;

    let mut wifi = BlockingWifi::wrap(&mut esp_wifi, sys_loop.clone())?;

    utils::wifi::connect(WIFI_SSID, WIFI_PASS, &mut wifi)?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info();
    log::info!("[+] Wifi DHCP IP info: {:?}", ip_info);

    let _server = http_server::main::run(shared_state.clone())?;

    log::info!("[+] Http server started");

    log::info!("[+] Alhamdulillah :)");

    let mut counter = 0;

    loop {
        // this loop will end after an hour to keep the ESP32 running for an hour
        if counter >= 60 * 60 {
            break;
        }

        // if !led_is_on {
        //     let err = &led.set_high();
        //     match err {
        //         Err(e) => {
        //             log::error!("[!] Failed to turn on the LED: {:?}", e);
        //         }
        //         _ => {
        //             led_is_on = true;
        //             log::info!("[+] LED is turned on");
        //         }
        //     };
        // } else {
        //     let err = &led.set_low();

        //     match err {
        //         Err(e) => {
        //             log::error!("[!] Failed to turn off the LED: {:?}", e);
        //         }
        //         _ => {
        //             led_is_on = false;
        //             log::info!("[+] LED is turned off");
        //         }
        //     };
        // }

        log::info!("[+] main fn is alive for: {} seconds", counter);
        counter += 1;

        FreeRtos::delay_ms(1000u32);
    }

    Ok(())
}
