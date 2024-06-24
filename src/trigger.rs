use crate::transaction::ParsedTransaction;
use reqwest::StatusCode;
use tokio::time::{sleep, Duration};

pub async fn trigger_api(msg: Vec<ParsedTransaction>, endpoint: String) {
    let client = reqwest::Client::new();
    let max_retries = 10;

    for transaction in msg {
        let data = serde_json::to_string(&transaction).unwrap();
        let mut retries = 0;

        loop {
            match client
                .post(&endpoint)
                .header("Content-Type", "application/json")
                .body(data.clone())
                .send()
                .await
            {
                Ok(response) => {
                    if response.status() == StatusCode::OK {
                        break;
                    } else {
                        println!("Failed with status: {}", response.status());
                        retries += 1;
                    }
                }
                Err(e) => {
                    println!("Error: {}", e);
                    retries += 1;
                }
            }

            if retries > max_retries {
                println!("Max retries reached for transaction: {:?}", transaction);
                break;
            }

            if retries <= max_retries {
                println!("Retrying request...");
                sleep(Duration::from_secs(60 * ((1_u64 << retries) - 1))).await;
            } else {
                break;
            }
        }
    }
}
