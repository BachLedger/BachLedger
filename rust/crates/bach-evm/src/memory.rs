//! EVM memory implementation

use crate::stack::U256;

/// EVM memory (byte-addressable, expandable)
#[derive(Clone, Debug, Default)]
pub struct Memory {
    data: Vec<u8>,
}

impl Memory {
    /// Create new empty memory
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Get current memory size in bytes
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Ensure memory is at least `size` bytes, expanding if needed
    /// Returns the gas cost for expansion
    pub fn expand(&mut self, offset: usize, size: usize) -> usize {
        if size == 0 {
            return 0;
        }

        let new_size = offset.saturating_add(size);
        let current_size = self.data.len();

        if new_size > current_size {
            // Round up to 32-byte word boundary
            let new_size_words = new_size.div_ceil(32);
            let new_size_aligned = new_size_words * 32;
            self.data.resize(new_size_aligned, 0);
            new_size_aligned
        } else {
            current_size
        }
    }

    /// Load a 32-byte word from memory
    pub fn load(&self, offset: usize) -> U256 {
        let mut result = [0u8; 32];
        let end = (offset + 32).min(self.data.len());
        let copy_len = end.saturating_sub(offset);

        if copy_len > 0 && offset < self.data.len() {
            result[..copy_len].copy_from_slice(&self.data[offset..end]);
        }

        result
    }

    /// Store a 32-byte word to memory
    pub fn store(&mut self, offset: usize, value: &U256) {
        self.expand(offset, 32);
        self.data[offset..offset + 32].copy_from_slice(value);
    }

    /// Store a single byte to memory
    pub fn store8(&mut self, offset: usize, value: u8) {
        self.expand(offset, 1);
        self.data[offset] = value;
    }

    /// Load a byte slice from memory
    pub fn load_slice(&self, offset: usize, size: usize) -> Vec<u8> {
        if size == 0 {
            return Vec::new();
        }

        let mut result = vec![0u8; size];
        let end = (offset + size).min(self.data.len());
        let copy_len = end.saturating_sub(offset);

        if copy_len > 0 && offset < self.data.len() {
            result[..copy_len].copy_from_slice(&self.data[offset..end]);
        }

        result
    }

    /// Store a byte slice to memory
    pub fn store_slice(&mut self, offset: usize, data: &[u8]) {
        if data.is_empty() {
            return;
        }
        self.expand(offset, data.len());
        self.data[offset..offset + data.len()].copy_from_slice(data);
    }

    /// Copy within memory (for MCOPY)
    pub fn copy(&mut self, dest: usize, src: usize, size: usize) {
        if size == 0 {
            return;
        }

        // Expand to cover both source and destination
        let max_offset = dest.max(src);
        self.expand(max_offset, size);

        // Use copy_within for overlapping copies
        if dest <= src {
            self.data.copy_within(src..src + size, dest);
        } else {
            // Copy backwards for overlapping regions where dest > src
            for i in (0..size).rev() {
                self.data[dest + i] = self.data[src + i];
            }
        }
    }

    /// Clear memory
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Get raw data slice
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stack::u64_to_u256;

    #[test]
    fn test_memory_expand() {
        let mut mem = Memory::new();
        assert_eq!(mem.size(), 0);

        mem.expand(0, 32);
        assert_eq!(mem.size(), 32);

        mem.expand(0, 64);
        assert_eq!(mem.size(), 64);

        // Aligned to 32 bytes
        mem.expand(0, 65);
        assert_eq!(mem.size(), 96);
    }

    #[test]
    fn test_memory_store_load() {
        let mut mem = Memory::new();
        let value = u64_to_u256(0x1234567890ABCDEF);

        mem.store(0, &value);
        let loaded = mem.load(0);
        assert_eq!(loaded, value);
    }

    #[test]
    fn test_memory_store8() {
        let mut mem = Memory::new();
        mem.store8(0, 0x42);
        assert_eq!(mem.data[0], 0x42);
    }

    #[test]
    fn test_memory_load_slice() {
        let mut mem = Memory::new();
        mem.store_slice(0, &[1, 2, 3, 4, 5]);

        let slice = mem.load_slice(0, 5);
        assert_eq!(slice, vec![1, 2, 3, 4, 5]);

        // Load beyond data returns zeros
        let slice = mem.load_slice(3, 5);
        assert_eq!(slice, vec![4, 5, 0, 0, 0]);
    }

    #[test]
    fn test_memory_copy() {
        let mut mem = Memory::new();
        mem.store_slice(0, &[1, 2, 3, 4, 5]);

        // Non-overlapping copy
        mem.copy(10, 0, 5);
        assert_eq!(mem.load_slice(10, 5), vec![1, 2, 3, 4, 5]);

        // Overlapping copy (dest > src)
        mem.copy(2, 0, 5);
        assert_eq!(mem.load_slice(0, 7), vec![1, 2, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_memory_load_uninitialized() {
        let mem = Memory::new();
        let value = mem.load(0);
        assert_eq!(value, [0u8; 32]);
    }

    // ==================== Extended Memory Tests ====================

    #[test]
    fn test_memory_default() {
        let mem: Memory = Default::default();
        assert_eq!(mem.size(), 0);
        assert!(mem.data().is_empty());
    }

    #[test]
    fn test_memory_expand_zero_size() {
        let mut mem = Memory::new();
        let result = mem.expand(100, 0);
        assert_eq!(result, 0);
        assert_eq!(mem.size(), 0); // No expansion for size 0
    }

    #[test]
    fn test_memory_expand_word_alignment() {
        let mut mem = Memory::new();

        // Expand to 1 byte should align to 32
        mem.expand(0, 1);
        assert_eq!(mem.size(), 32);

        // Expand to 33 bytes should align to 64
        mem.expand(0, 33);
        assert_eq!(mem.size(), 64);

        // Expand to 64 bytes stays at 64
        mem.expand(0, 64);
        assert_eq!(mem.size(), 64);

        // Expand to 65 bytes should align to 96
        mem.expand(0, 65);
        assert_eq!(mem.size(), 96);
    }

    #[test]
    fn test_memory_expand_with_offset() {
        let mut mem = Memory::new();

        // offset=10, size=30 -> needs 40 bytes -> aligns to 64
        mem.expand(10, 30);
        assert_eq!(mem.size(), 64);
    }

    #[test]
    fn test_memory_expand_no_shrink() {
        let mut mem = Memory::new();
        mem.expand(0, 100);
        let original_size = mem.size();

        // Expanding to smaller size should not shrink
        let result = mem.expand(0, 10);
        assert_eq!(result, original_size);
        assert_eq!(mem.size(), original_size);
    }

    #[test]
    fn test_memory_store8_various_offsets() {
        let mut mem = Memory::new();

        mem.store8(0, 0x01);
        mem.store8(31, 0x02);
        mem.store8(32, 0x03);

        assert_eq!(mem.data()[0], 0x01);
        assert_eq!(mem.data()[31], 0x02);
        assert_eq!(mem.data()[32], 0x03);
        assert_eq!(mem.size(), 64); // Expanded to cover offset 32
    }

    #[test]
    fn test_memory_load_slice_zero_size() {
        let mem = Memory::new();
        let slice = mem.load_slice(0, 0);
        assert!(slice.is_empty());
    }

    #[test]
    fn test_memory_load_slice_beyond_memory() {
        let mem = Memory::new();
        // Load from uninitialized memory should return zeros
        let slice = mem.load_slice(100, 10);
        assert_eq!(slice.len(), 10);
        assert!(slice.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_memory_store_slice_empty() {
        let mut mem = Memory::new();
        mem.store_slice(0, &[]);
        assert_eq!(mem.size(), 0); // No expansion for empty slice
    }

    #[test]
    fn test_memory_copy_zero_size() {
        let mut mem = Memory::new();
        mem.store_slice(0, &[1, 2, 3, 4, 5]);
        let original = mem.load_slice(0, 5);

        mem.copy(10, 0, 0); // Copy 0 bytes
        assert_eq!(mem.load_slice(0, 5), original); // Unchanged
    }

    #[test]
    fn test_memory_copy_overlapping_backward() {
        let mut mem = Memory::new();
        mem.store_slice(0, &[0, 0, 1, 2, 3, 4, 5, 0, 0, 0]);

        // Copy bytes 2-6 to positions 0-4 (dest < src, overlapping)
        mem.copy(0, 2, 5);
        // Expected: [1, 2, 3, 4, 5, 4, 5, 0, 0, 0]
        assert_eq!(mem.load_slice(0, 10), vec![1, 2, 3, 4, 5, 4, 5, 0, 0, 0]);
    }

    #[test]
    fn test_memory_copy_same_position() {
        let mut mem = Memory::new();
        mem.store_slice(0, &[1, 2, 3, 4, 5]);
        let original = mem.load_slice(0, 5);

        mem.copy(0, 0, 5); // Copy to same position
        assert_eq!(mem.load_slice(0, 5), original); // Should be unchanged
    }

    #[test]
    fn test_memory_clear() {
        let mut mem = Memory::new();
        mem.store_slice(0, &[1, 2, 3, 4, 5]);
        assert!(mem.size() > 0);

        mem.clear();
        assert_eq!(mem.size(), 0);
        assert!(mem.data().is_empty());
    }

    #[test]
    fn test_memory_large_offset() {
        let mut mem = Memory::new();
        // Store at a large offset
        mem.store8(1000, 0x42);
        assert!(mem.size() >= 1001);
        assert_eq!(mem.data()[1000], 0x42);
    }

    #[test]
    fn test_memory_store_load_at_boundary() {
        let mut mem = Memory::new();
        let value = u64_to_u256(0xDEADBEEF);

        // Store at word boundary
        mem.store(32, &value);
        let loaded = mem.load(32);
        assert_eq!(loaded, value);

        // Store crossing word boundary
        mem.store(48, &value);
        let loaded = mem.load(48);
        assert_eq!(loaded, value);
    }

    #[test]
    fn test_memory_data_access() {
        let mut mem = Memory::new();
        mem.store_slice(0, &[1, 2, 3, 4, 5]);

        let data = mem.data();
        assert!(data.len() >= 5);
        assert_eq!(&data[0..5], &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_memory_clone() {
        let mut mem = Memory::new();
        mem.store_slice(0, &[1, 2, 3, 4, 5]);

        let cloned = mem.clone();
        assert_eq!(cloned.size(), mem.size());
        assert_eq!(cloned.data(), mem.data());
    }

    #[test]
    fn test_memory_mstore8_lowest_byte() {
        let mut mem = Memory::new();
        // MSTORE8 should store the least significant byte
        mem.store8(0, 0xFF);
        assert_eq!(mem.data()[0], 0xFF);
    }

    #[test]
    fn test_memory_load_partial_overlap() {
        let mut mem = Memory::new();
        // Store only 5 bytes
        mem.store_slice(0, &[1, 2, 3, 4, 5]);

        // Load 32 bytes starting at offset 3
        // Should get [4, 5, 0, 0, ...zeros...]
        let value = mem.load(3);
        assert_eq!(value[0], 4);
        assert_eq!(value[1], 5);
        for i in 2..32 {
            assert_eq!(value[i], 0);
        }
    }
}
