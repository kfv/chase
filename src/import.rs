use std::fs;
use std::time::Duration;
use log::{error, info};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

use crate::config::Config;

#[derive(Debug, Serialize, Deserialize)]
struct Token {
    name: String,
    mint: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Wallets(Vec<String>);

#[derive(Debug, Deserialize)]
struct ApiToken {
    contract: String,
    decimalPrecision: u8,
    currencyId: String,
}

pub async fn start_import_tasks(config: Config) {
    let config1 = config.clone();
    tokio::spawn(async move {
        import_tokens_loop(config1).await;
    });

    let config2 = config.clone();
    tokio::spawn(async move {
        import_wallets_loop(config2).await;
    });
}

async fn import_tokens_loop(config: Config) {
    let client = Client::new();
    
    loop {
        match import_tokens(&client, &config.import_tokens_endpoint, &config.tokens_file).await {
            Ok(_) => info!("Successfully imported tokens"),
            Err(e) => error!("Failed to import tokens: {}", e),
        }
        
        sleep(Duration::from_secs(config.import_interval_seconds)).await;
    }
}

async fn import_wallets_loop(config: Config) {
    let client = Client::new();
    
    loop {
        match import_wallets(&client, &config.import_wallets_endpoint, &config.wallets_file).await {
            Ok(_) => info!("Successfully imported wallets"),
            Err(e) => error!("Failed to import wallets: {}", e),
        }
        
        sleep(Duration::from_secs(config.import_interval_seconds)).await;
    }
}

async fn import_tokens(client: &Client, endpoint: &str, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let response = client.get(endpoint).send().await?;
    let api_tokens: Vec<ApiToken> = response.json().await?;
    
    // Transform API tokens into our format
    let tokens: Vec<Token> = api_tokens
        .into_iter()
        .map(|api_token| Token {
            name: api_token.currencyId,
            mint: api_token.contract,
        })
        .collect();
    
    let json = serde_json::to_string_pretty(&tokens)?;
    fs::write(file_path, json)?;
    
    Ok(())
}

async fn import_wallets(client: &Client, endpoint: &str, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let response = client.get(endpoint).send().await?;
    let wallets: Vec<String> = response.json().await?;
    
    let wallets = Wallets(wallets);
    let json = serde_json::to_string_pretty(&wallets)?;
    fs::write(file_path, json)?;
    
    Ok(())
} 