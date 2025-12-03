use crate::models::wallet::WalletRow;
use crate::models::owner::OwnerRow;
use crate::models::transaction::TransactionRow;

use clickhouse::Client;
use std::sync::Arc;
use anyhow::Result;
use nanoid::nanoid;

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

    let person_id = generate_person_id();

    let wallet = WalletRow {
        address: addr.into(),
        balance: balance,
        nonce: nonce,
        wallet_type: wallet_type,
        person_id: person_id

    };

    let owner = OwnerRow {
        address: addr.into(),
        person_name: "".into(),
        person_id: person_id,
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

pub fn generate_person_id() -> u16{

    let alphabet: [char; 10] = [
        '1', '2', '3', '4', '5', '6', '7', '8', '9', '0'
    ];
    nanoid!(10, &alphabet).parse().expect("Cannot Cast String to U16 (PersonId)")
}
