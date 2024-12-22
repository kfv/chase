pub mod config;
pub mod transaction;
pub mod trigger;

use log::error;

pub async fn run() {
    if let Err(e) = config::setup_logging() {
        error!("Error setting up logging: {}", e);
        return;
    }

    transaction::watch().await;
}
