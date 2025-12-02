use tokio;
use anyhow::Result;
use arz_axum_for_bitcoin::config::AppConfig;
use arz_axum_for_bitcoin::tasks::fetch_loop::{run_btc_loop};

// Axum Section
//use arz_axum_for_bitcoin::router::build_router;


#[tokio::main]
async fn main() -> Result<()> {
    
    let config = AppConfig::default();

    let fetch_handle = tokio::spawn({
        let config = config.clone();
        async move { run_btc_loop(config).await.unwrap(); }
    });

    fetch_handle.await.unwrap();
    Ok(())

    // Axum Section Skip
    // let app = router::create(state);
    // axum::Server::bind(&cfg.bind_addr)
    //     .serve(app.into_make_service())
    //     .await
    //     .unwrap();
}
