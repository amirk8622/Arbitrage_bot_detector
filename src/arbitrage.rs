use std::sync::Arc;

use ethers::prelude::*;
use sqlx::SqlitePool;
use tracing::{info, warn};

use crate::config::Config;
use crate::db;
use crate::dex::{self, Dex, DexType};

// Token addresses on Polygon
const WETH: &str = "0x7ceb23fd6bc0add59e62ac25578270cff1b9f619";
const USDC: &str = "0x2791bca1f2de4661ed88a30c99a7a9449aa84174";
const WBTC: &str = "0x1bfd67037b42cf73acf2047067bd4f2c47d9bfd6";

// DEX addresses on Polygon
const QUICKSWAP_ROUTER: &str = "0xa5E0829CaCEd8fFDD4De3c43696c57F7D7A678ff";
const SUSHISWAP_ROUTER: &str = "0x1b02dA8Cb0d097eB8D57A175b88c7D8b47997506";
const UNISWAP_V3_QUOTER: &str = "0xb27308f9F90D607463bb33eA174115C648992456";
const UNISWAP_V3_ROUTER: &str = "0xE592427A0AEce92De3Edee1F18E0157C05861564";

/// Main function to check for arbitrage opportunities between a list of DEXes.
pub async fn check_arbitrage_opportunities(
    provider: Arc<Provider<Http>>,
    db_pool: &SqlitePool,
    config: &Config,
) -> eyre::Result<()> {
    info!("Checking for arbitrage opportunities...");

    let weth_addr: Address = WETH.parse()?;
    let usdc_addr: Address = USDC.parse()?;
    let wbtc_addr: Address = WBTC.parse()?;

    let dexes = [
        Dex {
            name: "QuickSwap",
            router_address: QUICKSWAP_ROUTER.parse()?,
            dex_type: DexType::UniswapV2,
        },
        Dex {
            name: "SushiSwap",
            router_address: SUSHISWAP_ROUTER.parse()?,
            dex_type: DexType::UniswapV2,
        },
        Dex {
            name: "Uniswap V3",
            router_address: UNISWAP_V3_ROUTER.parse()?,
            dex_type: DexType::UniswapV3 {
                quoter_address: UNISWAP_V3_QUOTER.parse()?,
                fee: 3000, // 0.3% fee pool, most common for WETH/USDC
            },
        },
    ];

    // We will check for WETH/USDC and WBTC/USDC pairs
    let token_pairs = [
        ("WETH", weth_addr, 18), 
        ("WBTC", wbtc_addr, 8)
    ];
    let usdc_decimals = 6;

    // Amount of USDC to trade (1000 USDC)
    let amount_in_usdc = U256::from(config.trade_amount_usdc as u64) * U256::from(10).pow(U256::from(usdc_decimals));

    for (token_symbol, token_addr, token_decimals) in token_pairs {
        for i in 0..dexes.len() {
            for j in 0..dexes.len() {
                if i == j {
                    continue;
                }

                let buy_dex = &dexes[i];
                let sell_dex = &dexes[j];

                // Scenario: Buy Token on DEX A with USDC, Sell Token on DEX B for USDC
                check_and_log_opportunity(
                    provider.clone(),
                    db_pool,
                    config,
                    buy_dex,
                    sell_dex,
                    "USDC",
                    token_symbol,
                    usdc_addr,
                    token_addr,
                    amount_in_usdc,
                    usdc_decimals,
                    token_decimals,
                )
                .await?;
            }
        }
    }

    Ok(())
}

/// Checks a single path for an arbitrage opportunity.
#[allow(clippy::too_many_arguments)]
async fn check_and_log_opportunity(
    provider: Arc<Provider<Http>>,
    db_pool: &SqlitePool,
    config: &Config,
    buy_dex: &Dex<'_>,
    sell_dex: &Dex<'_>,
    buy_token_symbol: &str,
    sell_token_symbol: &str,
    buy_token_addr: Address,
    sell_token_addr: Address,
    amount_in: U256,
    buy_token_decimals: u8,
    sell_token_decimals: u8,
) -> eyre::Result<()> {
    // 1. Get how much of `sell_token` we can get on the `buy_dex`
    let amount_out_res = dex::get_amount_out(
        buy_dex,
        provider.clone(),
        amount_in,
        buy_token_addr,
        sell_token_addr,
    )
    .await;

    let amount_out_intermediate = match amount_out_res {
        Ok(amount) => amount,
        Err(e) => {
            warn!(
                "Could not get price from {}->{} on {}: {}",
                buy_token_symbol, sell_token_symbol, buy_dex.name, e
            );
            return Ok(());
        }
    };

    if amount_out_intermediate.is_zero() {
        return Ok(());
    }

    // 2. Get how much `buy_token` we get back when selling the `sell_token` on the `sell_dex`
    let final_amount_out_res = dex::get_amount_out(
        sell_dex,
        provider,
        amount_out_intermediate,
        sell_token_addr,
        buy_token_addr,
    )
    .await;

    let final_amount_out = match final_amount_out_res {
        Ok(amount) => amount,
        Err(e) => {
            warn!(
                "Could not get price from {}->{} on {}: {}",
                sell_token_symbol, buy_token_symbol, sell_dex.name, e
            );
            return Ok(());
        }
    };

    // 3. Calculate profit
    let initial_amount_f = ethers::utils::format_units(amount_in, buy_token_decimals.to_string().as_str())?.parse::<f64>()?;
    let final_amount_f = ethers::utils::format_units(final_amount_out, buy_token_decimals.to_string().as_str())?.parse::<f64>()?;

    let gross_profit = final_amount_f - initial_amount_f;
    let net_profit = gross_profit - config.simulated_gas_cost_usd;

    if net_profit > config.min_profit_threshold_usd {
        let intermediate_amount_f = ethers::utils::format_units(amount_out_intermediate, sell_token_decimals.to_string().as_str())?.parse::<f64>()?;

        info!(
            "🚀 Arbitrage Opportunity Found! 🚀\n\
            - Path: {} -> {} -> {}\n\
            - Buy on: {} -> Sell on: {}\n\
            - Amount In: {:.4} {}\n\
            - Intermediate Amount: {:.4} {}\n\
            - Amount Out: {:.4} {}\n\
            - Gross Profit: {:.4} {}\n\
            - Net Profit (after gas): {:.4} {}",
            buy_token_symbol, sell_token_symbol, buy_token_symbol,
            buy_dex.name, sell_dex.name,
            initial_amount_f, buy_token_symbol,
            intermediate_amount_f, sell_token_symbol,
            final_amount_f, buy_token_symbol,
            gross_profit, buy_token_symbol,
            net_profit, buy_token_symbol
        );

        // Log to database
        db::log_opportunity(
            db_pool,
            buy_dex.name,
            sell_dex.name,
            buy_token_symbol,
            sell_token_symbol,
            initial_amount_f,
            final_amount_f,
            net_profit,
        )
        .await?;
    }

    Ok(())
}
