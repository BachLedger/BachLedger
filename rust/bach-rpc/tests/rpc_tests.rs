//! Comprehensive RPC tests for bach-rpc
//!
//! Test-Driven Development: These tests define expected JSON-RPC behavior.
//! Implementation must pass all tests.
//!
//! Test categories:
//! 1. Transaction APIs: sendTransaction, sendRawTransaction
//! 2. Query APIs: call, getBalance, getStorageAt, getCode
//! 3. Block APIs: getBlockByNumber, getBlockByHash
//! 4. Receipt APIs: getTransactionReceipt, getLogs
//! 5. Error Handling: Invalid params, not found

use bach_rpc::{
    EthApi, EthApiError, RpcServer, RpcConfig,
    // Request/Response types
    SendTransactionRequest, CallRequest, LogFilter,
    TransactionReceipt as RpcReceipt, Block as RpcBlock,
    // Hex types
    HexU256, HexBytes, HexAddress, HexH256,
};
use bach_primitives::{Address, H256, U256};
use bach_storage::Storage;
use std::sync::Arc;

// =============================================================================
// Test Helpers
// =============================================================================

/// Creates a test address
fn test_address(seed: u8) -> Address {
    Address::from([seed; 20])
}

/// Creates a test hash
fn test_hash(seed: u8) -> H256 {
    H256::from([seed; 32])
}

/// Creates a hex-encoded address
fn hex_address(seed: u8) -> String {
    format!("0x{}", "aa".repeat(20).replace("aa", &format!("{:02x}", seed)))
}

/// Creates a hex-encoded hash
fn hex_hash(seed: u8) -> String {
    format!("0x{}", hex::encode([seed; 32]))
}

/// Creates a hex-encoded number
fn hex_u64(n: u64) -> String {
    format!("0x{:x}", n)
}

// =============================================================================
// 1. Transaction API Tests
// =============================================================================

mod transaction_apis {
    use super::*;

    #[tokio::test]
    async fn test_send_transaction_valid() {
        let api = create_test_api().await;

        let request = SendTransactionRequest {
            from: hex_address(0xaa),
            to: Some(hex_address(0xbb)),
            value: Some("0x1000".to_string()),
            data: None,
            nonce: Some("0x0".to_string()),
            gas: Some("0x5208".to_string()), // 21000
            gas_price: Some("0x3b9aca00".to_string()), // 1 gwei
        };

        let result = api.eth_send_transaction(request).await;

        // Should return transaction hash
        assert!(result.is_ok());
        let tx_hash = result.unwrap();
        assert!(tx_hash.starts_with("0x"));
        assert_eq!(tx_hash.len(), 66); // 0x + 64 hex chars
    }

    #[tokio::test]
    async fn test_send_transaction_contract_creation() {
        let api = create_test_api().await;

        // Contract creation: to = None
        let request = SendTransactionRequest {
            from: hex_address(0xaa),
            to: None, // Contract creation
            value: Some("0x0".to_string()),
            data: Some("0x6080604052".to_string()), // Simple bytecode
            nonce: Some("0x0".to_string()),
            gas: Some("0x100000".to_string()),
            gas_price: Some("0x3b9aca00".to_string()),
        };

        let result = api.eth_send_transaction(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_transaction_invalid_from() {
        let api = create_test_api().await;

        let request = SendTransactionRequest {
            from: "invalid_address".to_string(),
            to: Some(hex_address(0xbb)),
            value: None,
            data: None,
            nonce: None,
            gas: None,
            gas_price: None,
        };

        let result = api.eth_send_transaction(request).await;
        assert!(matches!(result, Err(EthApiError::InvalidParams(_))));
    }

    #[tokio::test]
    async fn test_send_raw_transaction() {
        let api = create_test_api().await;

        // RLP-encoded signed transaction
        let raw_tx = "0xf86c098504a817c800825208943535353535353535353535353535353535353535880de0b6b3a76400008025a028ef61340bd939bc2195fe537567866003e1a15d3c71ff63e1590620aa636276a067cbe9d8997f761aecb703304b3800ccf555c9f3dc64214b297fb1966a3b6d83";

        let result = api.eth_send_raw_transaction(raw_tx.to_string()).await;

        // Should return transaction hash or validation error
        // (Signature verification may fail for test data)
        match result {
            Ok(hash) => {
                assert!(hash.starts_with("0x"));
            }
            Err(EthApiError::InvalidTransaction(_)) => {
                // Expected for invalid signature
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_send_raw_transaction_invalid_hex() {
        let api = create_test_api().await;

        let result = api.eth_send_raw_transaction("not_hex".to_string()).await;
        assert!(matches!(result, Err(EthApiError::InvalidParams(_))));
    }

    #[tokio::test]
    async fn test_get_transaction_count() {
        let api = create_test_api().await;

        let result = api
            .eth_get_transaction_count(hex_address(0xaa), "latest".to_string())
            .await;

        assert!(result.is_ok());
        let count = result.unwrap();
        // Should return hex-encoded number
        assert!(count.starts_with("0x"));
    }

    #[tokio::test]
    async fn test_get_transaction_by_hash() {
        let api = create_test_api().await;

        let result = api.eth_get_transaction_by_hash(hex_hash(0x42)).await;

        // May return None if transaction doesn't exist
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_transaction_by_block_hash_and_index() {
        let api = create_test_api().await;

        let result = api
            .eth_get_transaction_by_block_hash_and_index(hex_hash(0x42), "0x0".to_string())
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_transaction_by_block_number_and_index() {
        let api = create_test_api().await;

        let result = api
            .eth_get_transaction_by_block_number_and_index("latest".to_string(), "0x0".to_string())
            .await;

        assert!(result.is_ok());
    }
}

// =============================================================================
// 2. Query API Tests
// =============================================================================

mod query_apis {
    use super::*;

    #[tokio::test]
    async fn test_call() {
        let api = create_test_api().await;

        let request = CallRequest {
            from: Some(hex_address(0xaa)),
            to: hex_address(0xbb),
            data: Some("0x70a08231".to_string()), // balanceOf selector
            value: None,
            gas: Some("0x100000".to_string()),
            gas_price: None,
        };

        let result = api.eth_call(request, "latest".to_string()).await;

        assert!(result.is_ok());
        // Should return hex-encoded return data
        let data = result.unwrap();
        assert!(data.starts_with("0x"));
    }

    #[tokio::test]
    async fn test_call_invalid_to() {
        let api = create_test_api().await;

        let request = CallRequest {
            from: None,
            to: "invalid".to_string(),
            data: None,
            value: None,
            gas: None,
            gas_price: None,
        };

        let result = api.eth_call(request, "latest".to_string()).await;
        assert!(matches!(result, Err(EthApiError::InvalidParams(_))));
    }

    #[tokio::test]
    async fn test_estimate_gas() {
        let api = create_test_api().await;

        let request = CallRequest {
            from: Some(hex_address(0xaa)),
            to: hex_address(0xbb),
            data: None,
            value: Some("0x1000".to_string()),
            gas: None,
            gas_price: None,
        };

        let result = api.eth_estimate_gas(request, None).await;

        assert!(result.is_ok());
        let gas = result.unwrap();
        assert!(gas.starts_with("0x"));
        // Simple transfer should be around 21000 gas
        let gas_value = u64::from_str_radix(&gas[2..], 16).unwrap();
        assert!(gas_value >= 21000);
    }

    #[tokio::test]
    async fn test_get_balance() {
        let api = create_test_api().await;

        let result = api
            .eth_get_balance(hex_address(0xaa), "latest".to_string())
            .await;

        assert!(result.is_ok());
        let balance = result.unwrap();
        assert!(balance.starts_with("0x"));
    }

    #[tokio::test]
    async fn test_get_balance_at_block() {
        let api = create_test_api().await;

        // Query at specific block number
        let result = api
            .eth_get_balance(hex_address(0xaa), "0x0".to_string())
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_balance_pending() {
        let api = create_test_api().await;

        let result = api
            .eth_get_balance(hex_address(0xaa), "pending".to_string())
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_storage_at() {
        let api = create_test_api().await;

        let result = api
            .eth_get_storage_at(
                hex_address(0xaa),
                "0x0".to_string(), // Slot 0
                "latest".to_string(),
            )
            .await;

        assert!(result.is_ok());
        let storage = result.unwrap();
        assert!(storage.starts_with("0x"));
        assert_eq!(storage.len(), 66); // 32 bytes = 64 hex + 0x
    }

    #[tokio::test]
    async fn test_get_storage_at_invalid_slot() {
        let api = create_test_api().await;

        let result = api
            .eth_get_storage_at(hex_address(0xaa), "invalid".to_string(), "latest".to_string())
            .await;

        assert!(matches!(result, Err(EthApiError::InvalidParams(_))));
    }

    #[tokio::test]
    async fn test_get_code() {
        let api = create_test_api().await;

        let result = api
            .eth_get_code(hex_address(0xaa), "latest".to_string())
            .await;

        assert!(result.is_ok());
        let code = result.unwrap();
        // Either empty (0x) or hex-encoded bytecode
        assert!(code.starts_with("0x"));
    }

    #[tokio::test]
    async fn test_get_code_eoa() {
        let api = create_test_api().await;

        // EOA (externally owned account) should have empty code
        let result = api
            .eth_get_code(hex_address(0xff), "latest".to_string())
            .await;

        assert!(result.is_ok());
        let code = result.unwrap();
        assert_eq!(code, "0x");
    }
}

// =============================================================================
// 3. Block API Tests
// =============================================================================

mod block_apis {
    use super::*;

    #[tokio::test]
    async fn test_get_block_by_number_latest() {
        let api = create_test_api().await;

        let result = api
            .eth_get_block_by_number("latest".to_string(), false)
            .await;

        assert!(result.is_ok());
        // May be None if no blocks yet
        if let Some(block) = result.unwrap() {
            assert!(block.number.starts_with("0x"));
            assert!(block.hash.starts_with("0x"));
        }
    }

    #[tokio::test]
    async fn test_get_block_by_number_with_txs() {
        let api = create_test_api().await;

        let result = api
            .eth_get_block_by_number("latest".to_string(), true) // full transactions
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_block_by_number_pending() {
        let api = create_test_api().await;

        let result = api
            .eth_get_block_by_number("pending".to_string(), false)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_block_by_number_earliest() {
        let api = create_test_api().await;

        let result = api
            .eth_get_block_by_number("earliest".to_string(), false)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_block_by_number_specific() {
        let api = create_test_api().await;

        let result = api.eth_get_block_by_number("0x0".to_string(), false).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_block_by_hash() {
        let api = create_test_api().await;

        let result = api.eth_get_block_by_hash(hex_hash(0x42), false).await;

        assert!(result.is_ok());
        // May be None if block doesn't exist
    }

    #[tokio::test]
    async fn test_get_block_by_hash_with_txs() {
        let api = create_test_api().await;

        let result = api.eth_get_block_by_hash(hex_hash(0x42), true).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_block_by_hash_invalid() {
        let api = create_test_api().await;

        let result = api.eth_get_block_by_hash("invalid".to_string(), false).await;

        assert!(matches!(result, Err(EthApiError::InvalidParams(_))));
    }

    #[tokio::test]
    async fn test_block_number() {
        let api = create_test_api().await;

        let result = api.eth_block_number().await;

        assert!(result.is_ok());
        let number = result.unwrap();
        assert!(number.starts_with("0x"));
    }

    #[tokio::test]
    async fn test_get_block_transaction_count_by_hash() {
        let api = create_test_api().await;

        let result = api
            .eth_get_block_transaction_count_by_hash(hex_hash(0x42))
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_block_transaction_count_by_number() {
        let api = create_test_api().await;

        let result = api
            .eth_get_block_transaction_count_by_number("latest".to_string())
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_uncle_count_by_block_hash() {
        let api = create_test_api().await;

        let result = api.eth_get_uncle_count_by_block_hash(hex_hash(0x42)).await;

        assert!(result.is_ok());
        // Should return "0x0" (no uncles in PoS)
        let count = result.unwrap();
        assert_eq!(count, "0x0");
    }

    #[tokio::test]
    async fn test_get_uncle_count_by_block_number() {
        let api = create_test_api().await;

        let result = api
            .eth_get_uncle_count_by_block_number("latest".to_string())
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "0x0");
    }
}

// =============================================================================
// 4. Receipt API Tests
// =============================================================================

mod receipt_apis {
    use super::*;

    #[tokio::test]
    async fn test_get_transaction_receipt() {
        let api = create_test_api().await;

        let result = api.eth_get_transaction_receipt(hex_hash(0x42)).await;

        assert!(result.is_ok());
        // May be None if transaction doesn't exist
    }

    #[tokio::test]
    async fn test_get_transaction_receipt_invalid_hash() {
        let api = create_test_api().await;

        let result = api
            .eth_get_transaction_receipt("invalid".to_string())
            .await;

        assert!(matches!(result, Err(EthApiError::InvalidParams(_))));
    }

    #[tokio::test]
    async fn test_get_logs_by_block_range() {
        let api = create_test_api().await;

        let filter = LogFilter {
            from_block: Some("0x0".to_string()),
            to_block: Some("latest".to_string()),
            address: None,
            topics: None,
        };

        let result = api.eth_get_logs(filter).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_logs_by_address() {
        let api = create_test_api().await;

        let filter = LogFilter {
            from_block: None,
            to_block: None,
            address: Some(hex_address(0xaa)),
            topics: None,
        };

        let result = api.eth_get_logs(filter).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_logs_by_topics() {
        let api = create_test_api().await;

        // Filter by event signature topic
        let transfer_topic = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";

        let filter = LogFilter {
            from_block: None,
            to_block: None,
            address: None,
            topics: Some(vec![Some(transfer_topic.to_string())]),
        };

        let result = api.eth_get_logs(filter).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_logs_complex_filter() {
        let api = create_test_api().await;

        // Complex filter with multiple constraints
        let filter = LogFilter {
            from_block: Some("0x0".to_string()),
            to_block: Some("0x100".to_string()),
            address: Some(hex_address(0xaa)),
            topics: Some(vec![
                Some(hex_hash(0x11)), // topic[0]
                None,                 // any topic[1]
                Some(hex_hash(0x33)), // topic[2]
            ]),
        };

        let result = api.eth_get_logs(filter).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_logs_block_hash() {
        let api = create_test_api().await;

        // Filter by specific block hash
        let filter = LogFilter {
            from_block: None,
            to_block: None,
            address: None,
            topics: None,
            // block_hash: Some(hex_hash(0x42)), // Alternative to block range
        };

        let result = api.eth_get_logs(filter).await;
        assert!(result.is_ok());
    }
}

// =============================================================================
// 5. Error Handling Tests
// =============================================================================

mod error_handling {
    use super::*;

    #[tokio::test]
    async fn test_invalid_address_format() {
        let api = create_test_api().await;

        let result = api
            .eth_get_balance("not_an_address".to_string(), "latest".to_string())
            .await;

        assert!(matches!(result, Err(EthApiError::InvalidParams(_))));
    }

    #[tokio::test]
    async fn test_invalid_block_tag() {
        let api = create_test_api().await;

        let result = api
            .eth_get_balance(hex_address(0xaa), "invalid_tag".to_string())
            .await;

        assert!(matches!(result, Err(EthApiError::InvalidParams(_))));
    }

    #[tokio::test]
    async fn test_invalid_hex_number() {
        let api = create_test_api().await;

        // Block number with invalid hex
        let result = api
            .eth_get_block_by_number("0xGGGG".to_string(), false)
            .await;

        assert!(matches!(result, Err(EthApiError::InvalidParams(_))));
    }

    #[tokio::test]
    async fn test_block_not_found() {
        let api = create_test_api().await;

        // Very high block number that doesn't exist
        let result = api
            .eth_get_block_by_number("0xffffffffffffff".to_string(), false)
            .await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none()); // Should return None, not error
    }

    #[tokio::test]
    async fn test_transaction_not_found() {
        let api = create_test_api().await;

        // Non-existent transaction
        let result = api.eth_get_transaction_by_hash(hex_hash(0xff)).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_receipt_not_found() {
        let api = create_test_api().await;

        let result = api.eth_get_transaction_receipt(hex_hash(0xff)).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}

// =============================================================================
// Web3 and Net Namespace Tests
// =============================================================================

mod web3_and_net {
    use super::*;

    #[tokio::test]
    async fn test_web3_client_version() {
        let api = create_test_api().await;

        let result = api.web3_client_version().await;

        assert!(result.is_ok());
        let version = result.unwrap();
        assert!(!version.is_empty());
        // Should contain "BachLedger" or similar identifier
    }

    #[tokio::test]
    async fn test_web3_sha3() {
        let api = create_test_api().await;

        // keccak256("hello")
        let result = api.web3_sha3("0x68656c6c6f".to_string()).await;

        assert!(result.is_ok());
        let hash = result.unwrap();
        assert!(hash.starts_with("0x"));
        assert_eq!(hash.len(), 66);

        // Known hash of "hello"
        let expected = "0x1c8aff950685c2ed4bc3174f3472287b56d9517b9c948127319a09a7a36deac8";
        assert_eq!(hash.to_lowercase(), expected);
    }

    #[tokio::test]
    async fn test_web3_sha3_invalid_hex() {
        let api = create_test_api().await;

        let result = api.web3_sha3("not_hex".to_string()).await;
        assert!(matches!(result, Err(EthApiError::InvalidParams(_))));
    }

    #[tokio::test]
    async fn test_net_version() {
        let api = create_test_api().await;

        let result = api.net_version().await;

        assert!(result.is_ok());
        let version = result.unwrap();
        // Should return chain ID as string
        assert!(!version.is_empty());
    }

    #[tokio::test]
    async fn test_net_peer_count() {
        let api = create_test_api().await;

        let result = api.net_peer_count().await;

        assert!(result.is_ok());
        let count = result.unwrap();
        assert!(count.starts_with("0x"));
    }

    #[tokio::test]
    async fn test_net_listening() {
        let api = create_test_api().await;

        let result = api.net_listening().await;

        assert!(result.is_ok());
        // Returns boolean
    }
}

// =============================================================================
// Additional Eth Namespace Tests
// =============================================================================

mod eth_additional {
    use super::*;

    #[tokio::test]
    async fn test_eth_chain_id() {
        let api = create_test_api().await;

        let result = api.eth_chain_id().await;

        assert!(result.is_ok());
        let chain_id = result.unwrap();
        assert!(chain_id.starts_with("0x"));
    }

    #[tokio::test]
    async fn test_eth_gas_price() {
        let api = create_test_api().await;

        let result = api.eth_gas_price().await;

        assert!(result.is_ok());
        let gas_price = result.unwrap();
        assert!(gas_price.starts_with("0x"));

        // Gas price should be positive
        let value = u64::from_str_radix(&gas_price[2..], 16).unwrap();
        assert!(value > 0);
    }

    #[tokio::test]
    async fn test_eth_max_priority_fee_per_gas() {
        let api = create_test_api().await;

        let result = api.eth_max_priority_fee_per_gas().await;

        assert!(result.is_ok());
        let fee = result.unwrap();
        assert!(fee.starts_with("0x"));
    }

    #[tokio::test]
    async fn test_eth_fee_history() {
        let api = create_test_api().await;

        let result = api
            .eth_fee_history(
                "0x5".to_string(),                          // 5 blocks
                "latest".to_string(),
                Some(vec![25.0, 50.0, 75.0]), // percentiles
            )
            .await;

        assert!(result.is_ok());
        let history = result.unwrap();
        assert!(history.base_fee_per_gas.len() > 0);
    }

    #[tokio::test]
    async fn test_eth_syncing() {
        let api = create_test_api().await;

        let result = api.eth_syncing().await;

        assert!(result.is_ok());
        // Returns either false or sync status object
    }

    #[tokio::test]
    async fn test_eth_coinbase() {
        let api = create_test_api().await;

        let result = api.eth_coinbase().await;

        // May return error if no coinbase set
        match result {
            Ok(coinbase) => {
                assert!(coinbase.starts_with("0x"));
                assert_eq!(coinbase.len(), 42);
            }
            Err(EthApiError::NotSupported(_)) => {
                // Acceptable if not mining
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_eth_mining() {
        let api = create_test_api().await;

        let result = api.eth_mining().await;

        assert!(result.is_ok());
        // Returns boolean
    }

    #[tokio::test]
    async fn test_eth_hashrate() {
        let api = create_test_api().await;

        let result = api.eth_hashrate().await;

        assert!(result.is_ok());
        let hashrate = result.unwrap();
        // Should return 0x0 for PoS
        assert_eq!(hashrate, "0x0");
    }

    #[tokio::test]
    async fn test_eth_accounts() {
        let api = create_test_api().await;

        let result = api.eth_accounts().await;

        assert!(result.is_ok());
        let accounts = result.unwrap();
        // May return empty list
        for account in accounts {
            assert!(account.starts_with("0x"));
            assert_eq!(account.len(), 42);
        }
    }

    #[tokio::test]
    async fn test_eth_protocol_version() {
        let api = create_test_api().await;

        let result = api.eth_protocol_version().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_eth_sign() {
        let api = create_test_api().await;

        let result = api
            .eth_sign(hex_address(0xaa), "0x68656c6c6f".to_string())
            .await;

        // May fail if account is not unlocked
        match result {
            Ok(signature) => {
                assert!(signature.starts_with("0x"));
                assert_eq!(signature.len(), 132); // 65 bytes = 130 hex + 0x
            }
            Err(EthApiError::AccountLocked(_)) => {
                // Expected if no local signing
            }
            Err(EthApiError::NotSupported(_)) => {
                // Expected if signing not supported
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }
}

// =============================================================================
// Server Tests
// =============================================================================

mod server {
    use super::*;

    #[tokio::test]
    async fn test_rpc_config_defaults() {
        let config = RpcConfig::default();

        assert_eq!(config.port, 8545);
        assert_eq!(config.host, "127.0.0.1");
    }

    #[tokio::test]
    async fn test_rpc_config_builder() {
        let config = RpcConfig::default()
            .with_port(9545)
            .with_host("0.0.0.0".to_string())
            .with_max_connections(100);

        assert_eq!(config.port, 9545);
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.max_connections, 100);
    }

    #[tokio::test]
    async fn test_server_creation() {
        let config = RpcConfig::default().with_port(0); // Random port
        let api = create_test_api().await;

        let server = RpcServer::new(config, api);
        assert!(server.is_ok());
    }
}

// =============================================================================
// Hex Encoding Tests
// =============================================================================

mod hex_encoding {
    use super::*;

    #[test]
    fn test_hex_u256_from_str() {
        let hex = HexU256::from_str("0x1000").unwrap();
        assert_eq!(hex.value(), U256::from_u64(4096));
    }

    #[test]
    fn test_hex_u256_to_string() {
        let hex = HexU256::from(U256::from_u64(4096));
        assert_eq!(hex.to_string(), "0x1000");
    }

    #[test]
    fn test_hex_u256_zero() {
        let hex = HexU256::from(U256::ZERO);
        assert_eq!(hex.to_string(), "0x0");
    }

    #[test]
    fn test_hex_bytes_from_str() {
        let hex = HexBytes::from_str("0x1234").unwrap();
        assert_eq!(hex.bytes(), &[0x12, 0x34]);
    }

    #[test]
    fn test_hex_bytes_empty() {
        let hex = HexBytes::from_str("0x").unwrap();
        assert!(hex.bytes().is_empty());
    }

    #[test]
    fn test_hex_address_from_str() {
        let hex = HexAddress::from_str(&hex_address(0xaa)).unwrap();
        assert_eq!(hex.address(), test_address(0xaa));
    }

    #[test]
    fn test_hex_h256_from_str() {
        let hex = HexH256::from_str(&hex_hash(0xab)).unwrap();
        assert_eq!(hex.hash(), test_hash(0xab));
    }

    #[test]
    fn test_invalid_hex_prefix() {
        let result = HexU256::from_str("1000"); // Missing 0x
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_hex_chars() {
        let result = HexU256::from_str("0xGGGG");
        assert!(result.is_err());
    }

    #[test]
    fn test_address_wrong_length() {
        let result = HexAddress::from_str("0x1234"); // Too short
        assert!(result.is_err());
    }
}

// =============================================================================
// Test API Factory
// =============================================================================

/// Creates a test API instance with mock storage
async fn create_test_api() -> impl EthApi {
    // Create temporary storage for tests
    let temp_dir = tempfile::tempdir().unwrap();
    let storage = Storage::open(temp_dir.path()).unwrap();

    // Create API with mock implementations
    MockEthApi::new(Arc::new(storage))
}

/// Mock implementation of EthApi for testing
struct MockEthApi {
    storage: Arc<Storage>,
}

impl MockEthApi {
    fn new(storage: Arc<Storage>) -> Self {
        Self { storage }
    }
}

// Note: This test file assumes the EthApi trait and related types exist.
// The actual implementation would provide these types.
// For now, this file serves as a specification of expected behavior.
