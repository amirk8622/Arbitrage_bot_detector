use std::sync::Arc;
use std::time::Duration;

use ethers::prelude::*;
use eyre::Result;
use sqlx::SqlitePool;
use tracing::info;

mod arbitrage;
mod config;
mod db;
mod dex;

use crate::arbitrage::check_arbitrage_opportunities;
use crate::config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load configuration from .env file
    let config = Config::from_env()?;
    info!("Configuration loaded successfully.");

    // Set up the database connection pool and run migrations
    let db_pool = db::setup_database(&config.database_url).await?;
    info!("Database connected and migrations are up to date.");

    // Set up the Polygon RPC provider
    let provider = Provider::<Http>::try_from(&config.rpc_url)?;
    let provider = Arc::new(provider);
    info!("Connected to Polygon RPC at {}", config.rpc_url);

    // Define the main check interval
    let mut interval = tokio::time::interval(Duration::from_secs(config.check_interval_seconds));

    info!("Starting arbitrage detection loop...");

    // Main application loop
    loop {
        interval.tick().await;
        let _ = check_arbitrage_opportunities(provider.clone(), &db_pool, &config).await;
    }
}
