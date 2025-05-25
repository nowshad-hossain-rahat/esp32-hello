use anyhow::Error;
use esp_idf_hal::io::{Read, Write};
use esp_idf_svc::http::server::{EspHttpConnection, Request, Response};
use serde::{Deserialize, Serialize};

pub trait EspHttpRequestExt<'a> {
    fn content_length(&self) -> usize;
    fn read_to_string(&mut self) -> Result<String, Error>;
    fn read_to_json<T: for<'de> Deserialize<'de> + Serialize>(&mut self) -> Result<T, Error>;
    fn send_ok_with_json(self, data: &crate::http_server::types::RespData);
    fn send_bad_request(self);
    fn send_server_error(self, message: String);
}

impl<'a> EspHttpRequestExt<'a> for Request<&mut EspHttpConnection<'a>> {
    fn content_length(&self) -> usize {
        let parsed_value = self
            .header("Content-Length")
            .unwrap_or("0")
            .parse::<usize>();

        match parsed_value {
            Ok(length) => length,
            Err(_) => 0,
        }
    }

    fn read_to_string(&mut self) -> Result<String, Error> {
        let len = self.content_length();
        let mut buf: Vec<u8> = if len == 0 { Vec::new() } else { vec![0u8; len] };

        if len > 0 {
            return match self.read_exact(&mut buf) {
                Ok(()) => {
                    let data = String::from_utf8(buf)?;
                    Ok(data)
                }
                Err(e) => Err(Error::new(e)),
            };
        }

        let mut tmp = vec![0u8; 512];

        loop {
            match self.read(&mut tmp) {
                Ok(0) => break,
                Ok(n) => buf.extend_from_slice(&tmp[..n]),
                Err(e) => {
                    log::error!("{:?}", e);
                    break;
                }
            };
        }

        let data = String::from_utf8(buf)?;
        Ok(data)
    }

    fn read_to_json<T: for<'de> Deserialize<'de> + Serialize>(&mut self) -> Result<T, Error> {
        let len = self.content_length();
        let mut buf: Vec<u8> = if len == 0 { Vec::new() } else { vec![0u8; len] };

        if len > 0 {
            return match self.read_exact(&mut buf) {
                Ok(()) => {
                    let data = serde_json::from_slice::<T>(&buf)?;
                    Ok(data)
                }
                Err(e) => Err(Error::new(e)),
            };
        }

        let mut tmp = vec![0u8; 512];

        loop {
            match self.read(&mut tmp) {
                Ok(0) => break,
                Ok(n) => buf.extend_from_slice(&tmp[..n]),
                Err(e) => {
                    log::error!("{:?}", e);
                    break;
                }
            };
        }

        let data = serde_json::from_slice::<T>(&buf)?;
        Ok(data)
    }

    fn send_ok_with_json(self, data: &crate::http_server::types::RespData) {
        let json: String;
        let mut status: u16 = 200;

        match serde_json::to_string(data) {
            Ok(j) => {
                json = j;
            }
            Err(_) => {
                status = 500;
                json = r#"
                    {
                        "message": "Something went wrong, please try again",
                        "ok": true
                    }
                    "#
                .to_string();
            }
        };

        let mut resp = self
            .into_response(status, None, &[("Content-Type", "application/json")])
            .unwrap();

        resp.write_all(json.as_bytes());
    }

    fn send_bad_request(self) {
        let data = crate::http_server::types::RespData {
            ok: false,
            message: "Bad request".to_string(),
            led_on: None,
        };

        let json: String;
        let mut status = 400;

        match serde_json::to_string(&data) {
            Ok(j) => {
                json = j;
            }
            Err(_) => {
                status = 500;
                json = r#"
                {
                    "message": "Something went wrong, please try again",
                    "ok": true
                }
                "#
                .to_string();
            }
        };

        let mut resp = self
            .into_response(status, None, &[("Content-Type", "application/json")])
            .unwrap();

        let _ = resp.write_all(json.as_bytes());
    }

    fn send_server_error(self, message: String) {
        let data = crate::http_server::types::RespData {
            ok: false,
            message,
            led_on: None,
        };

        let json = match serde_json::to_string(&data) {
            Ok(j) => j,
            Err(_) => r#"
                    {
                        "message": "Something went wrong, please try again",
                        "ok": true
                    }
                    "#
            .to_string(),
        };

        let mut resp = self
            .into_response(500, None, &[("Content-Type", "application/json")])
            .unwrap();

        let _ = resp.write_all(json.as_bytes());
    }
}
