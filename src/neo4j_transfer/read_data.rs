// Read data from clickhouse

use clickhouse::{Client, Row};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct WalletInfo {
    address: String,
    balance: String,
    nonce: u64,
    wallet_type: String,
    person_name: Option<String>,
    person_id: Option<String>,
    personal_id: Option<u16>,
}

#[derive(Debug, Deserialize)]
struct Tx {
    hash: String,
    block_number: u64,
    from_addr: String,
    to_addr: String,
    value: String,
    sensivity: u8,
}

async fn fetch_wallets(client: &Client) -> Vec<WalletInfo> {
    client
        .query("
            SELECT w.address, w.balance, w.nonce, w.wallet_type,
                   o.person_name, o.person_id, o.personal_id
            FROM btc_database.wallet_info w
            LEFT JOIN btc_database.owner_info o ON w.address = o.address
        ")
        .fetch::<WalletInfo>()
        .await
        .unwrap()
}

async fn fetch_txs(client: &Client) -> Vec<Tx> {
    client
        .query("
            SELECT hash, block_number, from_addr, to_addr, value, sensivity
            FROM btc_database.transactions
        ")
        .fetch::<Tx>()
        .await
        .unwrap()
}
