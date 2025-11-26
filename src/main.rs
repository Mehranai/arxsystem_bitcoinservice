use anyhow::Error;

use arz_axum_for_bitcoin::{
    models::{OwnerRow, owner},
    services::bitcoin
};
use serde::Serialize;
;

#[tokio::main]
async fn main() -> Result<(), Error>{



    let _output_btc = bitcoin::get_sound().await;
    Ok(())

//     let cfg = config::load();
//     let state = state::init(&cfg).await;

//     let app = router::create(state);

//     axum::Server::bind(&cfg.bind_addr)
//         .serve(app.into_make_service())
//         .await
//         .unwrap();
}
