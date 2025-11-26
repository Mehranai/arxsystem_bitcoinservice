use std::sync::Arc;
use crate::services::{loader::Loader, bitcoin, ethereum};
use crate::config::AppConfig;
use anyhow::Result;

pub async fn run_btc_loop(config: AppConfig) -> Result<()> {
    let loader = Arc::new(Loader::new(&config).await?);

    // Fetch BTC
    bitcoin::fetch_btc(
        loader.clone(),
        config.btc_start_block,
        config.total_btc_txs,
        &config.btc_api_url,
    ).await?;

    Ok(())
}

pub async fn run_eth_loop(config: AppConfig) -> Result<()> {
    let loader = Arc::new(Loader::new(&config).await?);
        // Fetch ETH
    ethereum::fetch_eth(
        loader.clone(),
        config.eth_start_block,
        config.total_eth_txs,
    ).await?;
    Ok(())
}
