pub mod config;
pub mod transaction;
pub mod trigger;

use log::error;

pub async fn run(slot: Option<u64>, dry_run: bool) {
    if let Err(e) = config::setup_logging() {
        error!("Error setting up logging: {}", e);
        return;
    }

    match slot {
        Some(slot_number) => {
            // Process only the specified slot and exit
            transaction::process_slot(slot_number, dry_run).await;
        }
        None => {
            // Watch for new slots continuously
            transaction::watch(dry_run).await;
        }
    }
}
