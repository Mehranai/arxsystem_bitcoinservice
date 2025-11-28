use crate::models::wallet::WalletRow;
use crate::models::owner::OwnerRow;
use crate::models::transaction::TransactionRow;

use clickhouse::Client;
use std::sync::Arc;
use anyhow::Result;

pub async fn save_tx(clickhouse: Arc<Client>,hash: String,block_number: u64, from: String, to:String, value:String, sensivity: u8 ) -> Result<()> {

    let tx_row = TransactionRow {
        hash: hash,
        block_number: block_number,
        from_addr: from,
        to_addr: to,
        value: value,
        sensivity: sensivity,
    };

    let mut insert_tx = clickhouse.insert::<TransactionRow>("transactions").await?;
    insert_tx.write(&tx_row).await?;
    insert_tx.end().await?;

    Ok(())
}

pub async fn save_wallet(clickhouse: Arc<Client>, addr: &String, balance: String, nonce: u64,  wallet_type: String) -> Result<()> {

    if addr.is_empty() { return Ok(()); }

    let wallet = WalletRow {
        address: addr.into(),
        balance: balance,
        nonce: nonce,
        wallet_type: wallet_type,
    };

    let owner = OwnerRow {
        address: addr.into(),
        person_name: "".into(),
        person_id: 0,
        personal_id: 0,
    };

    let mut insert_wallet = clickhouse.insert::<WalletRow>("wallet_info").await?;
    insert_wallet.write(&wallet).await?;
    insert_wallet.end().await?;

    let mut insert_owner = clickhouse.insert::<OwnerRow>("owner_info").await?;
    insert_owner.write(&owner).await?;
    insert_owner.end().await?;

    Ok(())
}
