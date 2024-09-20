use axum::{
    extract::Extension,
    http::StatusCode,
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio_postgres::{NoTls, Client};
use tokio::time::{Duration, Instant};
use rand::rngs::StdRng;
use rand::SeedableRng;
use rand::Rng;
use log::{info, error, debug};
use env_logger::Env;
use tokio_util::sync::CancellationToken;
use tower_http::cors::{CorsLayer, Any};

#[derive(Serialize, Deserialize)]
struct VolumeRequest {
    pool_address: String,
}

#[derive(Serialize, Deserialize)]
struct VolumeResponse {
    pool_address: String,
    volume: f64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the logger
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    info!("Starting the server...");

    // Database connection
    let (db_client, connection) = tokio_postgres::connect(
        "host=localhost user=myuser password=mypassword dbname=mydb",
        NoTls,
    )
    .await?;

    info!("Connected to the database.");

    // Spawn the connection to run in the background and store the handle
    let connection_handle = tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("Database connection error: {}", e);
        }
    });

    let db_client = Arc::new(db_client);

    // Create a shutdown token
    let shutdown_token = CancellationToken::new();

    // Clone the token for each task
    let shutdown_token_txn = shutdown_token.clone();

    // Spawn the transaction generator task
    let db_client_clone = db_client.clone();
    let txn_handle = tokio::spawn(async move {
        if let Err(e) = generate_transactions(db_client_clone, shutdown_token_txn).await {
            error!("Error generating transactions: {}", e);
        }
        info!("Transaction generator task has completed.");
    });

    // Set up CORS middleware using tower_http
    let cors = CorsLayer::new()
        .allow_origin(Any) // Allow all origins for development; specify exact origins for production
        .allow_methods(Any) // Allow all HTTP methods
        .allow_headers(Any) // Allow all headers
        .max_age(Duration::from_secs(3600)); // Cache preflight response for 1 hour

    // Set up the HTTP server with CORS
    let app = Router::new()
        .route("/volume", post(handle_volume_request))
        .layer(cors) // Apply CORS middleware here
        .layer(Extension(db_client.clone()))
        .layer(Extension(shutdown_token.clone()));

    // Define the server address
    let server_addr = "0.0.0.0:8000".parse()?;
    info!("HTTP server listening on {}", server_addr);

    // Clone the shutdown_token for the server task
    let shutdown_token_server = shutdown_token.clone();

    // Run the server in a separate task
    let server_handle = tokio::spawn(async move {
        if let Err(e) = axum::Server::bind(&server_addr)
            .serve(app.into_make_service())
            .with_graceful_shutdown(shutdown_token_server.cancelled())
            .await
        {
            error!("HTTP server error: {}", e);
        }
    });

    // Wait for Ctrl+C signal to exit
    tokio::signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
    info!("Shutting down gracefully...");

    // Signal the tasks to cancel
    shutdown_token.cancel();

    // Wait for the tasks to finish
    let _ = txn_handle.await;
    let _ = server_handle.await;

    // Explicitly drop the db_client to close the connection
    drop(db_client);

    // Wait for the connection to finish
    let _ = connection_handle.await;

    info!("Server has shut down.");

    Ok(())
}

async fn generate_transactions(
    db_client: Arc<Client>,
    shutdown_token: CancellationToken,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut rng = StdRng::from_entropy(); // Use a `Send` RNG

    // Use an interval to schedule transactions every 20 milliseconds
    let mut interval = tokio::time::interval(Duration::from_millis(20));

    let mut count = 0;
    let mut last_report = Instant::now();

    loop {
        tokio::select! {
            _ = interval.tick() => {
                // Check if shutdown is requested
                if shutdown_token.is_cancelled() {
                    info!("Transaction generator is shutting down...");
                    break;
                }

                // Generate a random transaction signature
                let signature = format!("tx_{}", rng.gen::<u64>());

                // Generate a random amount between 0.1 and 10.0
                let amount: f64 = rng.gen_range(0.1..10.0);

                // Insert the transaction into the database
                insert_transaction(&db_client, &signature, "6d4UYGAEs4Akq6py8Vb3Qv5PvMkecPLS1Z9bBCcip2R7", amount).await?;

                count += 1;

                // Log the transaction
                debug!(
                    "Inserted simulated transaction: signature={}, amount={:.2}",
                    signature, amount
                );

                // Every second, report the number of transactions inserted
                if last_report.elapsed() >= Duration::from_secs(1) {
                    info!(
                        "Inserted {} transactions in the last {:.2} seconds",
                        count,
                        last_report.elapsed().as_secs_f64()
                    );
                    count = 0;
                    last_report = Instant::now();
                }
            }
            _ = shutdown_token.cancelled() => {
                info!("Transaction generator received shutdown signal.");
                break;
            }
        }
    }

    Ok(())
}

async fn insert_transaction(
    db_client: &Client,
    signature: &str,
    pool_address: &str,
    amount: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    db_client
        .execute(
            "INSERT INTO transactions (signature, pool_address, amount, timestamp) VALUES ($1, $2, $3, NOW()) ON CONFLICT DO NOTHING",
            &[&signature, &pool_address, &amount],
        )
        .await?;
    Ok(())
}

async fn handle_volume_request(
    Extension(db_client): Extension<Arc<Client>>,
    Json(payload): Json<VolumeRequest>,
) -> Result<Json<VolumeResponse>, (StatusCode, String)> {
    let pool_address = payload.pool_address.clone();
    let cache_url = "http://127.0.0.1:3030/volume";

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5)) // Set a timeout
        .build()
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to build reqwest client: {}", e),
            )
        })?;

    let req = VolumeRequest {
        pool_address: pool_address.clone(),
    };

    // Attempt to get volume from cache
    match client.post(cache_url).json(&req).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<VolumeResponse>().await {
                    Ok(volume_response) => {
                        debug!(
                            "Cache hit for pool_address={}: volume={}",
                            volume_response.pool_address, volume_response.volume
                        );
                        return Ok(Json(volume_response));
                    }
                    Err(e) => {
                        error!("Failed to parse cache response: {}", e);
                        // Proceed to query the database directly
                    }
                }
            } else {
                debug!(
                    "Cache miss or error for pool_address={}: status={}",
                    pool_address,
                    response.status()
                );
                // Proceed to query the database directly
            }
        }
        Err(e) => {
            error!("Failed to send request to cache server: {}", e);
            // Proceed to query the database directly
        }
    }

    // If cache miss or error, query the database
    match get_volume_from_db(&db_client, &pool_address).await {
        Ok(volume) => {
            let volume_response = VolumeResponse {
                pool_address: pool_address.clone(),
                volume,
            };

            // Optionally, you can update the cache here by sending the volume to the cache server

            Ok(Json(volume_response))
        }
        Err(e) => {
            error!("Failed to fetch volume from database: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to fetch volume from database".to_string(),
            ))
        }
    }
}

async fn get_volume_from_db(
    db_client: &Arc<Client>,
    pool_address: &str,
) -> Result<f64, tokio_postgres::Error> {
    let row = db_client
        .query_one(
            "SELECT COALESCE(SUM(amount), 0)::DOUBLE PRECISION as volume FROM transactions WHERE pool_address = $1 AND timestamp >= NOW() - INTERVAL '5 minutes'",
            &[&pool_address],
        )
        .await?;

    let volume: f64 = row.get("volume");
    Ok(volume)
}

