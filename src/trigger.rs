use crate::transaction::ParsedTransaction;
use log::{error, info};
use reqwest::StatusCode;
use tokio::time::{sleep, Duration};

pub async fn trigger_api(msg: Vec<ParsedTransaction>, endpoint: String, api_token: String) {
    let client = reqwest::Client::new();
    let max_retries = 5;

    for transaction in msg {
        let data = match serde_json::to_string(&transaction) {
            Ok(data) => data,
            Err(e) => {
                error!("Failed to serialise transaction: {}", e);
                continue;
            }
        };

        for retries in 0..=max_retries {
            match client
                .post(&endpoint)
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", api_token))
                .body(data.clone())
                .send()
                .await
            {
                Ok(response) => {
                    if response.status() == StatusCode::OK {
                        info!("Successfully sent transaction");
                        break;
                    }
                    error!("Failed with status: {}", response.status());
                }
                Err(e) => {
                    error!("Request error: {}", e);
                }
            }

            if retries == max_retries {
                error!("Max retries reached for transaction: {:?}", transaction);
                break;
            }

            let backoff = Duration::from_secs(60 * ((1_u64 << retries.min(4)) - 1));
            info!("Retrying request in {} seconds...", backoff.as_secs());
            sleep(backoff).await;
        }
    }
}
