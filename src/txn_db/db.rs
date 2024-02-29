use super::{DBBase, TxnDB};
use rocksdb::{DBIteratorWithThreadMode, IteratorMode, TransactionDB, DB};

impl DBBase for TransactionDB {
    fn get<K: AsRef<[u8]>>(&self, key: K) -> Result<Option<Vec<u8>>, rocksdb::Error> {
        TransactionDB::get(self, key)
    }

    fn iterator(&self, mode: IteratorMode) -> DBIteratorWithThreadMode<'_, Self> {
        TransactionDB::iterator(self, mode)
    }
}

impl DBBase for DB {
    fn get<K: AsRef<[u8]>>(&self, key: K) -> Result<Option<Vec<u8>>, rocksdb::Error> {
        DB::get(self, key)
    }

    fn iterator(&self, mode: IteratorMode) -> DBIteratorWithThreadMode<'_, Self> {
        DB::iterator(self, mode)
    }
}

impl<T: DBBase> TxnDB for T {
    fn get_u64(&self, key: &str) -> u64 {
        let value = self.get(key.as_bytes()).unwrap();
        if value.is_none() {
            return 0;
        }

        let bytes = value.unwrap();
        let ptr = bytes.as_ptr() as *const u64;
        unsafe { u64::from_be(*ptr) }
    }

    fn get_string(&self, key: &str) -> Option<String> {
        match self.get(key.as_bytes()).unwrap() {
            Some(bytes) => Some(String::from_utf8(bytes).unwrap()),
            None => None,
        }
    }
}
