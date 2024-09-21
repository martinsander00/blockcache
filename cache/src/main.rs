use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;

use dotenv::dotenv;
use env_logger::Env;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tokio_postgres::{Client, NoTls};
use warp::Filter;
use warp::http::StatusCode;

#[derive(Deserialize)]
struct VolumeRequest {
    pool_address: String,
}

struct AppState {
    db_client: Arc<Client>,
    cache: Arc<RwLock<HashMap<String, f64>>>,
}

#[derive(Serialize)]
struct VolumeResponse {
    pool_address: String,
    volume: f64,
}

#[tokio::main]
async fn main() {
    // Load environment variables from .env file if present
    dotenv().ok();

    // Initialize logging
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    info!("Starting the cache server...");

    // Database connection
    let (db_client, connection) = tokio_postgres::connect(
        "host=localhost user=myuser password=mypassword dbname=mydb",
        NoTls,
    )
    .await
    .expect("Failed to connect to the database");

    info!("Connected to the database.");

    // Spawn the connection to run in the background
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Database connection error: {}", e);
        }
    });

    let db_client = Arc::new(db_client);

    let cache = Arc::new(RwLock::new(HashMap::new()));

    // Define your pool addresses
    let pool_addresses = vec![
        "6d4UYGAEs4Akq6py8Vb3Qv5PvMkecPLS1Z9bBCcip2R7".to_string(),
        "CWjGo5jkduSW5LN5rxgiQ18vGnJJEKWPCXkpJGxKSQTH".to_string(),
        "7xuPLn8Bun4ZGHeD95xYLnPKReKtSe7zfVRzRJWJZVZW".to_string(),
    ];

    // Initialize cache with pool addresses
    {
        let mut cache_lock = cache.write().await;
        for address in &pool_addresses {
            cache_lock.insert(address.clone(), 0.0);
        }
    }

    let db_client_clone = db_client.clone();
    let cache_clone = cache.clone();
    let pool_addresses_clone = pool_addresses.clone();

    // Spawn the background task to update the cache
    tokio::spawn(async move {
        update_cache_periodically(db_client_clone, cache_clone, pool_addresses_clone).await;
    });

    let state = Arc::new(AppState {
        db_client: db_client.clone(),
        cache: cache.clone(),
    });

    // Define the route
    let volume_route = warp::path("volume")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_state(state.clone()))
        .and_then(handle_volume_request);

    info!("Cache server is running at http://127.0.0.1:3030");

    warp::serve(volume_route).run(([127, 0, 0, 1], 3030)).await;
}

fn with_state(
    state: Arc<AppState>,
) -> impl Filter<Extract = (Arc<AppState>,), Error = Infallible> + Clone {
    warp::any().map(move || state.clone())
}

async fn update_cache_periodically(
    db_client: Arc<Client>,
    cache: Arc<RwLock<HashMap<String, f64>>>,
    pool_addresses: Vec<String>,
) {
    let mut interval = interval(Duration::from_secs(30));

    loop {
        interval.tick().await;

        for pool_address in &pool_addresses {
            match get_volume(&db_client, pool_address).await {
                Ok(volume) => {
                    let mut cache_lock = cache.write().await;
                    cache_lock.insert(pool_address.clone(), volume);
                    info!("Updated cache for pool {}: {}", pool_address, volume);
                }
                Err(e) => {
                    error!("Failed to update cache for pool {}: {}", pool_address, e);
                }
            }
        }
    }
}

async fn get_volume(
    db_client: &Client,
    pool_address: &str,
) -> Result<f64, tokio_postgres::Error> {
    debug!("Querying volume for pool: {}", pool_address);

    let row = db_client
        .query_one(
            "SELECT COALESCE(SUM(amount), 0) as volume FROM transactions WHERE pool_address = $1 AND timestamp >= NOW() - INTERVAL '5 minutes'",
            &[&pool_address],
        )
        .await?;

    let volume: f64 = row.get("volume");

    debug!(
        "Volume for pool {} in the last 5 minutes: {}",
        pool_address, volume
    );

    Ok(volume)
}

async fn handle_volume_request(
    req: VolumeRequest,
    state: Arc<AppState>,
) -> Result<impl warp::Reply, Infallible> {
    info!("Received volume request for pool: {}", req.pool_address);

    let cache_read = state.cache.read().await;

    if let Some(&volume) = cache_read.get(&req.pool_address) {
        let response = VolumeResponse {
            pool_address: req.pool_address,
            volume,
        };
        Ok(warp::reply::with_status(
            warp::reply::json(&response),
            StatusCode::OK,
        ))
    } else {
        error!("Volume not found in cache for pool: {}", req.pool_address);
        let error_message = warp::reply::json(&"Volume not found in cache");
        Ok(warp::reply::with_status(
            error_message,
            StatusCode::NOT_FOUND,
        ))
    }
}

