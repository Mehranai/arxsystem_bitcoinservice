use crate::services::loader::LoaderBtc;
use crate::models::transaction::{Sensivity, TransactionRow};
use crate::models::blockstreams::*;
use crate::services::progress::{save_tx, save_wallet};

use clickhouse::Client;
use std::sync::Arc;
use anyhow::Result;
use futures::stream::{FuturesUnordered, StreamExt};
use reqwest::Client as HttpClient;
use tokio::sync::{mpsc, Semaphore, Mutex};

fn btc_from_sats(sats: u64) -> f64 {
    sats as f64 / 100_000_000.0
}

fn calc_sensivity_btc(value: f64) -> Sensivity {
    if value > 100.0 { Sensivity::Red }
    else if value > 10.0 { Sensivity::Yellow }
    else { Sensivity::Green }
}

// Batch برای worker 
struct TxBatch {
    rows: Vec<BlockTx>,
    block_numbers: Vec<u64>,
    capacity: usize,
}

impl TxBatch {
    fn new(capacity: usize) -> Self {
        Self { rows: Vec::with_capacity(capacity), block_numbers: Vec::with_capacity(capacity), capacity }
    }

    fn push(&mut self, tx: BlockTx, block: u64) {
        self.rows.push(tx);
        self.block_numbers.push(block);
    }

    fn is_full(&self) -> bool {
        self.rows.len() >= self.capacity
    }

    fn clear(&mut self) {
        self.rows.clear();
        self.block_numbers.clear();
    }

    fn len(&self) -> usize { self.rows.len() }
}

// Workers
const WORKERS: usize = 4;

async fn start_workers(
    rx: Arc<Mutex<mpsc::Receiver<(BlockTx, u64)>>>,
    clickhouse: Arc<Client>,
    batch_size: usize,
) -> Vec<tokio::task::JoinHandle<()>> {
    let mut handles = Vec::new();

    for id in 0..WORKERS {
        let rx = rx.clone();
        let ch = clickhouse.clone();

        let handle = tokio::spawn(async move {
            let mut batch = TxBatch::new(batch_size);

            loop {
                let msg_opt = {
                    let mut rx_locked = rx.lock().await;
                    rx_locked.recv().await
                };

                match msg_opt {
                    Some((tx, block)) => {
                        batch.push(tx, block);

                        if batch.is_full() {
                            if let Err(e) = flush_batch_with_save(&ch, &mut batch).await {
                                eprintln!("Worker {id} flush error: {:?}", e);
                            }
                            batch.clear();
                        }
                    },
                    None => {
                        if batch.len() > 0 {
                            if let Err(e) = flush_batch_with_save(&ch, &mut batch).await {
                                eprintln!("Worker {id} final flush error: {:?}", e);
                            }
                        }
                        println!("Worker {id} finished.");
                        break;
                    }
                }
            }
        });

        handles.push(handle);
    }

    handles
}

// Flush batch + save_tx/save_wallet 
async fn flush_batch_with_save(clickhouse: &Arc<Client>, batch: &mut TxBatch) -> Result<()> {
    for (i, tx) in batch.rows.iter().enumerate() {
        let block_number = batch.block_numbers[i];
        let from_addr = tx.vin.iter().filter_map(|v| v.prevout.as_ref()?.scriptpubkey_address.clone()).next().unwrap_or_default();
        let to_addr = tx.vout.iter().filter_map(|v| v.scriptpubkey_address.clone()).next().unwrap_or_default();
        let total_value_sats: u64 = tx.vout.iter().map(|v| v.value).sum();
        let total_value = btc_from_sats(total_value_sats);

        save_tx(
            clickhouse.clone(),
            tx.txid.clone(),
            block_number,
            from_addr.clone(),
            to_addr.clone(),
            total_value.to_string(),
            calc_sensivity_btc(total_value) as u8
        ).await?;

        save_wallet(clickhouse.clone(), &from_addr, total_value.to_string(), 0, "".to_string()).await?;

        save_wallet(clickhouse.clone(), &to_addr, total_value.to_string(), 0, "".to_string()).await?;
    }

    Ok(())
}

// Fetch BTC 
pub async fn fetch_btc(
    loader: Arc<LoaderBtc>,
    start_block: u64,
    total_txs: u64,
    base_url: &str
) -> Result<()> {
    let (tx, rx) = mpsc::channel::<(BlockTx, u64)>(20_000);
    let rx = Arc::new(Mutex::new(rx));
    let clickhouse = loader.clickhouse.clone();
    let client = Arc::new(HttpClient::new());

    let handles = start_workers(rx.clone(), clickhouse.clone(), 500).await;

    let max_blocks_parallel = 10;
    let sem = Arc::new(Semaphore::new(max_blocks_parallel));
    let mut block_tasks = FuturesUnordered::new();
    let mut processed = 0u64;

    for height in start_block..start_block + 10_000 {
        if processed >= total_txs { break; }

        let tx_clone = tx.clone();
        let client = client.clone();
        let base_url = base_url.to_string();
        let sem = sem.clone();

        block_tasks.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();

            let hash = get_block_hash_by_height(&base_url, height, &client).await?;
            let txs = get_block_txs(&base_url, &hash, &client).await?;

            let tx_count = txs.len(); 

            for t in txs {
                tx_clone.send((t, height)).await.unwrap();
            }

            Ok::<usize, anyhow::Error>(tx_count)
        }));
    }

    while let Some(res) = block_tasks.next().await {
        processed += res?? as u64;
        println!("Processed: {}", processed);
        if processed >= total_txs { break; }
    }

    drop(tx); // بستن channel
    for h in handles {
        let _ = h.await;
    }

    println!("FINISHED — total txs: {}", processed);
    Ok(())
}

// API Helpers 
async fn get_block_hash_by_height(base_url: &str, height: u64, client: &HttpClient) -> Result<String> {
    let url = format!("{}/block-height/{}", base_url, height);
    let resp = client.get(&url).send().await?;
    Ok(resp.text().await?.trim().to_string())
}

async fn get_block_txs(base_url: &str, block_hash: &str, client: &HttpClient) -> Result<Vec<BlockTx>> {
    let url = format!("{}/block/{}/txs", base_url, block_hash);
    let resp = client.get(&url).send().await?;
    let body_text = resp.text().await?;
    let txs: Vec<BlockTx> = serde_json::from_str(&body_text)?;
    Ok(txs)
}
