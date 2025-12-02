use crate::services::loader::LoaderBtc;
use crate::models::transaction::Sensivity;
use crate::models::blockstreams::*;
use clickhouse::Client;
use std::sync::Arc;
use anyhow::Result;
use futures::stream::{FuturesUnordered, StreamExt};
use reqwest::Client as HttpClient;
use tokio::sync::{mpsc, Mutex};

// ---------------- Helper Functions ----------------
fn btc_from_sats(sats: u64) -> f64 {
    sats as f64 / 100_000_000.0
}

fn calc_sensivity_btc(value: f64) -> Sensivity {
    if value > 100.0 { Sensivity::Red }
    else if value > 10.0 { Sensivity::Yellow }
    else { Sensivity::Green }
}

// ---------------- API Functions ----------------
async fn get_block_hash_by_height(base_url: &str, height: u64, client: &HttpClient) -> Result<String> {
    let url = format!("{}/block-height/{}", base_url, height);
    let res = client.get(&url).send().await?;
    Ok(res.text().await?.trim().to_string())
}

async fn get_block_txs(base_url: &str, block_hash: &str, client: &HttpClient) -> Result<Vec<BlockTx>> {
    let url = format!("{}/block/{}/txs", base_url, block_hash);
    let res = client.get(&url).send().await?;
    let body_text = res.text().await?;
    Ok(serde_json::from_str(&body_text)?)
}

// ---------------- Batch Struct ----------------
struct TxBatch {
    rows: Vec<(String, u64, String, String, String, u8)>,
}

impl TxBatch {
    fn new() -> Self { Self { rows: Vec::with_capacity(500) } }

    fn push(&mut self, txid: String, block: u64, from: String, to: String, value: String, sens: u8) {
        self.rows.push((txid, block, from, to, value, sens));
    }

    fn is_full(&self) -> bool { self.rows.len() >= 500 }

    fn clear(&mut self) { self.rows.clear(); }
}

// ---------------- Async Batch Insert ----------------
async fn flush_batch(client: &Client, batch: &TxBatch) -> Result<()> {
    if batch.rows.is_empty() {
        return Ok(());
    }

    // مشخص کردن نوع ردیف‌ها برای Rust
    let mut insert = client
        .insert::<(String, u64, String, String, String, u8)>("transactions")
        .await?;

    for (hash, block, from, to, value, sens) in batch.rows.iter() {
        insert.write(&(hash.clone(), *block, from.clone(), to.clone(), value.clone(), *sens)).await?;
    }

    insert.end().await?;
    Ok(())
}

// ---------------- Worker Pool ----------------
const WORKERS: usize = 12;

async fn start_workers(rx: Arc<Mutex<mpsc::Receiver<(BlockTx, u64)>>>, clickhouse: Arc<Client>) -> Vec<tokio::task::JoinHandle<()>> {
    let mut handles = Vec::new();

    for id in 0..WORKERS {
        let rx = rx.clone();
        let ch = clickhouse.clone();

        let handle = tokio::spawn(async move {
            let mut batch = TxBatch::new();

            loop {
                let msg_opt = {
                    let mut rx_locked = rx.lock().await;
                    rx_locked.recv().await
                };

                match msg_opt {
                    Some((tx, block)) => {
                        let from_addr = tx.vin.iter()
                            .filter_map(|v| v.prevout.as_ref()?.scriptpubkey_address.clone())
                            .next().unwrap_or_default();

                        let to_addr = tx.vout.iter()
                            .filter_map(|v| v.scriptpubkey_address.clone())
                            .next().unwrap_or_default();

                        let total_value_sats: u64 = tx.vout.iter().map(|v| v.value).sum();
                        let total_value = btc_from_sats(total_value_sats);

                        batch.push(
                            tx.txid,
                            block,
                            from_addr,
                            to_addr,
                            total_value.to_string(),
                            calc_sensivity_btc(total_value) as u8
                        );

                        if batch.is_full() {
                            if let Err(e) = flush_batch(&ch, &batch).await {
                                eprintln!("Worker {id} flush error: {:?}", e);
                            }
                            batch.clear();
                        }
                    },
                    None => {
                        if let Err(e) = flush_batch(&ch, &batch).await {
                            eprintln!("Worker {id} final flush error: {:?}", e);
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

// ---------------- Main Fetch Function ----------------
pub async fn fetch_btc(
    loader: Arc<LoaderBtc>,
    start_block: u64,
    total_txs: u64,
    base_url: &str
) -> Result<()> {

    let (tx, rx) = mpsc::channel::<(BlockTx, u64)>(20_000);
    let rx = Arc::new(Mutex::new(rx)); // shared Receiver بین workerها
    let clickhouse = loader.clickhouse.clone();
    let client = Arc::new(HttpClient::new());

    let handles = start_workers(rx, clickhouse).await;

    let max_blocks_parallel = 20;
    let sem = Arc::new(tokio::sync::Semaphore::new(max_blocks_parallel));

    let mut block_tasks = FuturesUnordered::new();
    let mut processed = 0u64;

    for height in start_block..start_block + 10_000 {
        if processed >= total_txs { break; }

        let tx_clone = tx.clone();
        let base = base_url.to_string();
        let client = client.clone();
        let sem = sem.clone();

        block_tasks.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();

            let hash = get_block_hash_by_height(&base, height, &client).await?;
            let txs = get_block_txs(&base, &hash, &client).await?;

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

    drop(tx); // close channel

    for h in handles {
        let _ = h.await;
    }

    println!(" ✅ FINISHED — total txs: {}", processed);
    Ok(())
}
