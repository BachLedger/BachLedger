//! Keccak-256 hashing

use bach_primitives::H256;
use sha3::{Digest, Keccak256};

/// Compute Keccak-256 hash of the input data
pub fn keccak256(data: &[u8]) -> H256 {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    let result = hasher.finalize();
    H256::from_bytes(result.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Ethereum official test vectors ====================

    #[test]
    fn test_keccak256_empty() {
        // keccak256("") = 0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470
        let hash = keccak256(&[]);
        assert_eq!(
            hash.to_hex(),
            "0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470"
        );
    }

    #[test]
    fn test_keccak256_hello() {
        // keccak256("hello") = 0x1c8aff950685c2ed4bc3174f3472287b56d9517b9c948127319a09a7a36deac8
        let hash = keccak256(b"hello");
        assert_eq!(
            hash.to_hex(),
            "0x1c8aff950685c2ed4bc3174f3472287b56d9517b9c948127319a09a7a36deac8"
        );
    }

    #[test]
    fn test_keccak256_hello_world() {
        // keccak256("Hello, World!") verified with ethers.js
        let hash = keccak256(b"Hello, World!");
        assert_eq!(
            hash.to_hex(),
            "0xacaf3289d7b601cbd114fb36c4d29c85bbfd5e133f14cb355c3fd8d99367964f"
        );
    }

    #[test]
    fn test_keccak256_quick_brown_fox() {
        // keccak256("The quick brown fox jumps over the lazy dog")
        let hash = keccak256(b"The quick brown fox jumps over the lazy dog");
        assert_eq!(
            hash.to_hex(),
            "0x4d741b6f1eb29cb2a9b9911c82f56fa8d73b04959d3d9d222895df6c0b28aa15"
        );
    }

    // ==================== Various input lengths ====================

    #[test]
    fn test_keccak256_single_byte() {
        // keccak256("\x00")
        let hash = keccak256(&[0x00]);
        assert_eq!(
            hash.to_hex(),
            "0xbc36789e7a1e281436464229828f817d6612f7b477d66591ff96a9e064bcc98a"
        );
    }

    #[test]
    fn test_keccak256_single_byte_ff() {
        // keccak256("\xff") - verified with sha3 crate
        let hash = keccak256(&[0xff]);
        assert_eq!(
            hash.to_hex(),
            "0x8b1a944cf13a9a1c08facb2c9e98623ef3254d2ddb48113885c3e8e97fec8db9"
        );
    }

    #[test]
    fn test_keccak256_32_bytes() {
        // 32 bytes of zeros
        let data = [0u8; 32];
        let hash = keccak256(&data);
        assert_eq!(
            hash.to_hex(),
            "0x290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563"
        );
    }

    #[test]
    fn test_keccak256_64_bytes() {
        // 64 bytes of zeros (exactly one block)
        let data = [0u8; 64];
        let hash = keccak256(&data);
        // This is deterministic - we just verify it doesn't panic and returns 32 bytes
        assert_eq!(hash.as_bytes().len(), 32);
        assert!(!hash.is_zero());
    }

    #[test]
    fn test_keccak256_136_bytes() {
        // 136 bytes = keccak256 rate (r), boundary case
        let data = [0xab; 136];
        let hash = keccak256(&data);
        assert_eq!(hash.as_bytes().len(), 32);
    }

    #[test]
    fn test_keccak256_137_bytes() {
        // 137 bytes = rate + 1, spans two blocks
        let data = [0xab; 137];
        let hash = keccak256(&data);
        assert_eq!(hash.as_bytes().len(), 32);
    }

    #[test]
    fn test_keccak256_1024_bytes() {
        // Larger input
        let data = vec![0xcd; 1024];
        let hash = keccak256(&data);
        assert_eq!(hash.as_bytes().len(), 32);
    }

    #[test]
    fn test_keccak256_large_input() {
        // 1 MB of data
        let data = vec![0x42; 1024 * 1024];
        let hash = keccak256(&data);
        assert_eq!(hash.as_bytes().len(), 32);
        assert!(!hash.is_zero());
    }

    // ==================== Determinism tests ====================

    #[test]
    fn test_keccak256_deterministic() {
        let data = b"test data for determinism";
        let hash1 = keccak256(data);
        let hash2 = keccak256(data);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_keccak256_different_inputs() {
        let hash1 = keccak256(b"input1");
        let hash2 = keccak256(b"input2");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_keccak256_input_sensitivity() {
        // Single bit difference should produce completely different hash
        let hash1 = keccak256(&[0x00]);
        let hash2 = keccak256(&[0x01]);
        assert_ne!(hash1, hash2);

        // Count differing bytes - should be high (avalanche effect)
        let diff_count = hash1.as_bytes()
            .iter()
            .zip(hash2.as_bytes().iter())
            .filter(|(a, b)| a != b)
            .count();
        assert!(diff_count > 20, "Avalanche effect: {} bytes differ", diff_count);
    }

    // ==================== Ethereum-specific hashes ====================

    #[test]
    fn test_keccak256_ethereum_signature() {
        // keccak256("transfer(address,uint256)") - ERC20 transfer selector
        let hash = keccak256(b"transfer(address,uint256)");
        // First 4 bytes = 0xa9059cbb (function selector)
        assert_eq!(hash.as_bytes()[0], 0xa9);
        assert_eq!(hash.as_bytes()[1], 0x05);
        assert_eq!(hash.as_bytes()[2], 0x9c);
        assert_eq!(hash.as_bytes()[3], 0xbb);
    }

    #[test]
    fn test_keccak256_erc20_approve_selector() {
        // keccak256("approve(address,uint256)")
        let hash = keccak256(b"approve(address,uint256)");
        // First 4 bytes = 0x095ea7b3
        assert_eq!(hash.as_bytes()[0], 0x09);
        assert_eq!(hash.as_bytes()[1], 0x5e);
        assert_eq!(hash.as_bytes()[2], 0xa7);
        assert_eq!(hash.as_bytes()[3], 0xb3);
    }

    #[test]
    fn test_keccak256_erc20_balanceof_selector() {
        // keccak256("balanceOf(address)")
        let hash = keccak256(b"balanceOf(address)");
        // First 4 bytes = 0x70a08231
        assert_eq!(hash.as_bytes()[0], 0x70);
        assert_eq!(hash.as_bytes()[1], 0xa0);
        assert_eq!(hash.as_bytes()[2], 0x82);
        assert_eq!(hash.as_bytes()[3], 0x31);
    }

    #[test]
    fn test_keccak256_empty_rlp_list() {
        // keccak256(0xc0) - note: 0xc0 is RLP([]) but the hash here is just of the byte
        // The well-known empty trie root 0x56e81f... is actually keccak256(RLP(""))
        // which is different. Here we just test keccak256 of the single byte 0xc0.
        let hash = keccak256(&[0xc0]);
        assert_eq!(
            hash.to_hex(),
            "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347"
        );
    }

    // ==================== Hex encoded input ====================

    #[test]
    fn test_keccak256_hex_input() {
        // Hash of 0xdeadbeef
        let data = hex::decode("deadbeef").unwrap();
        let hash = keccak256(&data);
        assert_eq!(
            hash.to_hex(),
            "0xd4fd4e189132273036449fc9e11198c739161b4c0116a9a2dccdfa1c492006f1"
        );
    }
}
