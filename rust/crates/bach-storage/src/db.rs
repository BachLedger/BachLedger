//! RocksDB wrapper

use crate::error::{StorageError, StorageResult};
use parking_lot::RwLock;
use rocksdb::{BoundColumnFamily, ColumnFamilyDescriptor, DBWithThreadMode, MultiThreaded, Options, WriteBatch};
use std::path::Path;
use std::sync::Arc;

/// Column family names
pub mod cf {
    /// Account state
    pub const ACCOUNTS: &str = "accounts";
    /// Contract storage
    pub const STORAGE: &str = "storage";
    /// Contract code
    pub const CODE: &str = "code";
    /// Block headers
    pub const HEADERS: &str = "headers";
    /// Block bodies
    pub const BODIES: &str = "bodies";
    /// Transaction receipts
    pub const RECEIPTS: &str = "receipts";
    /// Block number to hash mapping
    pub const BLOCK_INDEX: &str = "block_index";
    /// Metadata
    pub const META: &str = "meta";
}

/// All column family names
pub const ALL_CFS: &[&str] = &[
    cf::ACCOUNTS,
    cf::STORAGE,
    cf::CODE,
    cf::HEADERS,
    cf::BODIES,
    cf::RECEIPTS,
    cf::BLOCK_INDEX,
    cf::META,
];

type RocksDB = DBWithThreadMode<MultiThreaded>;

/// Database configuration
#[derive(Clone, Debug)]
pub struct DbConfig {
    /// Create database if missing
    pub create_if_missing: bool,
    /// Maximum number of open files
    pub max_open_files: i32,
    /// Write buffer size
    pub write_buffer_size: usize,
    /// Maximum write buffers
    pub max_write_buffer_number: i32,
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            create_if_missing: true,
            max_open_files: 512,
            write_buffer_size: 64 * 1024 * 1024, // 64MB
            max_write_buffer_number: 3,
        }
    }
}

/// RocksDB wrapper with column family support
pub struct Database {
    db: Arc<RwLock<Option<RocksDB>>>,
    path: String,
}

impl Database {
    /// Create a new database instance (not yet opened)
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            db: Arc::new(RwLock::new(None)),
            path: path.as_ref().to_string_lossy().to_string(),
        }
    }

    /// Open the database with default config
    pub fn open(&self) -> StorageResult<()> {
        self.open_with_config(DbConfig::default())
    }

    /// Open the database with custom config
    pub fn open_with_config(&self, config: DbConfig) -> StorageResult<()> {
        let mut db_guard = self.db.write();
        if db_guard.is_some() {
            return Err(StorageError::AlreadyOpen);
        }

        let mut opts = Options::default();
        opts.create_if_missing(config.create_if_missing);
        opts.create_missing_column_families(true);
        opts.set_max_open_files(config.max_open_files);
        opts.set_write_buffer_size(config.write_buffer_size);
        opts.set_max_write_buffer_number(config.max_write_buffer_number);

        let cf_descriptors: Vec<ColumnFamilyDescriptor> = ALL_CFS
            .iter()
            .map(|name| ColumnFamilyDescriptor::new(*name, Options::default()))
            .collect();

        let db = RocksDB::open_cf_descriptors(&opts, &self.path, cf_descriptors)?;
        *db_guard = Some(db);
        Ok(())
    }

    /// Close the database
    pub fn close(&self) {
        let mut db_guard = self.db.write();
        *db_guard = None;
    }

    /// Check if database is open
    pub fn is_open(&self) -> bool {
        self.db.read().is_some()
    }

    /// Get a value from a column family
    pub fn get(&self, cf_name: &str, key: &[u8]) -> StorageResult<Option<Vec<u8>>> {
        let db_guard = self.db.read();
        let db = db_guard.as_ref().ok_or(StorageError::NotOpen)?;
        let cf = self.get_cf(db, cf_name)?;
        Ok(db.get_cf(&cf, key)?)
    }

    /// Put a value to a column family
    pub fn put(&self, cf_name: &str, key: &[u8], value: &[u8]) -> StorageResult<()> {
        let db_guard = self.db.read();
        let db = db_guard.as_ref().ok_or(StorageError::NotOpen)?;
        let cf = self.get_cf(db, cf_name)?;
        db.put_cf(&cf, key, value)?;
        Ok(())
    }

    /// Delete a value from a column family
    pub fn delete(&self, cf_name: &str, key: &[u8]) -> StorageResult<()> {
        let db_guard = self.db.read();
        let db = db_guard.as_ref().ok_or(StorageError::NotOpen)?;
        let cf = self.get_cf(db, cf_name)?;
        db.delete_cf(&cf, key)?;
        Ok(())
    }

    /// Create a write batch
    pub fn batch(&self) -> WriteBatchWrapper {
        WriteBatchWrapper::new()
    }

    /// Execute a write batch
    pub fn write_batch(&self, batch: WriteBatchWrapper) -> StorageResult<()> {
        let db_guard = self.db.read();
        let db = db_guard.as_ref().ok_or(StorageError::NotOpen)?;

        // Apply all operations from the batch
        let mut rocks_batch = WriteBatch::default();
        for op in batch.operations {
            match op {
                BatchOp::Put { cf_name, key, value } => {
                    let cf = self.get_cf(db, &cf_name)?;
                    rocks_batch.put_cf(&cf, &key, &value);
                }
                BatchOp::Delete { cf_name, key } => {
                    let cf = self.get_cf(db, &cf_name)?;
                    rocks_batch.delete_cf(&cf, &key);
                }
            }
        }

        db.write(rocks_batch)?;
        Ok(())
    }

    /// Get column family handle
    fn get_cf<'a>(&self, db: &'a RocksDB, name: &str) -> StorageResult<Arc<BoundColumnFamily<'a>>> {
        db.cf_handle(name)
            .ok_or_else(|| StorageError::InvalidColumnFamily(name.to_string()))
    }

    /// Get database path
    pub fn path(&self) -> &str {
        &self.path
    }
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Self {
            db: Arc::clone(&self.db),
            path: self.path.clone(),
        }
    }
}

/// Batch operation
enum BatchOp {
    Put { cf_name: String, key: Vec<u8>, value: Vec<u8> },
    Delete { cf_name: String, key: Vec<u8> },
}

/// Write batch wrapper
pub struct WriteBatchWrapper {
    operations: Vec<BatchOp>,
}

impl WriteBatchWrapper {
    /// Create a new write batch
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    /// Add a put operation
    pub fn put(&mut self, cf_name: &str, key: &[u8], value: &[u8]) {
        self.operations.push(BatchOp::Put {
            cf_name: cf_name.to_string(),
            key: key.to_vec(),
            value: value.to_vec(),
        });
    }

    /// Add a delete operation
    pub fn delete(&mut self, cf_name: &str, key: &[u8]) {
        self.operations.push(BatchOp::Delete {
            cf_name: cf_name.to_string(),
            key: key.to_vec(),
        });
    }

    /// Get number of operations
    pub fn len(&self) -> usize {
        self.operations.len()
    }

    /// Check if batch is empty
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }
}

impl Default for WriteBatchWrapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::Arc;
    use std::thread;

    fn temp_db_path() -> String {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let cnt = COUNTER.fetch_add(1, Ordering::SeqCst);
        format!("/tmp/bach_test_db_{}_{}", id, cnt)
    }

    fn cleanup(path: &str) {
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn test_open_close() {
        let path = temp_db_path();
        let db = Database::new(&path);

        assert!(!db.is_open());
        db.open().unwrap();
        assert!(db.is_open());
        db.close();
        assert!(!db.is_open());

        cleanup(&path);
    }

    #[test]
    fn test_put_get() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        db.put(cf::ACCOUNTS, b"key1", b"value1").unwrap();
        let value = db.get(cf::ACCOUNTS, b"key1").unwrap();
        assert_eq!(value, Some(b"value1".to_vec()));

        let missing = db.get(cf::ACCOUNTS, b"missing").unwrap();
        assert_eq!(missing, None);

        db.close();
        cleanup(&path);
    }

    #[test]
    fn test_delete() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        db.put(cf::CODE, b"code1", b"bytecode").unwrap();
        assert!(db.get(cf::CODE, b"code1").unwrap().is_some());

        db.delete(cf::CODE, b"code1").unwrap();
        assert!(db.get(cf::CODE, b"code1").unwrap().is_none());

        db.close();
        cleanup(&path);
    }

    #[test]
    fn test_write_batch() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let mut batch = db.batch();
        batch.put(cf::ACCOUNTS, b"acc1", b"data1");
        batch.put(cf::ACCOUNTS, b"acc2", b"data2");
        batch.put(cf::STORAGE, b"store1", b"val1");

        assert_eq!(batch.len(), 3);
        db.write_batch(batch).unwrap();

        assert_eq!(db.get(cf::ACCOUNTS, b"acc1").unwrap(), Some(b"data1".to_vec()));
        assert_eq!(db.get(cf::ACCOUNTS, b"acc2").unwrap(), Some(b"data2".to_vec()));
        assert_eq!(db.get(cf::STORAGE, b"store1").unwrap(), Some(b"val1".to_vec()));

        db.close();
        cleanup(&path);
    }

    #[test]
    fn test_not_open_error() {
        let db = Database::new("/tmp/not_opened");
        let result = db.get(cf::ACCOUNTS, b"key");
        assert!(matches!(result, Err(StorageError::NotOpen)));
    }

    #[test]
    fn test_column_families() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        // Test all column families work
        for cf in ALL_CFS {
            db.put(cf, b"test_key", b"test_value").unwrap();
            let val = db.get(cf, b"test_key").unwrap();
            assert_eq!(val, Some(b"test_value".to_vec()));
        }

        db.close();
        cleanup(&path);
    }

    // ==================== Additional Database Tests ====================

    #[test]
    fn test_already_open_error() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let result = db.open();
        assert!(matches!(result, Err(StorageError::AlreadyOpen)));

        db.close();
        cleanup(&path);
    }

    #[test]
    fn test_reopen_database() {
        let path = temp_db_path();
        let db = Database::new(&path);

        // First open
        db.open().unwrap();
        db.put(cf::ACCOUNTS, b"key1", b"value1").unwrap();
        db.close();

        // Reopen and verify data persisted
        db.open().unwrap();
        let value = db.get(cf::ACCOUNTS, b"key1").unwrap();
        assert_eq!(value, Some(b"value1".to_vec()));

        db.close();
        cleanup(&path);
    }

    #[test]
    fn test_database_path() {
        let path = "/tmp/test_path_db";
        let db = Database::new(path);
        assert_eq!(db.path(), path);
    }

    #[test]
    fn test_put_not_open() {
        let db = Database::new("/tmp/not_opened_put");
        let result = db.put(cf::ACCOUNTS, b"key", b"value");
        assert!(matches!(result, Err(StorageError::NotOpen)));
    }

    #[test]
    fn test_delete_not_open() {
        let db = Database::new("/tmp/not_opened_delete");
        let result = db.delete(cf::ACCOUNTS, b"key");
        assert!(matches!(result, Err(StorageError::NotOpen)));
    }

    #[test]
    fn test_write_batch_not_open() {
        let db = Database::new("/tmp/not_opened_batch");
        let batch = db.batch();
        let result = db.write_batch(batch);
        assert!(matches!(result, Err(StorageError::NotOpen)));
    }

    // ==================== WriteBatchWrapper Tests ====================

    #[test]
    fn test_write_batch_default() {
        let batch = WriteBatchWrapper::default();
        assert!(batch.is_empty());
        assert_eq!(batch.len(), 0);
    }

    #[test]
    fn test_write_batch_operations() {
        let mut batch = WriteBatchWrapper::new();

        assert!(batch.is_empty());

        batch.put(cf::ACCOUNTS, b"key1", b"value1");
        assert_eq!(batch.len(), 1);
        assert!(!batch.is_empty());

        batch.put(cf::ACCOUNTS, b"key2", b"value2");
        batch.delete(cf::ACCOUNTS, b"key3");
        assert_eq!(batch.len(), 3);
    }

    #[test]
    fn test_write_batch_with_delete() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        // First put some data
        db.put(cf::ACCOUNTS, b"key1", b"value1").unwrap();
        db.put(cf::ACCOUNTS, b"key2", b"value2").unwrap();

        // Batch with delete
        let mut batch = db.batch();
        batch.delete(cf::ACCOUNTS, b"key1");
        batch.put(cf::ACCOUNTS, b"key3", b"value3");
        db.write_batch(batch).unwrap();

        // Verify
        assert!(db.get(cf::ACCOUNTS, b"key1").unwrap().is_none());
        assert_eq!(db.get(cf::ACCOUNTS, b"key2").unwrap(), Some(b"value2".to_vec()));
        assert_eq!(db.get(cf::ACCOUNTS, b"key3").unwrap(), Some(b"value3".to_vec()));

        db.close();
        cleanup(&path);
    }

    // ==================== DbConfig Tests ====================

    #[test]
    fn test_db_config_default() {
        let config = DbConfig::default();
        assert!(config.create_if_missing);
        assert_eq!(config.max_open_files, 512);
        assert_eq!(config.write_buffer_size, 64 * 1024 * 1024);
        assert_eq!(config.max_write_buffer_number, 3);
    }

    #[test]
    fn test_open_with_custom_config() {
        let path = temp_db_path();
        let db = Database::new(&path);

        let config = DbConfig {
            create_if_missing: true,
            max_open_files: 256,
            write_buffer_size: 32 * 1024 * 1024,
            max_write_buffer_number: 2,
        };

        db.open_with_config(config).unwrap();
        assert!(db.is_open());

        db.close();
        cleanup(&path);
    }

    // ==================== Clone Tests ====================

    #[test]
    fn test_database_clone() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let db_clone = db.clone();

        // Both should see the same data
        db.put(cf::ACCOUNTS, b"key1", b"value1").unwrap();
        let value = db_clone.get(cf::ACCOUNTS, b"key1").unwrap();
        assert_eq!(value, Some(b"value1".to_vec()));

        db.close();
        cleanup(&path);
    }

    // ==================== Concurrent Access Tests ====================

    #[test]
    fn test_concurrent_reads() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        // Write some data
        for i in 0..100u8 {
            let key = format!("key_{}", i).into_bytes();
            let value = format!("value_{}", i).into_bytes();
            db.put(cf::ACCOUNTS, &key, &value).unwrap();
        }

        let db = Arc::new(db);
        let mut handles = vec![];

        // Multiple threads reading
        for thread_id in 0..5 {
            let db = Arc::clone(&db);
            let handle = thread::spawn(move || {
                for i in 0..100u8 {
                    let key = format!("key_{}", i).into_bytes();
                    let expected = format!("value_{}", i).into_bytes();
                    let value = db.get(cf::ACCOUNTS, &key).unwrap().unwrap();
                    assert_eq!(value, expected, "Thread {} failed on key_{}", thread_id, i);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        db.close();
        cleanup(&path);
    }

    #[test]
    fn test_concurrent_writes_different_keys() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let db = Arc::new(db);
        let mut handles = vec![];

        // Multiple threads writing to different keys
        for thread_id in 0..5 {
            let db = Arc::clone(&db);
            let handle = thread::spawn(move || {
                for i in 0..20u8 {
                    let key = format!("thread_{}_key_{}", thread_id, i).into_bytes();
                    let value = format!("thread_{}_value_{}", thread_id, i).into_bytes();
                    db.put(cf::ACCOUNTS, &key, &value).unwrap();
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Verify all writes
        for thread_id in 0..5 {
            for i in 0..20u8 {
                let key = format!("thread_{}_key_{}", thread_id, i).into_bytes();
                let expected = format!("thread_{}_value_{}", thread_id, i).into_bytes();
                let value = db.get(cf::ACCOUNTS, &key).unwrap().unwrap();
                assert_eq!(value, expected);
            }
        }

        db.close();
        cleanup(&path);
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_empty_key() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        db.put(cf::ACCOUNTS, b"", b"empty_key_value").unwrap();
        let value = db.get(cf::ACCOUNTS, b"").unwrap();
        assert_eq!(value, Some(b"empty_key_value".to_vec()));

        db.close();
        cleanup(&path);
    }

    #[test]
    fn test_empty_value() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        db.put(cf::ACCOUNTS, b"key_with_empty_value", b"").unwrap();
        let value = db.get(cf::ACCOUNTS, b"key_with_empty_value").unwrap();
        assert_eq!(value, Some(Vec::new()));

        db.close();
        cleanup(&path);
    }

    #[test]
    fn test_large_value() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        // 1 MB value
        let large_value = vec![0xaa; 1024 * 1024];
        db.put(cf::CODE, b"large_code", &large_value).unwrap();

        let retrieved = db.get(cf::CODE, b"large_code").unwrap().unwrap();
        assert_eq!(retrieved.len(), 1024 * 1024);
        assert_eq!(retrieved, large_value);

        db.close();
        cleanup(&path);
    }

    #[test]
    fn test_overwrite_value() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        db.put(cf::ACCOUNTS, b"key1", b"original").unwrap();
        assert_eq!(db.get(cf::ACCOUNTS, b"key1").unwrap(), Some(b"original".to_vec()));

        db.put(cf::ACCOUNTS, b"key1", b"updated").unwrap();
        assert_eq!(db.get(cf::ACCOUNTS, b"key1").unwrap(), Some(b"updated".to_vec()));

        db.close();
        cleanup(&path);
    }

    #[test]
    fn test_delete_nonexistent() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        // Deleting a key that doesn't exist should not error
        db.delete(cf::ACCOUNTS, b"nonexistent").unwrap();

        db.close();
        cleanup(&path);
    }

    // ==================== Column Family Isolation Tests ====================

    #[test]
    fn test_column_family_isolation() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        // Same key in different column families
        db.put(cf::ACCOUNTS, b"same_key", b"accounts_value").unwrap();
        db.put(cf::STORAGE, b"same_key", b"storage_value").unwrap();
        db.put(cf::CODE, b"same_key", b"code_value").unwrap();

        assert_eq!(db.get(cf::ACCOUNTS, b"same_key").unwrap(), Some(b"accounts_value".to_vec()));
        assert_eq!(db.get(cf::STORAGE, b"same_key").unwrap(), Some(b"storage_value".to_vec()));
        assert_eq!(db.get(cf::CODE, b"same_key").unwrap(), Some(b"code_value".to_vec()));

        // Delete from one CF shouldn't affect others
        db.delete(cf::ACCOUNTS, b"same_key").unwrap();
        assert!(db.get(cf::ACCOUNTS, b"same_key").unwrap().is_none());
        assert!(db.get(cf::STORAGE, b"same_key").unwrap().is_some());
        assert!(db.get(cf::CODE, b"same_key").unwrap().is_some());

        db.close();
        cleanup(&path);
    }
}
