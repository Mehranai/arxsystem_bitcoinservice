use clickhouse::Client;
use ethers::prelude::*;
use std::sync::Arc;

pub struct Loader {
    pub clickhouse: Arc<Client>,
    pub eth_provider: Arc<Provider<Http>>,
}

impl Loader {
    pub async fn new(config: &crate::config::AppConfig) -> anyhow::Result<Self> {
        let clickhouse = Arc::new(
            Client::default()
                .with_url(&config.clickhouse_url)
                .with_user(&config.clickhouse_user)
                .with_password(&config.clickhouse_pass)
                .with_database(&config.clickhouse_db),
        );

        let eth_provider = Arc::new(
            Provider::<Http>::try_from(&config.eth_rpc_url)?
        );

        Ok(Self { clickhouse, eth_provider })
    }
}
