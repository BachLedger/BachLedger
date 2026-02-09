//! Tests for Keccak-256 hash functions
//!
//! Test-driven development: these tests are written BEFORE implementation.
//! All tests should FAIL until implementation is complete.

use bach_crypto::{keccak256, keccak256_concat};
use bach_primitives::H256;

// =============================================================================
// keccak256 tests
// =============================================================================

mod keccak256_tests {
    use super::*;

    #[test]
    fn empty_input() {
        // keccak256("") is a well-known test vector
        // Expected: 0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470
        let result = keccak256(&[]);
        let expected = H256::from_hex("0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn single_byte_zero() {
        // keccak256([0x00])
        // Expected: 0xbc36789e7a1e281436464229828f817d6612f7b477d66591ff96a9e064bcc98a
        let result = keccak256(&[0x00]);
        let expected = H256::from_hex("0xbc36789e7a1e281436464229828f817d6612f7b477d66591ff96a9e064bcc98a").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn hello_world() {
        // keccak256("hello world") - ASCII bytes
        // Note: This is NOT the same as keccak256("hello world\n")
        let input = b"hello world";
        let result = keccak256(input);
        // Pre-computed expected value
        let expected = H256::from_hex("0x47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn abc() {
        // keccak256("abc") - standard test vector
        let input = b"abc";
        let result = keccak256(input);
        let expected = H256::from_hex("0x4e03657aea45a94fc7d47ba826c8d667c0d1e6e33a64a036ec44f58fa12d6c45").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn deterministic() {
        // Same input should always produce same output
        let input = b"test input for determinism";
        let result1 = keccak256(input);
        let result2 = keccak256(input);
        assert_eq!(result1, result2);
    }

    #[test]
    fn different_inputs_different_outputs() {
        let result1 = keccak256(b"input1");
        let result2 = keccak256(b"input2");
        assert_ne!(result1, result2);
    }

    #[test]
    fn single_bit_change_affects_output() {
        // Changing a single bit should completely change the hash
        let input1 = [0x00u8; 32];
        let mut input2 = [0x00u8; 32];
        input2[0] = 0x01; // Change one bit

        let result1 = keccak256(&input1);
        let result2 = keccak256(&input2);
        assert_ne!(result1, result2);
    }

    #[test]
    fn long_input() {
        // Test with a longer input (1KB of zeros)
        let input = vec![0u8; 1024];
        let result = keccak256(&input);
        // Should produce a valid 32-byte hash
        assert!(!result.is_zero()); // Very unlikely to be zero
    }

    #[test]
    fn result_is_32_bytes() {
        let result = keccak256(b"any input");
        assert_eq!(result.as_bytes().len(), 32);
    }

    #[test]
    fn ethereum_address_style_input() {
        // Test with 20-byte address-like input
        let input = [0xdeu8, 0xad, 0xbe, 0xef, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55,
                     0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff];
        let result = keccak256(&input);
        assert!(!result.is_zero());
    }

    #[test]
    fn all_ones_input() {
        let input = [0xffu8; 32];
        let result = keccak256(&input);
        // Pre-computed for verification
        // The actual value depends on implementation - we're testing it's consistent
        let result2 = keccak256(&input);
        assert_eq!(result, result2);
    }

    #[test]
    fn sequential_bytes() {
        let input: Vec<u8> = (0u8..=255).collect();
        let result = keccak256(&input);
        assert!(!result.is_zero());
    }

    #[test]
    fn hash_of_hash() {
        // Hash the hash of empty input
        let first = keccak256(&[]);
        let second = keccak256(first.as_bytes());
        assert_ne!(first, second);
    }
}

// =============================================================================
// keccak256_concat tests
// =============================================================================

mod keccak256_concat_tests {
    use super::*;

    #[test]
    fn empty_slices() {
        // No slices should equal hash of empty input
        let result = keccak256_concat(&[]);
        let expected = keccak256(&[]);
        assert_eq!(result, expected);
    }

    #[test]
    fn single_empty_slice() {
        let result = keccak256_concat(&[&[]]);
        let expected = keccak256(&[]);
        assert_eq!(result, expected);
    }

    #[test]
    fn single_slice() {
        let data = b"hello";
        let result = keccak256_concat(&[data.as_slice()]);
        let expected = keccak256(data);
        assert_eq!(result, expected);
    }

    #[test]
    fn two_slices_equals_concatenated() {
        let slice1 = b"hello";
        let slice2 = b"world";

        let result = keccak256_concat(&[slice1.as_slice(), slice2.as_slice()]);

        // Should equal keccak256("helloworld")
        let expected = keccak256(b"helloworld");
        assert_eq!(result, expected);
    }

    #[test]
    fn three_slices() {
        let slice1 = b"one";
        let slice2 = b"two";
        let slice3 = b"three";

        let result = keccak256_concat(&[slice1.as_slice(), slice2.as_slice(), slice3.as_slice()]);
        let expected = keccak256(b"onetwothree");
        assert_eq!(result, expected);
    }

    #[test]
    fn order_matters() {
        let slice1 = b"first";
        let slice2 = b"second";

        let result1 = keccak256_concat(&[slice1.as_slice(), slice2.as_slice()]);
        let result2 = keccak256_concat(&[slice2.as_slice(), slice1.as_slice()]);

        assert_ne!(result1, result2);
    }

    #[test]
    fn empty_slice_in_middle() {
        let slice1 = b"hello";
        let slice2: &[u8] = &[];
        let slice3 = b"world";

        let result = keccak256_concat(&[slice1.as_slice(), slice2, slice3.as_slice()]);
        let expected = keccak256(b"helloworld");
        assert_eq!(result, expected);
    }

    #[test]
    fn multiple_empty_slices() {
        let result = keccak256_concat(&[&[], &[], &[]]);
        let expected = keccak256(&[]);
        assert_eq!(result, expected);
    }

    #[test]
    fn single_byte_slices() {
        let a: &[u8] = &[0x01];
        let b: &[u8] = &[0x02];
        let c: &[u8] = &[0x03];

        let result = keccak256_concat(&[a, b, c]);
        let expected = keccak256(&[0x01, 0x02, 0x03]);
        assert_eq!(result, expected);
    }

    #[test]
    fn binary_data_slices() {
        let tx_hash = [0xaau8; 32];
        let block_hash = [0xbbu8; 32];

        let result = keccak256_concat(&[&tx_hash, &block_hash]);

        let mut combined = [0u8; 64];
        combined[..32].copy_from_slice(&tx_hash);
        combined[32..].copy_from_slice(&block_hash);
        let expected = keccak256(&combined);

        assert_eq!(result, expected);
    }

    #[test]
    fn mixed_length_slices() {
        let short = b"a";
        let medium = b"medium";
        let long = b"this is a longer slice of bytes";

        let result = keccak256_concat(&[short.as_slice(), medium.as_slice(), long.as_slice()]);

        let combined = b"amediumthis is a longer slice of bytes";
        let expected = keccak256(combined);
        assert_eq!(result, expected);
    }

    #[test]
    fn many_slices() {
        let slices: Vec<&[u8]> = (0..100).map(|i| {
            // Each slice is a single byte
            // We need to be careful here - using a box to ensure lifetime
            Box::leak(Box::new([i as u8])) as &[u8]
        }).collect();

        let result = keccak256_concat(&slices);

        let combined: Vec<u8> = (0..100).map(|i| i as u8).collect();
        let expected = keccak256(&combined);
        assert_eq!(result, expected);
    }
}

// =============================================================================
// Thread safety tests
// =============================================================================

mod thread_safety {
    use super::*;

    #[test]
    fn keccak256_can_be_called_from_multiple_threads() {
        use std::thread;

        let handles: Vec<_> = (0..4).map(|i| {
            thread::spawn(move || {
                let input = format!("thread {}", i);
                keccak256(input.as_bytes())
            })
        }).collect();

        let results: Vec<H256> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        // All results should be different
        for i in 0..results.len() {
            for j in (i + 1)..results.len() {
                assert_ne!(results[i], results[j]);
            }
        }
    }
}

// =============================================================================
// Known test vectors from external sources
// =============================================================================

mod test_vectors {
    use super::*;

    #[test]
    fn nist_like_test_vector_short() {
        // Short message test
        let input = b"The quick brown fox jumps over the lazy dog";
        let result = keccak256(input);
        let expected = H256::from_hex("0x4d741b6f1eb29cb2a9b9911c82f56fa8d73b04959d3d9d222895df6c0b28aa15").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn nist_like_test_vector_with_period() {
        // Same as above but with a period - completely different hash
        let input = b"The quick brown fox jumps over the lazy dog.";
        let result = keccak256(input);
        let expected = H256::from_hex("0x578951e24efd62a3d63a86f7cd19aaa53c898fe287d2552133220370240b572d").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn ethereum_transfer_encoding_style() {
        // This mimics what you might hash for an Ethereum transaction
        // transfer(address,uint256) = 0xa9059cbb...
        let function_selector = hex::decode("a9059cbb").unwrap();
        let result = keccak256(&function_selector);
        assert!(!result.is_zero());
    }

    #[test]
    fn solidity_function_signature() {
        // keccak256("transfer(address,uint256)") should give 0xa9059cbb...
        let sig = b"transfer(address,uint256)";
        let result = keccak256(sig);
        // First 4 bytes should be a9059cbb
        let bytes = result.as_bytes();
        assert_eq!(bytes[0], 0xa9);
        assert_eq!(bytes[1], 0x05);
        assert_eq!(bytes[2], 0x9c);
        assert_eq!(bytes[3], 0xbb);
    }

    #[test]
    fn solidity_event_signature() {
        // keccak256("Transfer(address,address,uint256)")
        let sig = b"Transfer(address,address,uint256)";
        let result = keccak256(sig);
        let expected = H256::from_hex("0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef").unwrap();
        assert_eq!(result, expected);
    }
}
