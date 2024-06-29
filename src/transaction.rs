use std::collections::HashMap;
use std::fs;
use std::str::FromStr;
use std::sync::Arc;

use chrono::DateTime;
use futures::StreamExt;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::{json, value::Value};
use solana_client::{
    nonblocking::{pubsub_client::PubsubClient, rpc_client::RpcClient},
    rpc_config::RpcBlockConfig,
};
use solana_rpc_client_api::{client_error::ErrorKind, request::RpcError};
use solana_sdk::{clock::UnixTimestamp, commitment_config::CommitmentConfig, signature::Signature};
use solana_transaction_status::{
    EncodedTransaction, TransactionDetails, UiInstruction, UiParsedInstruction,
    UiTransactionEncoding,
};
use tokio::time::{sleep, Duration};

use crate::config::Config;
use crate::trigger::trigger_api;

const SOL_MINT: &str = "So11111111111111111111111111111111111111112";

#[derive(Debug, Deserialize)]
struct Token {
    name: String,
    mint: String,
}

#[derive(Debug, Deserialize)]
struct Wallets(Vec<String>);

#[derive(Debug, Serialize)]
pub struct ParsedTransaction {
    #[serde(rename = "network_id")]
    network: String,
    blocktime: String,
    #[serde(rename = "wallet")]
    destination: String,
    #[serde(rename = "currency_id")]
    token: String,
    #[serde(rename = "currency_address")]
    mint: String,
    amount: String,
    #[serde(rename = "tx_hash")]
    signature: String,
    success: bool,
}

impl ParsedTransaction {
    fn new(
        blocktime: String,
        destination: String,
        token: String,
        mint: String,
        amount: String,
        signature: String,
        success: bool,
    ) -> Self {
        Self {
            network: "SOL".to_string(),
            blocktime,
            destination,
            token,
            mint,
            amount,
            signature,
            success,
        }
    }
}

pub async fn watch() {
    let app_config = Config::new().expect("Failed to load configuration");

    let client = Arc::new(RpcClient::new(app_config.sol_rpc_endpoint.clone()));
    let config = RpcBlockConfig {
        encoding: Some(UiTransactionEncoding::JsonParsed),
        transaction_details: Some(TransactionDetails::Full),
        rewards: Some(true),
        commitment: Some(CommitmentConfig::finalized()),
        max_supported_transaction_version: Some(0),
    };

    let tokens = load_tokens(app_config.tokens_file.as_str());
    let wallets = load_wallets(app_config.wallets_file.as_str());

    let pubsub_client = PubsubClient::new(app_config.sol_wss_endpoint.clone().as_str())
        .await
        .unwrap();
    let (mut slot, unsub) = pubsub_client.slot_subscribe().await.unwrap();

    while let Some(response) = slot.next().await {
        let client = Arc::clone(&client);
        let tokens = tokens.clone();
        let wallets = wallets.clone();
        let trigger_endpoint = app_config.trigger_endpoint.clone();

        tokio::spawn(async move {
            get_block(
                &client,
                response.slot - 64,
                &config,
                &tokens,
                &wallets,
                trigger_endpoint,
            )
            .await;
        });
    }

    unsub().await;
}

pub async fn get_block(
    client: &Arc<RpcClient>,
    slot: u64,
    config: &RpcBlockConfig,
    tokens: &HashMap<String, String>,
    wallets: &HashMap<String, bool>,
    trigger_endpoint: String,
) {
    let mut retries = 0;

    async fn parse_instructions(
        instructions: Vec<UiInstruction>,
        meta: &Value,
        timestamp: i64,
        wallets: &HashMap<String, bool>,
        tokens: &HashMap<String, String>,
        tx_signatures: &[String],
        tx_parsed_result: &mut Vec<ParsedTransaction>,
    ) {
        for instruction in instructions {
            if let UiInstruction::Parsed(UiParsedInstruction::Parsed(parsed_instruction)) = instruction {
                let obj = parsed_instruction.parsed;
                parse_tx(&obj, meta, timestamp, wallets, tokens, tx_signatures, tx_parsed_result);
            }
        }
    }

    loop {
        let mut tx_parsed_result: Vec<ParsedTransaction> = Vec::new();
        match client.get_block_with_config(slot, *config).await {
            Ok(block) => {
                let timestamp = block.block_time.unwrap_or_default();
                for transaction in block.clone().transactions.unwrap_or_default() {
                    let tx_signatures = match transaction.transaction.clone() {
                        EncodedTransaction::Json(json_string) => json_string.signatures,
                        _ => continue,
                    };
                    
                    let meta_ref = match &transaction.meta {
                        Some(meta) => serde_json::to_value(meta).unwrap_or_default(),
                        None => continue,
                    };

                    if let EncodedTransaction::Json(x) = transaction.transaction {
                        if let solana_transaction_status::UiMessage::Parsed(p) = x.message {
                            parse_instructions(p.instructions, &meta_ref, timestamp, wallets, tokens, &tx_signatures, &mut tx_parsed_result).await;
                        }
                    };
                    
                    if let Some(meta) = transaction.meta {
                        let meta_tx: Option<Vec<solana_transaction_status::UiInnerInstructions>> = meta.inner_instructions.into();
                        if let Some(inner_instructions) = meta_tx {
                            for inner_instruction in inner_instructions {
                                parse_instructions(inner_instruction.instructions, &meta_ref, timestamp, wallets, tokens, &tx_signatures, &mut tx_parsed_result).await;
                            }
                        }
                    }
                }

                let signatures: Vec<Signature> = tx_parsed_result
                    .iter()
                    .filter_map(|s| Signature::from_str(s.signature.as_str()).ok())
                    .collect();
                let statuses = match client
                    .get_signature_statuses_with_history(&signatures)
                    .await
                {
                    Ok(res) => res.value,
                    Err(_) => return,
                };
                for (s, status) in tx_parsed_result.iter_mut().zip(statuses) {
                    match status.unwrap().status {
                        Ok(()) => s.success = true,
                        Err(_) => s.success = false,
                    }
                }

                info!("Successfully parsed block {}", slot);
                if !tx_parsed_result.is_empty() {
                    info!("Transaction Received ({}):\n{:#?}", slot, &tx_parsed_result);
                    trigger_api(tx_parsed_result, trigger_endpoint).await;
                }
            }
            Err(e) => match e.kind {
                ErrorKind::RpcError(RpcError::RpcResponseError { code, .. }) => {
                    if code == -32004 {
                        warn!("{} (try: {})", e, retries);
                        retries += 1;
                        if retries > 50 {
                            return;
                        }
                        sleep(Duration::from_secs((1_u64 << retries) / 2)).await;
                        continue;
                    } else {
                        warn!("{}", e);
                        return;
                    }
                }
                _ => {
                    error!("Uncaught error: {} ({})", e, slot);
                    return;
                }
            },
        };
        // If not returned or continued by this point, break
        break;
    }
}

pub fn load_tokens(file_path: &str) -> HashMap<String, String> {
    let contents = fs::read_to_string(file_path).expect("Could not read tokens file");
    let tokens: Vec<Token> = serde_json::from_str(&contents).expect("Error parsing tokens file");
    tokens
        .into_iter()
        .map(|token| (token.mint, token.name))
        .collect()
}

pub fn load_wallets(file_path: &str) -> HashMap<String, bool> {
    let contents = fs::read_to_string(file_path).expect("Could not read wallets file");
    let wallets: Wallets = serde_json::from_str(&contents).expect("Error parsing wallets file");
    wallets.0.into_iter().map(|wallet| (wallet, true)).collect()
}

fn parse_tx(
    obj: &Value,
    meta: &Value,
    timestamp: UnixTimestamp,
    wallets: &HashMap<String, bool>,
    tokens: &HashMap<String, String>,
    tx_signatures: &[String],
    res: &mut Vec<ParsedTransaction>,
) {
    let mut is_wallet_found = false;
    
    let tx_blocktime = DateTime::from_timestamp(timestamp, 0)
        .unwrap_or_default()
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();
    
    let tx_destination = match obj.get("info").and_then(|data| data.get("destination")) {
        Some(dst) => dst.clone().as_str().unwrap_or_default().to_string(),
        None => return,
    };
    
    if !wallets.contains_key(&tx_destination) {
        if let Some(post_token_balances) = meta.get("postTokenBalances").and_then(|val| val.as_array()) {
            for balance in post_token_balances {
                if let Some(owner) = balance.get("owner").and_then(|owner| owner.as_str()) {
                    if wallets.contains_key(owner) {
                        is_wallet_found = true;
                        break;
                    }
                }
            }
        }
    } else {
        is_wallet_found = true;
    }

    if !is_wallet_found { return; }

    match obj.get("type") {
        Some(val) => {
            if !matches!(
                val.as_str().unwrap_or_default(),
                "transfer" | "transferChecked"
            ) {
                return;
            } else {
                val
            }
        }
        _ => return,
    };

    let (tx_amount, tx_token, tx_mint) = match obj.get("info").and_then(|data| data.get("mint")) {
        Some(mint) => {
            if !tokens.contains_key(mint.as_str().unwrap()) { return; }
            let amount = obj
                .get("info")
                .and_then(|data| data.get("tokenAmount").and_then(|data| data.get("uiAmount")))
                .unwrap()
                .clone();
            let token = tokens.get(mint.as_str().unwrap()).unwrap().to_string();
            (amount, token, mint.clone())
        }
        None => match obj.get("info").and_then(|data| data.get("lamports")) {
            Some(lamports) => {
                let amount = (lamports.as_f64().unwrap_or_default() / 1_000_000_000.0).into();
                (amount, "SOL".to_string(), json!(SOL_MINT))
            }
            _ => return,
        },
    };

    let p = ParsedTransaction::new(
        tx_blocktime,
        tx_destination,
        tx_token,
        tx_mint.as_str().unwrap().to_string(),
        tx_amount.to_string(),
        tx_signatures.first().unwrap().to_string(),
        false,
    );

    res.push(p);
}
