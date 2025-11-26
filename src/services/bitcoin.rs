use crate::services::loader::Loader;
use crate::models::transaction::{TransactionRow, Sensivity};
use crate::models::wallet::WalletRow;
use crate::models::owner::OwnerRow;
use crate::models::blockstreams::*;

use clickhouse::Client;
use std::sync::Arc;
use anyhow::Result;
use futures::stream::{FuturesUnordered, StreamExt};
use reqwest::Client as HttpClient;

fn btc_from_sats(sats: u64) -> f64 { sats as f64 / 100_000_000.0 }

pub async fn get_wallet_balance(base_url: &str, address: &str) -> Result<f64> {
    let url = format!("{}/address/{}/utxo", base_url, address);
    let client = HttpClient::new();
    let resp = client.get(&url).send().await?;
    let body = resp.text().await?;
    let utxos: Vec<UTXO> = serde_json::from_str(&body)?;
    Ok(btc_from_sats(utxos.iter().map(|u| u.value).sum()))
}

fn calc_sensivity_btc(value: f64) -> Sensivity {
    if value > 100.0 { Sensivity::Red }
    else if value > 10.0 { Sensivity::Yellow }
    else { Sensivity::Green }
}

pub async fn fetch_btc(loader: Arc<Loader>, start_block: u64, total_txs: u64, base_url: &str) -> Result<()> {
    let clickhouse = loader.clickhouse.clone();
    let mut tx_count = 0;

    for block_height in start_block..start_block+1000 {
        if tx_count >= total_txs { break; }
        let block_hash = get_block_hash_by_height(base_url, block_height).await?;
        let txs = get_block_txs(base_url, &block_hash).await?;
        let mut tasks = FuturesUnordered::new();

        for tx in txs {
            if tx_count >= total_txs { break; }
            let clickhouse = clickhouse.clone();
            let base_url = base_url.to_string();
            tasks.push(tokio::spawn(async move {
                process_tx(clickhouse, tx, block_height, &base_url).await?;
                Ok::<(), anyhow::Error>(())
            }));
            tx_count += 1;
        }

        while let Some(res) = tasks.next().await {
            res??;
        }
    }
    Ok(())
}

async fn process_tx(clickhouse: Arc<Client>, tx: BlockTx, block_number: u64, base_url: &str) -> Result<()> {
    let from_addr = tx.vin.iter().filter_map(|v| v.prevout.as_ref()?.scriptpubkey_address.clone()).next().unwrap_or_default();
    let to_addr = tx.vout.iter().filter_map(|v| v.scriptpubkey_address.clone()).next().unwrap_or_default();
    let total_value_sats: u64 = tx.vout.iter().map(|v| v.value).sum();
    let total_value = btc_from_sats(total_value_sats);

    let tx_row = TransactionRow {
        hash: tx.txid.clone(),
        block_number,
        from_addr: from_addr.clone(),
        to_addr: to_addr.clone(),
        value: total_value.to_string(),
        sensivity: calc_sensivity_btc(total_value) as u8,
    };

    let mut insert_tx = clickhouse.insert::<TransactionRow>("transactions").await?;
    insert_tx.write(&tx_row).await?;
    insert_tx.end().await?;

    save_wallet(clickhouse.clone(), from_addr, total_value).await?;
    save_wallet(clickhouse.clone(), to_addr, total_value).await?;
    Ok(())
}

async fn save_wallet(clickhouse: Arc<Client>, address: String, balance: f64) -> Result<()> {
    if address.is_empty() { return Ok(()); }

    let wallet = WalletRow {
        address: address.clone(),
        balance: balance.to_string(),
        nonce: 0,
        wallet_type: "wallet".into(),
    };

    let owner = OwnerRow {
        address: address.clone(),
        person_name: "".into(),
        person_id: 0,
        personal_id: 0,
    };

    let mut insert_wallet = clickhouse.insert::<WalletRow>("wallet_info").await?;
    insert_wallet.write(&wallet).await?;
    insert_wallet.end().await?;

    let mut insert_owner = clickhouse.insert::<OwnerRow>("owner_info").await?;
    insert_owner.write(&owner).await?;
    insert_owner.end().await?;

    Ok(())
}

// API Helper
async fn get_block_hash_by_height(base_url: &str, height: u64) -> Result<String> {
    let url = format!("{}/block-height/{}", base_url, height);
    Ok(reqwest::get(&url).await?.text().await?.trim().to_string())
}

async fn get_block_txs(base_url: &str, block_hash: &str) -> Result<Vec<BlockTx>> {
    let mut all_txs = Vec::new();
    let mut start = 0;

    loop {
        let url = format!("{}/block/{}/txs/{}", base_url, block_hash, start);
        let resp = reqwest::get(&url).await?;
        if !resp.status().is_success() { break; }
        let body_text = resp.text().await?;
        let txs_page: Vec<BlockTx> = serde_json::from_str(&body_text)?;
        if txs_page.is_empty() { break; }
        let page_len = txs_page.len();
        all_txs.extend(txs_page.into_iter());
        if page_len < 25 { break; }
        start += 25;
    }

    Ok(all_txs)
}
