use clickhouse::Row;
use serde::{Serialize, Deserialize};
use chrono::NaiveDateTime;



//fetch data from blockchain
#[derive(Serialize)]
struct NewOwner {
    address: String,
    person_name: String,
    personal_id: u16
}

//Update owner Row with database upgrade
#[derive(Row, Serialize, Deserialize)]
pub struct OwnerRow {
    address: String,
    person_name: String,
    person_id: u16,
    personal_id: u16
}