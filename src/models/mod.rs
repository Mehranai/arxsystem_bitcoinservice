// src/models/mod.rs
pub mod wallet;
pub mod transaction;
pub mod owner;
pub mod blockstreams;

// Structs for ClickHouse
pub use wallet::WalletRow;
pub use transaction::TransactionRow;
pub use owner::OwnerRow;





