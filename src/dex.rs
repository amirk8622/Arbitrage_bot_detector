use ethers::prelude::*;
use std::sync::Arc;

// Generate type-safe bindings to the smart contracts.
// This is done via the `abigen!` macro and the ABI JSON files.
abigen!(
    IUniswapV2Router,
    "./src/abis/uniswap_v2_router.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

abigen!(
    IUniswapV3Quoter,
    "./src/abis/uniswap_v3_quoter.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

/// A representation of a DEX with its name and router address.
#[derive(Debug, Clone, Copy)]
pub struct Dex<'a> {
    pub name: &'a str,
    pub router_address: Address,
    pub dex_type: DexType,
}

#[derive(Debug, Clone, Copy)]
pub enum DexType {
    UniswapV2,
    UniswapV3 { quoter_address: Address, fee: u32 },
}

/// Fetches the output amount for a given input amount and token pair from a DEX.
pub async fn get_amount_out(
    dex: &Dex<'_>,
    provider: Arc<Provider<Http>>,
    amount_in: U256,
    token_in: Address,
    token_out: Address,
) -> eyre::Result<U256> {
    match dex.dex_type {
        DexType::UniswapV2 => {
            let router = IUniswapV2Router::new(dex.router_address, provider);
            let amounts_out = router
                .get_amounts_out(amount_in, vec![token_in, token_out])
                .call()
                .await?;
            Ok(amounts_out[1])
        }
        DexType::UniswapV3 { quoter_address, fee } => {
            let quoter = IUniswapV3Quoter::new(quoter_address, provider);
            let (amount_out, _, _, _) = quoter
                .quote_exact_input_single(token_in, token_out, fee, amount_in, U256::zero())
                .call()
                .await?;
            Ok(amount_out)
        }
    }
}
