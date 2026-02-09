//! BachLedger State
//!
//! State management for blockchain operations:
//! - `StateDB`: Trait for state storage
//! - `MemoryStateDB`: In-memory implementation
//! - `Snapshot`: Read-only state snapshot
//! - `OwnershipEntry`: Per-key ownership tracking
//! - `OwnershipTable`: Concurrent ownership table

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use bach_primitives::H256;
use bach_types::PriorityCode;

/// Errors from state operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateError {
    /// Key not found in state
    KeyNotFound(H256),
    /// Snapshot has expired or is invalid
    SnapshotExpired,
    /// Lock acquisition failed
    LockError(String),
}

/// Interface for state storage.
pub trait StateDB: Send + Sync {
    /// Reads a value by key.
    fn get(&self, key: &H256) -> Option<Vec<u8>>;

    /// Writes a value.
    fn set(&mut self, key: H256, value: Vec<u8>);

    /// Deletes a key.
    fn delete(&mut self, key: &H256);

    /// Creates a read-only snapshot.
    fn snapshot(&self) -> Snapshot;

    /// Commits a batch of writes atomically.
    fn commit(&mut self, writes: &[(H256, Vec<u8>)]);

    /// Returns all keys (for testing/debugging).
    fn keys(&self) -> Vec<H256>;
}

/// In-memory implementation of StateDB.
#[derive(Debug, Default)]
pub struct MemoryStateDB {
    data: HashMap<H256, Vec<u8>>,
}

impl MemoryStateDB {
    /// Creates a new empty state database.
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

impl StateDB for MemoryStateDB {
    fn get(&self, key: &H256) -> Option<Vec<u8>> {
        self.data.get(key).cloned()
    }

    fn set(&mut self, key: H256, value: Vec<u8>) {
        self.data.insert(key, value);
    }

    fn delete(&mut self, key: &H256) {
        self.data.remove(key);
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            data: self.data.clone(),
        }
    }

    fn commit(&mut self, writes: &[(H256, Vec<u8>)]) {
        for (key, value) in writes {
            self.data.insert(*key, value.clone());
        }
    }

    fn keys(&self) -> Vec<H256> {
        self.data.keys().copied().collect()
    }
}

/// A read-only snapshot of state at a point in time.
#[derive(Debug, Clone)]
pub struct Snapshot {
    data: HashMap<H256, Vec<u8>>,
}

impl Snapshot {
    /// Reads a value by key from the snapshot.
    pub fn get(&self, key: &H256) -> Option<Vec<u8>> {
        self.data.get(key).cloned()
    }
}

/// An entry in the ownership table for a single key.
/// Implements Algorithm 1 from the Seamless Scheduling paper.
pub struct OwnershipEntry {
    /// Current owner's priority code. Protected by RwLock for thread safety.
    /// DISOWNED state is represented by a released PriorityCode.
    owner: RwLock<PriorityCode>,
}

impl OwnershipEntry {
    /// Creates a new entry with DISOWNED status.
    pub fn new() -> Self {
        // Create a released priority code to represent DISOWNED
        let mut pc = PriorityCode::new(u64::MAX, H256::zero());
        pc.release();
        Self {
            owner: RwLock::new(pc),
        }
    }

    /// Releases ownership by setting status to DISOWNED.
    pub fn release_ownership(&self) {
        let mut owner = self.owner.write().unwrap();
        owner.release();
    }

    /// Checks if the given priority code can claim ownership.
    /// Returns true if `who <= current_owner` (higher or equal priority).
    pub fn check_ownership(&self, who: &PriorityCode) -> bool {
        let owner = self.owner.read().unwrap();
        who <= &*owner
    }

    /// Attempts to claim ownership.
    /// Returns true if ownership was successfully claimed (who has higher or equal priority).
    /// Returns false if a higher-priority transaction already owns this key.
    pub fn try_set_owner(&self, who: &PriorityCode) -> bool {
        let mut owner = self.owner.write().unwrap();
        if who <= &*owner {
            *owner = who.clone();
            true
        } else {
            false
        }
    }

    /// Returns a clone of the current owner's priority code.
    pub fn current_owner(&self) -> PriorityCode {
        self.owner.read().unwrap().clone()
    }
}

impl Default for OwnershipEntry {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for OwnershipEntry {
    fn clone(&self) -> Self {
        let owner = self.owner.read().unwrap().clone();
        Self {
            owner: RwLock::new(owner),
        }
    }
}

/// Table mapping storage keys to their ownership entries.
/// Uses a concurrent hashmap for thread-safe access.
pub struct OwnershipTable {
    entries: RwLock<HashMap<H256, Arc<OwnershipEntry>>>,
}

impl OwnershipTable {
    /// Creates a new empty ownership table.
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
        }
    }

    /// Gets the ownership entry for a key, creating one if it doesn't exist.
    pub fn get_or_create(&self, key: &H256) -> Arc<OwnershipEntry> {
        // First try to read
        {
            let entries = self.entries.read().unwrap();
            if let Some(entry) = entries.get(key) {
                return Arc::clone(entry);
            }
        }

        // Need to write - acquire write lock and check again
        let mut entries = self.entries.write().unwrap();
        if let Some(entry) = entries.get(key) {
            return Arc::clone(entry);
        }

        // Create new entry
        let entry = Arc::new(OwnershipEntry::new());
        entries.insert(*key, Arc::clone(&entry));
        entry
    }

    /// Releases ownership of all specified keys.
    pub fn release_all(&self, keys: &[H256]) {
        let entries = self.entries.read().unwrap();
        for key in keys {
            if let Some(entry) = entries.get(key) {
                entry.release_ownership();
            }
        }
    }

    /// Clears all entries from the table.
    pub fn clear(&self) {
        let mut entries = self.entries.write().unwrap();
        entries.clear();
    }

    /// Returns the number of entries.
    pub fn len(&self) -> usize {
        self.entries.read().unwrap().len()
    }

    /// Returns true if the table is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.read().unwrap().is_empty()
    }
}

impl Default for OwnershipTable {
    fn default() -> Self {
        Self::new()
    }
}
