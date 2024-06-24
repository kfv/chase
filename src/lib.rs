pub mod config;
pub mod transaction;
pub mod trigger;

pub async fn run() {
    if let Err(e) = config::setup_logging() {
        eprintln!("Error setting up logging: {}", e);
        return;
    }

    transaction::watch().await;
}
