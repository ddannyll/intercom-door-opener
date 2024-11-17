mod intercom;
use std::sync::{Arc, Mutex};

use tokio::sync::broadcast;

use self::intercom::Intercom;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let (intercom_state_tx, intercom_state_rx) = broadcast::channel(1000);
    let intercom = Arc::new(Mutex::new(Intercom::new(intercom_state_tx)));

    log::info!("Initialising intercom-open ESP32 application");
}
