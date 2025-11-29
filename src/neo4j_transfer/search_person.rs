pub async fn search_person_graph(session: &Session, pid: u16) -> Vec<Row> {
    let q = query(
        "
        MATCH (p:Owner {person_id: $pid})-[:OWNS]->(w:Wallet)
        OPTIONAL MATCH path = (w)-[:TRANSACTED*1..2]->(other)
        RETURN p, w, other, path
        ",
    )
    .param("pid", pid as i64);

    let mut result = session.execute(q).await.unwrap();
    let mut rows = vec![];

    while let Ok(Some(row)) = result.next().await {
        rows.push(row);
    }

    rows
}