use neo4rs::*;

pub async fn insert_wallet(session: &Session, w: &WalletInfo) {
    let mut query = query(
        "
        MERGE (w:Wallet {address: $address})
        SET w.balance = $balance,
            w.nonce = $nonce,
            w.wallet_type = $wallet_type
        ",
    )
    .param("address", &w.address)
    .param("balance", &w.balance)
    .param("nonce", w.nonce as i64)
    .param("wallet_type", &w.wallet_type);

    session.execute(query).await.unwrap();

    if let Some(person_id) = w.person_id {
        let owner_query = query(
            "
            MERGE (o:Owner {person_id: $pid})
            SET o.person_name = $pname,
                o.personal_id = $personal_id
            ",
        )
        .param("pid", person_id)
        .param("pname", w.person_name.clone().unwrap_or_default())
        .param("personal_id", w.personal_id.clone().unwrap_or_default());

        session.execute(owner_query).await.unwrap();


        let owns_query = query(
            "
            MATCH (o:Owner {person_id: $pid}), (w:Wallet {address: $address})
            MERGE (o)-[:OWNS]->(w)
            ",
        )
        .param("pid", person_id)
        .param("address", &w.address);

        session.execute(owns_query).await.unwrap();
    }
}

pub async fn insert_tx(session: &Session, tx: &Tx) {
    let q = query(
        "
        MATCH (f:Wallet {address: $from_addr})
        MATCH (t:Wallet {address: $to_addr})
        MERGE (f)-[tr:TRANSACTED {hash: $hash}]->(t)
        SET tr.block_number = $block_number,
            tr.value = $value,
            tr.sensivity = $sensivity
        ",
    )
    .param("from_addr", &tx.from_addr)
    .param("to_addr", &tx.to_addr)
    .param("hash", &tx.hash)
    .param("block_number", tx.block_number as i64)
    .param("value", &tx.value)
    .param("sensivity", tx.sensivity as i64);

    session.execute(q).await.unwrap();
}
