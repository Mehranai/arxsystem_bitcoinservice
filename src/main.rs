mod router;
mod config;
mod state;
mod models;

#[tokio::main]
async fn main() {
    let cfg = config::load();
    let state = state::init(&cfg).await;

    let app = router::create(state);

    axum::Server::bind(&cfg.bind_addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
