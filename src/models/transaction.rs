use clickhouse::Row;
use serde::{Serialize, Deserialize};
use chrono::NaiveDateTime;

#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionRow {
    pub hash: String,
    pub block_number: u64,
    pub from_addr: String,
    pub to_addr: String,
    pub value: String,
    pub sensivity: Sensivity,
}

#[repr(u8)]
#[derive(Serialize, Deserialize, Debug)]
enum Sensivity {
    Red = 1,
    Yellow = 2,
    Green = 3
}
