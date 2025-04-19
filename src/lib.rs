pub mod config;
pub mod transaction;
pub mod trigger;
pub mod import;

use log::error;
use crate::config::Config;

pub async fn run(slot: Option<u64>, dry_run: bool) {
    if let Err(e) = config::setup_logging() {
        error!("Error setting up logging: {}", e);
        return;
    }

    let config = match Config::new() {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            return;
        }
    };

    // Start the import tasks
    import::start_import_tasks(config.clone()).await;

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
