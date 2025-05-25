use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReqData {
    pub led_on: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RespData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub led_on: Option<bool>,
    pub message: String,
    pub ok: bool,
}
