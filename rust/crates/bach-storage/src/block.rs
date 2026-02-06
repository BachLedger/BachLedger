//! Block storage implementation

use crate::db::{cf, Database};
use crate::error::StorageResult;
use bach_primitives::H256;

/// Block database for storing block headers, bodies, and receipts
pub struct BlockDb {
    db: Database,
}

impl BlockDb {
    /// Create a new block database
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Get the underlying database
    pub fn database(&self) -> &Database {
        &self.db
    }

    // ========== Block Headers ==========

    /// Store a block header
    pub fn put_header(&self, hash: &H256, header: &[u8]) -> StorageResult<()> {
        self.db.put(cf::HEADERS, hash.as_bytes(), header)
    }

    /// Get a block header by hash
    pub fn get_header(&self, hash: &H256) -> StorageResult<Option<Vec<u8>>> {
        self.db.get(cf::HEADERS, hash.as_bytes())
    }

    /// Delete a block header
    pub fn delete_header(&self, hash: &H256) -> StorageResult<()> {
        self.db.delete(cf::HEADERS, hash.as_bytes())
    }

    // ========== Block Bodies ==========

    /// Store a block body
    pub fn put_body(&self, hash: &H256, body: &[u8]) -> StorageResult<()> {
        self.db.put(cf::BODIES, hash.as_bytes(), body)
    }

    /// Get a block body by hash
    pub fn get_body(&self, hash: &H256) -> StorageResult<Option<Vec<u8>>> {
        self.db.get(cf::BODIES, hash.as_bytes())
    }

    /// Delete a block body
    pub fn delete_body(&self, hash: &H256) -> StorageResult<()> {
        self.db.delete(cf::BODIES, hash.as_bytes())
    }

    // ========== Receipts ==========

    /// Store receipts for a block
    pub fn put_receipts(&self, hash: &H256, receipts: &[u8]) -> StorageResult<()> {
        self.db.put(cf::RECEIPTS, hash.as_bytes(), receipts)
    }

    /// Get receipts for a block
    pub fn get_receipts(&self, hash: &H256) -> StorageResult<Option<Vec<u8>>> {
        self.db.get(cf::RECEIPTS, hash.as_bytes())
    }

    /// Delete receipts for a block
    pub fn delete_receipts(&self, hash: &H256) -> StorageResult<()> {
        self.db.delete(cf::RECEIPTS, hash.as_bytes())
    }

    // ========== Block Index (number -> hash) ==========

    /// Store block number to hash mapping
    pub fn put_hash_by_number(&self, number: u64, hash: &H256) -> StorageResult<()> {
        self.db.put(cf::BLOCK_INDEX, &number.to_be_bytes(), hash.as_bytes())
    }

    /// Get block hash by number
    pub fn get_hash_by_number(&self, number: u64) -> StorageResult<Option<H256>> {
        let bytes = self.db.get(cf::BLOCK_INDEX, &number.to_be_bytes())?;
        match bytes {
            Some(b) => Ok(Some(H256::from_slice(&b).map_err(|e| {
                crate::error::StorageError::Deserialization(e.to_string())
            })?)),
            None => Ok(None),
        }
    }

    /// Delete block number to hash mapping
    pub fn delete_hash_by_number(&self, number: u64) -> StorageResult<()> {
        self.db.delete(cf::BLOCK_INDEX, &number.to_be_bytes())
    }

    // ========== Metadata ==========

    /// Store metadata value
    pub fn put_meta(&self, key: &[u8], value: &[u8]) -> StorageResult<()> {
        self.db.put(cf::META, key, value)
    }

    /// Get metadata value
    pub fn get_meta(&self, key: &[u8]) -> StorageResult<Option<Vec<u8>>> {
        self.db.get(cf::META, key)
    }

    /// Store latest block number
    pub fn set_latest_block(&self, number: u64) -> StorageResult<()> {
        self.put_meta(b"latest_block", &number.to_be_bytes())
    }

    /// Get latest block number
    pub fn get_latest_block(&self) -> StorageResult<Option<u64>> {
        let bytes = self.get_meta(b"latest_block")?;
        Ok(bytes.map(|b| {
            let arr: [u8; 8] = b.try_into().unwrap_or([0; 8]);
            u64::from_be_bytes(arr)
        }))
    }

    /// Store finalized block number
    pub fn set_finalized_block(&self, number: u64) -> StorageResult<()> {
        self.put_meta(b"finalized_block", &number.to_be_bytes())
    }

    /// Get finalized block number
    pub fn get_finalized_block(&self) -> StorageResult<Option<u64>> {
        let bytes = self.get_meta(b"finalized_block")?;
        Ok(bytes.map(|b| {
            let arr: [u8; 8] = b.try_into().unwrap_or([0; 8]);
            u64::from_be_bytes(arr)
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_db_path() -> String {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let cnt = COUNTER.fetch_add(1, Ordering::SeqCst);
        format!("/tmp/bach_block_test_{}_{}", id, cnt)
    }

    fn cleanup(path: &str) {
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn test_header_storage() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let block_db = BlockDb::new(db);
        let hash = H256::from_bytes([0x42; 32]);
        let header = b"mock_header_data";

        // Store and retrieve
        block_db.put_header(&hash, header).unwrap();
        let retrieved = block_db.get_header(&hash).unwrap().unwrap();
        assert_eq!(retrieved, header.to_vec());

        // Delete
        block_db.delete_header(&hash).unwrap();
        assert!(block_db.get_header(&hash).unwrap().is_none());

        block_db.database().close();
        cleanup(&path);
    }

    #[test]
    fn test_body_storage() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let block_db = BlockDb::new(db);
        let hash = H256::from_bytes([0x42; 32]);
        let body = b"mock_body_with_transactions";

        block_db.put_body(&hash, body).unwrap();
        let retrieved = block_db.get_body(&hash).unwrap().unwrap();
        assert_eq!(retrieved, body.to_vec());

        block_db.database().close();
        cleanup(&path);
    }

    #[test]
    fn test_receipts_storage() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let block_db = BlockDb::new(db);
        let hash = H256::from_bytes([0x42; 32]);
        let receipts = b"mock_receipts_data";

        block_db.put_receipts(&hash, receipts).unwrap();
        let retrieved = block_db.get_receipts(&hash).unwrap().unwrap();
        assert_eq!(retrieved, receipts.to_vec());

        block_db.database().close();
        cleanup(&path);
    }

    #[test]
    fn test_block_index() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let block_db = BlockDb::new(db);
        let hash = H256::from_bytes([0x42; 32]);
        let number: u64 = 12345;

        // Initially none
        assert!(block_db.get_hash_by_number(number).unwrap().is_none());

        // Store mapping
        block_db.put_hash_by_number(number, &hash).unwrap();
        let retrieved = block_db.get_hash_by_number(number).unwrap().unwrap();
        assert_eq!(retrieved, hash);

        // Delete mapping
        block_db.delete_hash_by_number(number).unwrap();
        assert!(block_db.get_hash_by_number(number).unwrap().is_none());

        block_db.database().close();
        cleanup(&path);
    }

    #[test]
    fn test_latest_block() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let block_db = BlockDb::new(db);

        // Initially none
        assert!(block_db.get_latest_block().unwrap().is_none());

        // Set latest
        block_db.set_latest_block(100).unwrap();
        assert_eq!(block_db.get_latest_block().unwrap(), Some(100));

        // Update
        block_db.set_latest_block(200).unwrap();
        assert_eq!(block_db.get_latest_block().unwrap(), Some(200));

        block_db.database().close();
        cleanup(&path);
    }

    #[test]
    fn test_finalized_block() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let block_db = BlockDb::new(db);

        block_db.set_finalized_block(50).unwrap();
        assert_eq!(block_db.get_finalized_block().unwrap(), Some(50));

        block_db.database().close();
        cleanup(&path);
    }

    // ==================== Additional Header/Body/Receipt Tests ====================

    #[test]
    fn test_header_not_found() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let block_db = BlockDb::new(db);
        let hash = H256::from_bytes([0x99; 32]);

        assert!(block_db.get_header(&hash).unwrap().is_none());

        block_db.database().close();
        cleanup(&path);
    }

    #[test]
    fn test_body_delete() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let block_db = BlockDb::new(db);
        let hash = H256::from_bytes([0x42; 32]);
        let body = b"body_data";

        block_db.put_body(&hash, body).unwrap();
        assert!(block_db.get_body(&hash).unwrap().is_some());

        block_db.delete_body(&hash).unwrap();
        assert!(block_db.get_body(&hash).unwrap().is_none());

        block_db.database().close();
        cleanup(&path);
    }

    #[test]
    fn test_receipts_delete() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let block_db = BlockDb::new(db);
        let hash = H256::from_bytes([0x42; 32]);
        let receipts = b"receipts_data";

        block_db.put_receipts(&hash, receipts).unwrap();
        assert!(block_db.get_receipts(&hash).unwrap().is_some());

        block_db.delete_receipts(&hash).unwrap();
        assert!(block_db.get_receipts(&hash).unwrap().is_none());

        block_db.database().close();
        cleanup(&path);
    }

    #[test]
    fn test_multiple_blocks() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let block_db = BlockDb::new(db);

        // Store multiple blocks
        for i in 0..10u8 {
            let hash = H256::from_bytes([i; 32]);
            let header = format!("header_{}", i).into_bytes();
            let body = format!("body_{}", i).into_bytes();
            let receipts = format!("receipts_{}", i).into_bytes();

            block_db.put_header(&hash, &header).unwrap();
            block_db.put_body(&hash, &body).unwrap();
            block_db.put_receipts(&hash, &receipts).unwrap();
            block_db.put_hash_by_number(i as u64, &hash).unwrap();
        }

        // Verify all blocks
        for i in 0..10u8 {
            let hash = H256::from_bytes([i; 32]);
            let expected_header = format!("header_{}", i).into_bytes();
            let expected_body = format!("body_{}", i).into_bytes();
            let expected_receipts = format!("receipts_{}", i).into_bytes();

            assert_eq!(block_db.get_header(&hash).unwrap().unwrap(), expected_header);
            assert_eq!(block_db.get_body(&hash).unwrap().unwrap(), expected_body);
            assert_eq!(block_db.get_receipts(&hash).unwrap().unwrap(), expected_receipts);
            assert_eq!(block_db.get_hash_by_number(i as u64).unwrap().unwrap(), hash);
        }

        block_db.database().close();
        cleanup(&path);
    }

    // ==================== Block Index Tests ====================

    #[test]
    fn test_block_index_boundary_values() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let block_db = BlockDb::new(db);
        let hash = H256::from_bytes([0x42; 32]);

        // Test block 0
        block_db.put_hash_by_number(0, &hash).unwrap();
        assert_eq!(block_db.get_hash_by_number(0).unwrap().unwrap(), hash);

        // Test large block number
        let large_number = u64::MAX - 1;
        block_db.put_hash_by_number(large_number, &hash).unwrap();
        assert_eq!(block_db.get_hash_by_number(large_number).unwrap().unwrap(), hash);

        block_db.database().close();
        cleanup(&path);
    }

    #[test]
    fn test_block_index_overwrite() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let block_db = BlockDb::new(db);
        let hash1 = H256::from_bytes([0x01; 32]);
        let hash2 = H256::from_bytes([0x02; 32]);
        let number: u64 = 100;

        block_db.put_hash_by_number(number, &hash1).unwrap();
        assert_eq!(block_db.get_hash_by_number(number).unwrap().unwrap(), hash1);

        // Overwrite with new hash
        block_db.put_hash_by_number(number, &hash2).unwrap();
        assert_eq!(block_db.get_hash_by_number(number).unwrap().unwrap(), hash2);

        block_db.database().close();
        cleanup(&path);
    }

    // ==================== Metadata Tests ====================

    #[test]
    fn test_meta_put_get() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let block_db = BlockDb::new(db);

        block_db.put_meta(b"custom_key", b"custom_value").unwrap();
        let value = block_db.get_meta(b"custom_key").unwrap().unwrap();
        assert_eq!(value, b"custom_value".to_vec());

        block_db.database().close();
        cleanup(&path);
    }

    #[test]
    fn test_meta_not_found() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let block_db = BlockDb::new(db);

        assert!(block_db.get_meta(b"nonexistent").unwrap().is_none());

        block_db.database().close();
        cleanup(&path);
    }

    #[test]
    fn test_latest_and_finalized_tracking() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let block_db = BlockDb::new(db);

        // Simulate block progression
        for i in 0..10u64 {
            block_db.set_latest_block(i).unwrap();
            if i >= 3 {
                block_db.set_finalized_block(i - 3).unwrap();
            }
        }

        assert_eq!(block_db.get_latest_block().unwrap(), Some(9));
        assert_eq!(block_db.get_finalized_block().unwrap(), Some(6));

        block_db.database().close();
        cleanup(&path);
    }

    #[test]
    fn test_finalized_initially_none() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let block_db = BlockDb::new(db);

        assert!(block_db.get_finalized_block().unwrap().is_none());

        block_db.database().close();
        cleanup(&path);
    }

    // ==================== Large Data Tests ====================

    #[test]
    fn test_large_header_body() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let block_db = BlockDb::new(db);
        let hash = H256::from_bytes([0x42; 32]);

        // Large header (1 KB)
        let large_header = vec![0xaa; 1024];
        block_db.put_header(&hash, &large_header).unwrap();
        assert_eq!(block_db.get_header(&hash).unwrap().unwrap(), large_header);

        // Large body (100 KB)
        let large_body = vec![0xbb; 100 * 1024];
        block_db.put_body(&hash, &large_body).unwrap();
        assert_eq!(block_db.get_body(&hash).unwrap().unwrap(), large_body);

        block_db.database().close();
        cleanup(&path);
    }

    #[test]
    fn test_empty_data() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let block_db = BlockDb::new(db);
        let hash = H256::from_bytes([0x42; 32]);

        // Empty header
        block_db.put_header(&hash, &[]).unwrap();
        assert_eq!(block_db.get_header(&hash).unwrap().unwrap(), Vec::<u8>::new());

        // Empty receipts
        block_db.put_receipts(&hash, &[]).unwrap();
        assert_eq!(block_db.get_receipts(&hash).unwrap().unwrap(), Vec::<u8>::new());

        block_db.database().close();
        cleanup(&path);
    }
}
