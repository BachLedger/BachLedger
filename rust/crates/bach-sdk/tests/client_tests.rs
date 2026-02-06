//! Client integration tests for bach-sdk
//!
//! Tests client creation, configuration, and RPC method wrappers.

use bach_sdk::{BachClient, MockTransport, Address, U256};
use bach_sdk::types::{BlockId, CallRequest};
use serde_json::Value;

// ==================== Client Creation Tests ====================

#[tokio::test]
async fn test_client_new_mock() {
    let client = BachClient::new_mock();
    let chain_id = client.chain_id().await.unwrap();
    assert_eq!(chain_id, 1);
}

#[tokio::test]
async fn test_client_with_custom_transport() {
    let transport = MockTransport::new();
    let client = BachClient::with_transport(transport);
    // Client should work with custom transport
    let chain_id = client.chain_id().await.unwrap();
    assert_eq!(chain_id, 1);
}

#[tokio::test]
async fn test_client_with_custom_chain_id() {
    let transport = MockTransport::new();
    transport.set_response("eth_chainId", Value::String("0x5".to_string()));
    let client = BachClient::with_transport(transport);

    let chain_id = client.chain_id().await.unwrap();
    assert_eq!(chain_id, 5); // Goerli
}

// ==================== Chain Info Tests ====================

#[tokio::test]
async fn test_get_chain_id() {
    let client = BachClient::new_mock();
    let chain_id = client.chain_id().await.unwrap();
    assert_eq!(chain_id, 1);
}

#[tokio::test]
async fn test_get_gas_price() {
    let client = BachClient::new_mock();
    let gas_price = client.gas_price().await.unwrap();
    assert_eq!(gas_price, 1_000_000_000); // 1 gwei (mock default)
}

#[tokio::test]
async fn test_get_block_number() {
    let client = BachClient::new_mock();
    let block_number = client.block_number().await.unwrap();
    assert_eq!(block_number, 256); // Mock default
}

// ==================== Account Query Tests ====================

#[tokio::test]
async fn test_get_balance() {
    let client = BachClient::new_mock();
    let balance = client
        .get_balance(&Address::ZERO, BlockId::Latest)
        .await
        .unwrap();
    assert_eq!(balance, U256::from(1_000_000_000_000_000_000u128)); // 1 ETH (mock default)
}

#[tokio::test]
async fn test_get_balance_with_address() {
    let client = BachClient::new_mock();
    let addr = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d").unwrap();
    let balance = client.get_balance(&addr, BlockId::Latest).await.unwrap();
    // Mock returns same balance for any address
    assert_eq!(balance, U256::from(1_000_000_000_000_000_000u128));
}

#[tokio::test]
async fn test_get_balance_at_block_number() {
    let client = BachClient::new_mock();
    let balance = client
        .get_balance(&Address::ZERO, BlockId::Number(100))
        .await
        .unwrap();
    assert_eq!(balance, U256::from(1_000_000_000_000_000_000u128));
}

#[tokio::test]
async fn test_get_nonce() {
    let client = BachClient::new_mock();
    let nonce = client
        .get_nonce(&Address::ZERO, BlockId::Latest)
        .await
        .unwrap();
    assert_eq!(nonce, 0); // Mock default
}

#[tokio::test]
async fn test_get_code() {
    let client = BachClient::new_mock();
    let code = client
        .get_code(&Address::ZERO, BlockId::Latest)
        .await
        .unwrap();
    // Mock returns empty code by default
    assert!(code.is_empty());
}

// ==================== Gas Estimation Tests ====================

#[tokio::test]
async fn test_estimate_gas_simple_transfer() {
    let client = BachClient::new_mock();
    let request = CallRequest {
        to: Some(Address::ZERO),
        ..Default::default()
    };
    let gas = client.estimate_gas(&request).await.unwrap();
    assert_eq!(gas, 21000); // Mock default for simple transfer
}

#[tokio::test]
async fn test_estimate_gas_with_data() {
    let client = BachClient::new_mock();
    let request = CallRequest {
        to: Some(Address::ZERO),
        data: Some(bytes::Bytes::from(vec![0xa9, 0x05, 0x9c, 0xbb])),
        ..Default::default()
    };
    let gas = client.estimate_gas(&request).await.unwrap();
    assert_eq!(gas, 21000); // Mock returns fixed value
}

// ==================== Call Tests ====================

#[tokio::test]
async fn test_eth_call() {
    let transport = MockTransport::new();
    transport.set_response("eth_call", Value::String("0x1234".to_string()));
    let client = BachClient::with_transport(transport);

    let request = CallRequest {
        to: Some(Address::ZERO),
        data: Some(bytes::Bytes::from(vec![0x01, 0x02])),
        ..Default::default()
    };
    let result = client.call(&request, BlockId::Latest).await.unwrap();
    assert_eq!(result.as_ref(), &[0x12, 0x34]);
}

#[tokio::test]
async fn test_eth_call_empty_result() {
    let transport = MockTransport::new();
    transport.set_response("eth_call", Value::String("0x".to_string()));
    let client = BachClient::with_transport(transport);

    let request = CallRequest::default();
    let result = client.call(&request, BlockId::Latest).await.unwrap();
    assert!(result.is_empty());
}

// ==================== Transaction Tests ====================

#[tokio::test]
async fn test_send_raw_transaction() {
    let client = BachClient::new_mock();
    let raw_tx = vec![0xf8, 0x65, 0x80]; // Dummy RLP data
    let pending = client.send_raw_transaction(&raw_tx).await.unwrap();

    // Mock returns a fixed hash
    assert!(!pending.hash().is_zero());
}

// ==================== Block Query Tests ====================

#[tokio::test]
async fn test_get_block_latest() {
    // Mock doesn't have eth_getBlockByNumber by default, so we add it
    let transport = MockTransport::new();
    transport.set_response("eth_getBlockByNumber", Value::Null);
    let client = BachClient::with_transport(transport);

    let block = client.get_block(BlockId::Latest).await.unwrap();
    // Current implementation returns None (TODO in client)
    assert!(block.is_none());
}

#[tokio::test]
async fn test_get_block_by_number() {
    let transport = MockTransport::new();
    transport.set_response("eth_getBlockByNumber", Value::Null);
    let client = BachClient::with_transport(transport);

    let block = client.get_block(BlockId::Number(100)).await.unwrap();
    assert!(block.is_none()); // TODO: Will be Some when parsing is implemented
}

// ==================== BlockId Tests ====================

#[tokio::test]
async fn test_block_id_latest() {
    let client = BachClient::new_mock();
    let _ = client.get_balance(&Address::ZERO, BlockId::Latest).await.unwrap();
}

#[tokio::test]
async fn test_block_id_pending() {
    let client = BachClient::new_mock();
    let _ = client.get_balance(&Address::ZERO, BlockId::Pending).await.unwrap();
}

#[tokio::test]
async fn test_block_id_earliest() {
    let client = BachClient::new_mock();
    let _ = client.get_balance(&Address::ZERO, BlockId::Earliest).await.unwrap();
}

#[tokio::test]
async fn test_block_id_number() {
    let client = BachClient::new_mock();
    let _ = client.get_balance(&Address::ZERO, BlockId::Number(12345)).await.unwrap();
}

// ==================== Custom Mock Response Tests ====================

#[tokio::test]
async fn test_mock_custom_balance() {
    let transport = MockTransport::new();
    // Set a custom balance: 100 ETH
    transport.set_response(
        "eth_getBalance",
        Value::String("0x56bc75e2d63100000".to_string()), // 100 ETH
    );
    let client = BachClient::with_transport(transport);

    let balance = client
        .get_balance(&Address::ZERO, BlockId::Latest)
        .await
        .unwrap();
    assert_eq!(balance, U256::from(100_000_000_000_000_000_000u128));
}

#[tokio::test]
async fn test_mock_custom_nonce() {
    let transport = MockTransport::new();
    transport.set_response("eth_getTransactionCount", Value::String("0xa".to_string()));
    let client = BachClient::with_transport(transport);

    let nonce = client
        .get_nonce(&Address::ZERO, BlockId::Latest)
        .await
        .unwrap();
    assert_eq!(nonce, 10);
}

#[tokio::test]
async fn test_mock_custom_block_number() {
    let transport = MockTransport::new();
    transport.set_response("eth_blockNumber", Value::String("0xf4240".to_string())); // 1000000
    let client = BachClient::with_transport(transport);

    let block_number = client.block_number().await.unwrap();
    assert_eq!(block_number, 1_000_000);
}

// ==================== Error Handling Tests ====================

#[tokio::test]
async fn test_rpc_method_not_found() {
    let transport = MockTransport::new();
    let client = BachClient::with_transport(transport);

    // eth_getBlockByNumber is not in mock defaults, so it should fail
    let result = client.get_block(BlockId::Latest).await;
    assert!(result.is_err());

    if let Err(bach_sdk::SdkError::Rpc { code, message }) = result {
        assert_eq!(code, -32601);
        assert!(message.contains("Method not found"));
    } else {
        panic!("Expected Rpc error");
    }
}
