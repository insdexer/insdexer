use rocksdb::TransactionDB;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};

pub const TRANSFER_TX_RAW_LENGTH: usize = 32;
pub const TRANSFER_TX_HEX_LENGTH: usize = 64;

pub const APP_PROTO_MARKET: &'static str = "market";
pub const APP_PROTO_COLLECTION: &'static str = "collection";

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum InscriptionVerifiedStatus {
    #[serde(rename = "0")]
    Unresolved,
    #[serde(rename = "1")]
    Successful,
    #[serde(rename = "2")]
    Failed,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum InscriptionMimeCategory {
    #[serde(rename = "0")]
    Null,
    #[serde(rename = "1")]
    Text,
    #[serde(rename = "2")]
    Image,
    #[serde(rename = "3")]
    Transfer,
    #[serde(rename = "4")]
    Json,
    #[serde(rename = "5")]
    Invoke,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Inscription {
    pub id: u64,
    pub from: String,
    pub to: String,
    pub blocknumber: u64,
    pub tx_hash: String,
    pub tx_index: u64,
    pub mime_category: InscriptionMimeCategory,
    pub mime_type: String,
    pub mime_data: String,
    pub timestamp: u64,
    pub event_logs: Vec<web3::types::Log>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_order_id: Option<String>,

    pub verified: InscriptionVerifiedStatus,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,

    #[serde(skip_serializing, default = "default_value_json")]
    pub json: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InscriptionToken {
    pub insc_id: u64,
    pub tick: String,
    pub tick_i: String,
    pub tx: String,
    pub from: String,
    pub blocknumber: u64,
    pub timestamp: u64,
    pub holders: u64,
    pub mint_max: u64,
    pub mint_limit: u64,
    pub mint_progress: u64,
    pub mint_finished: bool,

    #[serde(skip_serializing, default = "default_value_bool")]
    pub updated: bool,
    #[serde(skip_serializing, default = "default_value_bool")]
    pub deploy: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InscriptionTokenMint {
    pub insc_id: u64,
    pub tick: String,
    pub amount: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InscriptionTokenSnapshot {
    pub tick: String,
    pub address: String,
    pub amount: u64,
    pub blocknumber: u64,
    pub transaction_index: u64,
    pub timestamp: u64,
}

fn default_value_json() -> serde_json::Value {
    serde_json::Value::Null
}

fn default_value_bool() -> bool {
    false
}

pub struct Indexer {
    pub db: Arc<RwLock<TransactionDB>>,
    pub worker_sync: Arc<WorkerSync>,
    pub worker_inscribe: Arc<WorkerInscribe>,
}

pub struct InscribeContext {
    pub db: Arc<RwLock<rocksdb::TransactionDB>>,
    pub inscriptions: Vec<Inscription>,
    pub inscriptions_holder: HashMap<u64, String>,
    pub inscriptions_transfer: Vec<(u64, u64, u64)>,

    pub token_cache: HashMap<String, InscriptionToken>,
    pub token_balance_change: HashMap<String, HashMap<String, i64>>,
}

pub struct WorkerInscribe {
    pub db: Arc<RwLock<TransactionDB>>,
}

pub struct WorkerSyncState {
    pub blocks: HashMap<u64, web3::types::Block<web3::types::Transaction>>,
    pub event_logs: HashMap<u64, HashMap<u64, Vec<web3::types::Log>>>,
    pub worker_count: u64,
}

pub struct WorkerSync {
    pub db: Arc<RwLock<rocksdb::TransactionDB>>,
    pub state: Arc<Mutex<WorkerSyncState>>,
}