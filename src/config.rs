use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub clickhouse_url: String,
    pub clickhouse_user: String,
    pub clickhouse_pass: String,
    pub clickhouse_db_eth: String,
    pub clickhouse_db_btc: String,
    pub eth_rpc_url: String,
    pub btc_api_url: String,
    pub btc_start_block: u64,
    pub eth_start_block: u64,
    pub total_btc_txs: u64,
    pub total_eth_txs: u64,
}

impl AppConfig {
    pub fn default() -> Self {
        Self {
            clickhouse_url: "http://localhost:8123".into(),
            clickhouse_user: "mehran".into(),
            clickhouse_pass: "mehran.crypto9".into(),
            clickhouse_db_eth: "eth_database".into(),
            clickhouse_db_btc: "btc_database".into(),
            eth_rpc_url: "https://rpc.ankr.com/eth/<Klid>".into(),
            btc_api_url: "https://blockstream.info/api".into(),
            btc_start_block: 831000,
            eth_start_block: 19000000,
            total_btc_txs: 500,
            total_eth_txs: 300,
        }
    }
}
