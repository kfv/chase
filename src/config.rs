use std::env;
use std::error::Error;
use std::fs;
use log::error;

#[derive(Clone)]
pub struct Config {
    pub sol_rpc_endpoint: String,
    pub sol_wss_endpoint: String,
    pub tokens_file: String,
    pub trigger_endpoint: String,
    pub trigger_api_token: String,
    pub wallets_file: String,
    pub import_tokens_endpoint: String,
    pub import_wallets_endpoint: String,
    pub import_interval_seconds: u64,
}

impl Config {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let sol_rpc_endpoint = env::var("SOL_RPC_ENDPOINT")?;
        let sol_wss_endpoint = env::var("SOL_WSS_ENDPOINT")?;
        let tokens_file = env::var("TOKENS_FILE")?;
        let trigger_endpoint = env::var("TRIGGER_ENDPOINT")?;
        let trigger_api_token = env::var("TRIGGER_API_TOKEN")?;
        let wallets_file = env::var("WALLETS_FILE")?;
        let import_tokens_endpoint = env::var("IMPORT_TOKENS_ENDPOINT").unwrap_or_else(|_| "https://api.ompfinex.com/internal/v1/supported-tokens?networkId=".to_string());
        let import_wallets_endpoint = env::var("IMPORT_WALLETS_ENDPOINT").unwrap_or_else(|_| "https://api.ompfinex.com/internal/v1/supported-wallets?networkId=".to_string());
        let import_interval_seconds = env::var("IMPORT_INTERVAL_SECONDS")
            .map(|s| s.parse().unwrap_or(3600))
            .unwrap_or(3600);

        Ok(Self {
            sol_rpc_endpoint,
            sol_wss_endpoint,
            tokens_file,
            trigger_endpoint,
            trigger_api_token,
            wallets_file,
            import_tokens_endpoint,
            import_wallets_endpoint,
            import_interval_seconds,
        })
    }
}

pub fn setup_logging() -> Result<(), fern::InitError> {
    if let Err(e) = fs::create_dir_all("/var/log/chase/") {
        error!("Error creating log directory: {}", e);
    }

    let log_file = fs::File::create("/var/log/chase/application.log")?;
    let logger = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(log_file);

    logger.apply()?;
    Ok(())
}
