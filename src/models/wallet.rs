use clickhouse::Row;
use serde::{Serialize, Deserialize};
use chrono::NaiveDateTime;

#[derive(Serialize, Deserialize, Debug)]
pub struct WalletRow {
    pub address: String,
    pub balance: usize,
    pub nonce: u64,
    #[serde(rename = "type")]
    pub wallet_type: String,
}

enum WalletType {
    Custodial = 1,
    NonCustodail = 2,
}