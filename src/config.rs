use dotenvy::dotenv;
use eyre::{Context, Result};
use std::env;

/// Application configuration.
#[derive(Debug)]
pub struct Config {
    pub rpc_url: String,
    pub database_url: String,
    pub check_interval_seconds: u64,
    pub trade_amount_usdc: f64,
    pub min_profit_threshold_usd: f64,
    pub simulated_gas_cost_usd: f64,
}

impl Config {
    /// Loads configuration from environment variables.
    pub fn from_env() -> Result<Self> {
        // Load .env file if it exists
        dotenv().ok();

        let rpc_url = env::var("RPC_URL").context("RPC_URL must be set")?;
        let database_url = env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
        let check_interval_seconds = env::var("CHECK_INTERVAL_SECONDS")
            .unwrap_or_else(|_| "60".to_string())
            .parse::<u64>()
            .context("Failed to parse CHECK_INTERVAL_SECONDS")?;
        let trade_amount_usdc = env::var("TRADE_AMOUNT_USDC")
            .unwrap_or_else(|_| "1000.0".to_string())
            .parse::<f64>()
            .context("Failed to parse TRADE_AMOUNT_USDC")?;
        let min_profit_threshold_usd = env::var("MIN_PROFIT_THRESHOLD_USD")
            .unwrap_or_else(|_| "5.0".to_string())
            .parse::<f64>()
            .context("Failed to parse MIN_PROFIT_THRESHOLD_USD")?;
        let simulated_gas_cost_usd = env::var("SIMULATED_GAS_COST_USD")
            .unwrap_or_else(|_| "1.0".to_string())
            .parse::<f64>()
            .context("Failed to parse SIMULATED_GAS_COST_USD")?;

        Ok(Self {
            rpc_url,
            database_url,
            check_interval_seconds,
            trade_amount_usdc,
            min_profit_threshold_usd,
            simulated_gas_cost_usd,
        })
    }
}
