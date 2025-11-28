use crate::services::loader::LoaderEth;
use crate::models::transaction::Sensivity;
use crate::services::progress::{save_tx, save_wallet};

use ethers::prelude::*;
use clickhouse::Client;
use std::sync::Arc;
use anyhow::Result;
use reqwest::Client as HttpClient;
use futures::stream::{FuturesUnordered, StreamExt};
use serde::Deserialize;


pub async fn fetch_eth(loader: Arc<LoaderEth>, start_block: u64, total_txs: u64) -> Result<()> {
    let provider = loader.eth_provider.clone();
    let clickhouse = loader.clickhouse.clone();
    let mut tx_count = 0;

    for block_number in start_block..start_block+1000 {
        if tx_count >= total_txs { break; }
        if let Some(block) = provider.get_block_with_txs(block_number).await? {
            let mut tasks = FuturesUnordered::new();
            for tx in block.transactions {
                if tx_count >= total_txs { break; }
                let clickhouse = clickhouse.clone();
                let provider = provider.clone();
                tasks.push(tokio::spawn(async move {
                    process_tx(provider, clickhouse, tx, block_number).await?;
                    Ok::<(), anyhow::Error>(())
                }));
                tx_count += 1;
            }
            while let Some(res) = tasks.next().await { res??; }
        }
    }

    Ok(())
}

async fn process_tx(provider: Arc<Provider<Http>>, clickhouse: Arc<Client>, tx: Transaction, block_number: u64) -> Result<()> {

    let from = tx.from.to_string();
    let to = tx.to.unwrap_or_default().to_string();
    let value_str = tx.value.to_string();
    let sensivity = Sensivity::Green as u8;
    let hash = format!("{:#x}", tx.hash);

    save_tx(clickhouse.clone(), hash, block_number, from, to, value_str, sensivity).await?;
    save_wallet_eth(provider.clone(), clickhouse.clone(), tx.from).await?;
    save_wallet_eth(provider, clickhouse, tx.to.unwrap_or_default()).await?;
    Ok(())
}

async fn save_wallet_eth(provider: Arc<Provider<Http>>,clickhouse: Arc<Client>, addr: Address) -> Result<()> {

    let balance = provider.get_balance(addr, None).await?;
    let nonce = provider.get_transaction_count(addr, None).await?;
    let wallet_type = detect_wallet_type_from_etherscan(addr).await?;

    save_wallet(clickhouse.clone(), &addr.to_string(), balance.to_string(), nonce.as_u64(), wallet_type).await?;
    Ok(())
}

#[derive(Deserialize)]
struct EtherscanAbiResult {
    status: String,
    message: String,
}

pub async fn detect_wallet_type_from_etherscan(address: Address) -> anyhow::Result<String> {
    let api_key = "DWYGKM65G8A7HHE4J497BWF9TK3R4H9NGC";
    let url = format!(
        "https://api.etherscan.io/v2/api?chain=eth&chainid=1&module=contract&action=getabi&address={:?}&apikey={}",
        address, api_key
    );

    let client = HttpClient::new();
    let resp = client.get(&url).send().await?;
    let resp_text = resp.text().await?;
    let body: EtherscanAbiResult = serde_json::from_str(&resp_text)?;

    if body.status == "1" && body.message == "OK" {
        Ok("smart_contract".to_string())
    } else {
        Ok("wallet".to_string())
    }
}
