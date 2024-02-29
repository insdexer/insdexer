pub mod db;

use rocksdb::{DBAccess, DBIteratorWithThreadMode};

pub trait DBBase {
    fn get<K: AsRef<[u8]>>(&self, key: K) -> Result<Option<Vec<u8>>, rocksdb::Error>;
    fn iterator(&self, mode: rocksdb::IteratorMode) -> DBIteratorWithThreadMode<'_, Self>
    where
        Self: Sized + DBAccess;
}

pub trait TxnDB {
    fn get_u64(&self, key: &str) -> u64;
    fn get_string(&self, key: &str) -> Option<String>;
}
