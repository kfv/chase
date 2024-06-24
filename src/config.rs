use std::env;
use std::error::Error;
use std::fs;

pub struct Config {
    pub sol_rpc_endpoint: String,
    pub sol_wss_endpoint: String,
    pub tokens_file: String,
    pub trigger_endpoint: String,
    pub wallets_file: String,
}

impl Config {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let sol_rpc_endpoint = env::var("SOL_RPC_ENDPOINT")?;
        let sol_wss_endpoint = env::var("SOL_WSS_ENDPOINT")?;
        let tokens_file = env::var("TOKENS_FILE")?;
        let trigger_endpoint = env::var("TRIGGER_ENDPOINT")?;
        let wallets_file = env::var("WALLETS_FILE")?;

        Ok(Self {
            sol_rpc_endpoint,
            sol_wss_endpoint,
            tokens_file,
            trigger_endpoint,
            wallets_file,
        })
    }
}

pub fn setup_logging() -> Result<(), fern::InitError> {
    if let Err(e) = fs::create_dir_all("/var/log/chase/") {
        eprintln!("Error creating log directory: {}", e);
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
