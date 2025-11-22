use std::net::SocketAddr;

#[derive(Clone)]
pub struct Config {
    pub bind_addr: SocketAddr,
    pub bitcoin_rpc_url: String,
    pub bitcoin_rpc_user: String,
    pub bitcoin_rpc_pass: String,
    pub clickhouse_url: String,
}
    pub fn load() -> Config{

        // should sync this with Env!!!  and remove it from here
        config{
            bind_addr: "0.0.0.0:3000".parse().unwrap(),

            bitcoin_rpc_url: "http://127.0.0.1:8332".into(),
            bitcoin_rpc_user: "user".into(),
            bitcoin_rpc_pass: "pass".into(),

            clickhouse_url: "http://localhost:8123".into(),
        }
    }