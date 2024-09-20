use tokio::io::{self, AsyncBufReadExt};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct VolumeRequest {
    pool_address: String,
}

#[derive(Deserialize)]
struct VolumeResponse {
    pool_address: String,
    volume: f64,
}

#[tokio::main]
async fn main() {
    let stdin = io::BufReader::new(io::stdin());
    let mut lines = stdin.lines();

    println!("Enter commands ('get vol <POOL_ADDRESS>' or 'exit'):");

    while let Ok(Some(line)) = lines.next_line().await {
        let command = line.trim();
        if command == "exit" {
            println!("Exiting...");
            break;
        } else if command.starts_with("get vol") {
            let parts: Vec<&str> = command.split_whitespace().collect();
            if parts.len() == 3 {
                let pool_address = parts[2];
                handle_get_vol(pool_address).await;
            } else {
                println!("Usage: get vol <POOL_ADDRESS>");
            }
        } else {
            println!("Unknown command: {}", command);
        }
    }
}

async fn handle_get_vol(pool_address: &str) {
    let client = reqwest::Client::new();
    let req = VolumeRequest {
        pool_address: pool_address.to_string(),
    };

    match client
        .post("http://127.0.0.1:8000/volume")
        .json(&req)
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<VolumeResponse>().await {
                    Ok(volume_response) => {
                        println!(
                            "5-minute volume for {}: {}",
                            volume_response.pool_address, volume_response.volume
                        );
                    }
                    Err(e) => {
                        println!("Failed to parse response: {}", e);
                    }
                }
            } else {
                println!("Error: Received status code {}", response.status());
            }
        }
        Err(e) => {
            println!("Failed to send request: {}", e);
        }
    }
}

