CREATE DATABASE IF NOT EXISTS pajohesh;

CREATE TABLE IF NOT EXISTS pajohesh.wallet_info (
    address String,
    balance String,
    nonce UInt64,
    type String,
    person_id UInt64,
) ENGINE = ReplacingMergeTree()
ORDER BY address;

CREATE TABLE IF NOT EXISTS pajohesh.transactions (
    hash String,
    block_number UInt64,
    from_addr String,
    to_addr String,
    value String,
    sensivity UInt8
) ENGINE = MergeTree()
ORDER BY block_number;

CREATE TABLE IF NOT EXISTS pajohesh.owner_info (
    address String,
    person_name String,
    person_id UInt64,
    personal_id UInt64
) ENGINE = ReplacingMergeTree()
ORDER BY address;