use std::sync::{Arc, Mutex};

use crate::http_server::esp_http_server_ext::EspHttpRequestExt;
use crate::http_server::types;
use anyhow::Error;
use esp_idf_hal::io::Write;
use esp_idf_svc::http::{
    server::{Configuration, EspHttpServer},
    Method,
};

const MAX_LEN: u32 = 128;
const STACK_SIZE: u32 = 10240;

pub fn run(
    shared_state: Arc<Mutex<crate::types::state::State>>,
) -> Result<EspHttpServer<'static>, Error> {
    let http_config = Configuration::default();
    let mut server = EspHttpServer::new(&http_config)?;

    server.fn_handler("/led", Method::Post, move |mut req| {
        log::info!("[+] Received http request");

        let body = req.read_to_json::<types::ReqData>();

        match body {
            Ok(data) => {
                let state = shared_state.clone();

                {
                    let mut state_data = state.lock().unwrap();
                    let mut msg = "LED turned on".to_string();
                    let mut ok = true;

                    if data.led_on {
                        if state_data.led_on {
                            msg = "LED is already on".to_string();
                        } else {
                            match state_data.led_pin.set_high() {
                                Ok(_) => {
                                    state_data.led_on = true;
                                    msg = "LED turned on".to_string();
                                }
                                Err(e) => {
                                    msg = "Failed to turn on the LED".to_string();
                                    ok = false;
                                }
                            };
                        }
                    } else {
                        if !state_data.led_on {
                            msg = "LED is already off".to_string();
                        } else {
                            match state_data.led_pin.set_low() {
                                Ok(_) => {
                                    state_data.led_on = false;
                                    msg = "LED turned off".to_string();
                                }
                                Err(e) => {
                                    msg = "Failed to turn off the LED".to_string();
                                    ok = false;
                                }
                            };
                        }
                    }

                    req.send_ok_with_json(&types::RespData {
                        ok,
                        message: msg,
                        led_on: Some(state_data.led_on),
                    });
                }
            }
            Err(e) => {
                req.send_server_error(e.to_string());
            }
        };

        Ok::<(), Error>(())
    })?;

    Ok(server)
}
