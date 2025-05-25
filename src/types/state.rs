use esp_idf_hal::gpio::{Gpio2, Output, PinDriver};

pub struct State {
    pub led_on: bool,
    pub led_pin: PinDriver<'static, Gpio2, Output>,
}
