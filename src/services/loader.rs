use clickhouse::Client;
use ethers::prelude::*;
use std::sync::Arc;

pub struct LoaderEth {
    pub clickhouse: Arc<Client>,
    pub eth_provider: Arc<Provider<Http>>,
}

impl LoaderEth{
    pub async fn new(config: &crate::config::AppConfig) -> anyhow::Result<Self> {
        let clickhouse = Arc::new(
            Client::default()
                .with_url(&config.clickhouse_url)
                .with_user(&config.clickhouse_user)
                .with_password(&config.clickhouse_pass)
                .with_database(&config.clickhouse_db_eth),
        );

        let eth_provider = Arc::new(
            Provider::<Http>::try_from(&config.eth_rpc_url)?
        );

        Ok(Self { clickhouse, eth_provider })
    }
}

pub struct LoaderBtc {
     pub clickhouse: Arc<Client>
}

impl LoaderBtc {
    pub async fn new(config: &crate::config::AppConfig) -> anyhow::Result<Self> {
        let clickhouse = Arc::new(
                Client::default()
                    .with_url(&config.clickhouse_url)
                    .with_user(&config.clickhouse_user)
                    .with_password(&config.clickhouse_pass)
                    .with_database(&config.clickhouse_db_btc),
            );

            Ok(Self { clickhouse })
    }
}
