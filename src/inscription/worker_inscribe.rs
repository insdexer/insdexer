use super::{
    db::InscribeDB,
    types::{InscribeContext, Inscription, WorkerInscribe},
};
use crate::global::sleep_ms;
use log::info;
use rocksdb::TransactionDB;
use std::sync::{Arc, RwLock};

impl WorkerInscribe {
    pub fn new(db: Arc<RwLock<TransactionDB>>) -> Self {
        WorkerInscribe { db }
    }

    pub fn load_block(&self, blocknumber: u64) -> Vec<Inscription> {
        let mut insc_list = Vec::new();
        let db = self.db.read().unwrap();
        let mut insc_id = db.get_top_inscription_id() + 1;
        loop {
            if let Some(insc) = db.get_inscription_by_id(insc_id) {
                if insc.blocknumber != blocknumber {
                    break;
                }
                insc_list.push(insc);
                insc_id += 1;
            } else {
                break;
            }
        }
        insc_list
    }

    pub async fn inscribe(&self) -> bool {
        let insc_id = self.db.read().unwrap().get_top_inscription_id();
        let sync_id = self.db.read().unwrap().get_top_inscription_sync_id();
        let sync_blocknumber = self.db.read().unwrap().get_sync_blocknumber();

        if insc_id >= sync_id {
            info!("[indexer] inscribe: wait for new inscription");
            return false;
        }

        let insc = self.db.read().unwrap().get_inscription_by_id(insc_id + 1).unwrap();
        let current_blocknumber = insc.blocknumber;
        if current_blocknumber >= sync_blocknumber {
            info!("[indexer] inscribe: wait for new block");
            return false;
        }

        let insc_list = self.load_block(current_blocknumber);

        let mut context = InscribeContext::new(self.db.clone());
        context.inscriptions = insc_list;
        context.inscribe();
        context.save();

        return true;
    }

    pub async fn run(&self) {
        loop {
            if !self.inscribe().await {
                sleep_ms(3000).await;
            }
        }
    }
}
