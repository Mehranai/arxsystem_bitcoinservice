CREATE DATABASE IF NOT EXISTS btc_database;

CREATE TABLE IF NOT EXISTS btc_database.wallet_info (
    address String,
    balance String,
    nonce UInt64,
    type String,
    person_id String,
) ENGINE = ReplacingMergeTree()
ORDER BY address;

CREATE TABLE IF NOT EXISTS btc_database.transactions (
    hash String,
    block_number UInt64,
    from_addr String,
    to_addr String,
    value String,
    sensivity UInt8
) ENGINE = MergeTree()
ORDER BY block_number;

CREATE TABLE IF NOT EXISTS btc_database.owner_info (
    address String,
    person_name String,
    person_id String,
    personal_id UInt16
) ENGINE = ReplacingMergeTree()
ORDER BY address;