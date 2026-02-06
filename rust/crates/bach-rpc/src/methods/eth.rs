//! Ethereum namespace RPC methods (eth_*)

use std::sync::Arc;

use bach_crypto::{keccak256, public_key_to_address, recover_public_key, Signature};
use bach_primitives::{Address, H256};
use bach_rlp::{Rlp, RlpStream};
use bach_storage::StateReader;
use bach_types::{LegacyTx, SignedTransaction, TxSignature};
use bytes::Bytes;
use serde_json::Value;

use crate::error::JsonRpcError;
use crate::handler::RpcContext;
use crate::types::{
    format_bytes, format_u128, format_u64, parse_address, parse_block_id, parse_h256,
    parse_hex_bytes, BlockId, CallRequest, CallRequestRaw, RpcBlock, RpcTransaction,
};

/// eth_chainId - Returns the chain ID
pub async fn eth_chain_id(
    ctx: Arc<RpcContext>,
    _params: Vec<Value>,
) -> Result<Value, JsonRpcError> {
    Ok(Value::String(format_u64(ctx.chain_id)))
}

/// eth_blockNumber - Returns the current block number
pub async fn eth_block_number(
    ctx: Arc<RpcContext>,
    _params: Vec<Value>,
) -> Result<Value, JsonRpcError> {
    let latest = ctx
        .block_db
        .get_latest_block()
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?
        .unwrap_or(0);

    Ok(Value::String(format_u64(latest)))
}

/// eth_gasPrice - Returns the current gas price
pub async fn eth_gas_price(
    ctx: Arc<RpcContext>,
    _params: Vec<Value>,
) -> Result<Value, JsonRpcError> {
    Ok(Value::String(format_u64(ctx.get_gas_price())))
}

/// eth_getBalance - Returns the balance of an account
pub async fn eth_get_balance(
    ctx: Arc<RpcContext>,
    params: Vec<Value>,
) -> Result<Value, JsonRpcError> {
    if params.is_empty() {
        return Err(JsonRpcError::invalid_params("missing address parameter"));
    }

    let address = parse_address(&params[0])?;
    let _block_id = if params.len() > 1 {
        parse_block_id(&params[1])?
    } else {
        BlockId::Latest
    };

    let balance = ctx
        .state_db
        .get_balance(&address)
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;

    Ok(Value::String(format_u128(balance)))
}

/// eth_getTransactionCount - Returns the nonce of an account
pub async fn eth_get_transaction_count(
    ctx: Arc<RpcContext>,
    params: Vec<Value>,
) -> Result<Value, JsonRpcError> {
    if params.is_empty() {
        return Err(JsonRpcError::invalid_params("missing address parameter"));
    }

    let address = parse_address(&params[0])?;
    let block_id = if params.len() > 1 {
        parse_block_id(&params[1])?
    } else {
        BlockId::Latest
    };

    // For pending, include txpool nonces
    if block_id == BlockId::Pending {
        let pool_nonce = ctx.txpool.get_nonce(&address);
        if pool_nonce > 0 {
            return Ok(Value::String(format_u64(pool_nonce)));
        }
    }

    let nonce = ctx
        .state_db
        .get_nonce(&address)
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;

    Ok(Value::String(format_u64(nonce)))
}

/// eth_getCode - Returns the code at an address
pub async fn eth_get_code(
    ctx: Arc<RpcContext>,
    params: Vec<Value>,
) -> Result<Value, JsonRpcError> {
    if params.is_empty() {
        return Err(JsonRpcError::invalid_params("missing address parameter"));
    }

    let address = parse_address(&params[0])?;
    let _block_id = if params.len() > 1 {
        parse_block_id(&params[1])?
    } else {
        BlockId::Latest
    };

    let code_hash = ctx
        .state_db
        .get_code_hash(&address)
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;

    if code_hash.is_zero() || code_hash == bach_storage::EMPTY_CODE_HASH {
        return Ok(Value::String("0x".to_string()));
    }

    let code = ctx
        .state_db
        .get_code(&code_hash)
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?
        .unwrap_or_default();

    Ok(Value::String(format_bytes(&code)))
}

/// eth_getStorageAt - Returns storage value at a position
pub async fn eth_get_storage_at(
    ctx: Arc<RpcContext>,
    params: Vec<Value>,
) -> Result<Value, JsonRpcError> {
    if params.len() < 2 {
        return Err(JsonRpcError::invalid_params(
            "missing address or position parameter",
        ));
    }

    let address = parse_address(&params[0])?;
    let position = parse_h256(&params[1])?;
    let _block_id = if params.len() > 2 {
        parse_block_id(&params[2])?
    } else {
        BlockId::Latest
    };

    let value = ctx
        .state_db
        .get_storage(&address, &position)
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;

    Ok(Value::String(value.to_hex()))
}

/// eth_call - Execute a call without creating a transaction
pub async fn eth_call(
    ctx: Arc<RpcContext>,
    params: Vec<Value>,
) -> Result<Value, JsonRpcError> {
    if params.is_empty() {
        return Err(JsonRpcError::invalid_params("missing call request"));
    }

    let call_request_raw: CallRequestRaw = serde_json::from_value(params[0].clone())
        .map_err(|e| JsonRpcError::invalid_params(format!("invalid call request: {}", e)))?;
    let call_request = CallRequest::from_raw(call_request_raw)?;

    let _block_id = if params.len() > 1 {
        parse_block_id(&params[1])?
    } else {
        BlockId::Latest
    };

    // Get latest block for context
    let latest_number = ctx
        .block_db
        .get_latest_block()
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?
        .unwrap_or(0);

    // Create execution environment
    let from = call_request.from.unwrap_or(Address::ZERO);
    let to = call_request.to.unwrap_or(Address::ZERO);
    let data = call_request.data.map(|b| b.to_vec()).unwrap_or_default();
    let gas = call_request.gas.unwrap_or(30_000_000);
    let value = call_request
        .value
        .map(|v| v.low_u128())
        .unwrap_or(0);

    // Build block context
    let block_ctx = bach_evm::BlockContext {
        number: latest_number + 1,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        gas_limit: 30_000_000,
        coinbase: Address::ZERO,
        prevrandao: H256::ZERO,
        chain_id: ctx.chain_id,
        base_fee: ctx.get_gas_price() as u128,
    };

    let call_ctx = bach_evm::CallContext {
        caller: from,
        address: to,
        value,
        data: data.clone(),
        gas,
        is_static: false,
        depth: 0,
    };

    let tx_ctx = bach_evm::TxContext {
        origin: from,
        gas_price: ctx.get_gas_price() as u128,
    };

    let env = bach_evm::Environment::new(call_ctx, block_ctx, tx_ctx);

    // Get contract code
    let code_hash = ctx
        .state_db
        .get_code_hash(&to)
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;

    let code = if code_hash.is_zero() || code_hash == bach_storage::EMPTY_CODE_HASH {
        vec![]
    } else {
        ctx.state_db
            .get_code(&code_hash)
            .map_err(|e| JsonRpcError::internal_error(e.to_string()))?
            .unwrap_or_default()
    };

    if code.is_empty() {
        // No code to execute, return empty
        return Ok(Value::String("0x".to_string()));
    }

    // Execute
    let mut interp = bach_evm::Interpreter::new(code, gas);
    let result = interp.run(&env);

    if result.success {
        Ok(Value::String(format_bytes(&result.output)))
    } else {
        // Include revert data in error response for debugging
        let message = if result.output.is_empty() {
            "execution reverted".to_string()
        } else {
            format!("execution reverted: {}", format_bytes(&result.output))
        };
        Err(JsonRpcError::execution_error(message))
    }
}

/// eth_estimateGas - Estimate gas for a transaction
pub async fn eth_estimate_gas(
    ctx: Arc<RpcContext>,
    params: Vec<Value>,
) -> Result<Value, JsonRpcError> {
    if params.is_empty() {
        return Err(JsonRpcError::invalid_params("missing call request"));
    }

    let call_request_raw: CallRequestRaw = serde_json::from_value(params[0].clone())
        .map_err(|e| JsonRpcError::invalid_params(format!("invalid call request: {}", e)))?;
    let call_request = CallRequest::from_raw(call_request_raw)?;

    // Simple estimation: base gas + data gas
    let base_gas = 21000u64;
    let data_gas = call_request
        .data
        .as_ref()
        .map(|d| {
            d.iter()
                .map(|&b| if b == 0 { 4u64 } else { 16u64 })
                .sum::<u64>()
        })
        .unwrap_or(0);

    // If it's a contract call, add execution overhead
    let to = call_request.to.unwrap_or(Address::ZERO);
    let code_hash = ctx
        .state_db
        .get_code_hash(&to)
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;

    let execution_gas = if !code_hash.is_zero() && code_hash != bach_storage::EMPTY_CODE_HASH {
        50_000u64 // Add overhead for contract execution
    } else {
        0
    };

    let estimated = base_gas + data_gas + execution_gas;

    // Add 20% buffer
    let with_buffer = estimated + estimated / 5;

    Ok(Value::String(format_u64(with_buffer)))
}

/// eth_sendRawTransaction - Submit a raw transaction
pub async fn eth_send_raw_transaction(
    ctx: Arc<RpcContext>,
    params: Vec<Value>,
) -> Result<Value, JsonRpcError> {
    if params.is_empty() {
        return Err(JsonRpcError::invalid_params(
            "missing raw transaction data",
        ));
    }

    let raw_tx = parse_hex_bytes(&params[0])?;

    // Decode the transaction
    let signed_tx = decode_raw_transaction(&raw_tx)
        .map_err(|e| JsonRpcError::invalid_params(format!("failed to decode transaction: {}", e)))?;

    // Recover sender
    let tx_hash = compute_tx_hash(&raw_tx);
    let signing_hash = compute_signing_hash(&signed_tx, ctx.chain_id)?;

    let r: [u8; 32] = *signed_tx.signature.r.as_bytes();
    let s: [u8; 32] = *signed_tx.signature.s.as_bytes();

    let sig = Signature {
        v: (signed_tx.signature.v % 256) as u8,
        r,
        s,
    };

    let pubkey = recover_public_key(&signing_hash, &sig)
        .map_err(|e| JsonRpcError::invalid_params(format!("failed to recover sender: {}", e)))?;
    let sender = public_key_to_address(&pubkey);

    // Validate sender balance and nonce
    let sender_balance = ctx
        .state_db
        .get_balance(&sender)
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;

    let tx_cost = signed_tx.value() as u128 + (signed_tx.gas_limit() as u128 * signed_tx.gas_price().unwrap_or(ctx.get_gas_price() as u128));
    if sender_balance < tx_cost {
        return Err(JsonRpcError::transaction_rejected("insufficient funds for gas * price + value"));
    }

    let sender_nonce = ctx
        .state_db
        .get_nonce(&sender)
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;

    // Allow nonce to be current or in the future (for queued transactions)
    if signed_tx.nonce() < sender_nonce {
        return Err(JsonRpcError::transaction_rejected(format!(
            "nonce too low: have {}, expected >= {}",
            signed_tx.nonce(),
            sender_nonce
        )));
    }

    // Add to transaction pool
    ctx.txpool
        .add(signed_tx, sender, tx_hash)
        .map_err(|e| JsonRpcError::transaction_rejected(e.to_string()))?;

    Ok(Value::String(tx_hash.to_hex()))
}

/// eth_getBlockByNumber - Get block by number
pub async fn eth_get_block_by_number(
    ctx: Arc<RpcContext>,
    params: Vec<Value>,
) -> Result<Value, JsonRpcError> {
    if params.is_empty() {
        return Err(JsonRpcError::invalid_params("missing block number"));
    }

    let block_id = parse_block_id(&params[0])?;
    let full_txs = params.get(1).and_then(|v| v.as_bool()).unwrap_or(false);

    let block_number = resolve_block_number(&ctx, block_id)?;

    let block_hash = ctx
        .block_db
        .get_hash_by_number(block_number)
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;

    match block_hash {
        Some(hash) => get_block_by_hash_internal(&ctx, hash, block_number, full_txs),
        None => Ok(Value::Null),
    }
}

/// eth_getBlockByHash - Get block by hash
pub async fn eth_get_block_by_hash(
    ctx: Arc<RpcContext>,
    params: Vec<Value>,
) -> Result<Value, JsonRpcError> {
    if params.is_empty() {
        return Err(JsonRpcError::invalid_params("missing block hash"));
    }

    let block_hash = parse_h256(&params[0])?;
    let full_txs = params.get(1).and_then(|v| v.as_bool()).unwrap_or(false);

    // Find block number from hash (we need to scan or have reverse index)
    // For now, scan from latest
    let latest = ctx
        .block_db
        .get_latest_block()
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?
        .unwrap_or(0);

    for number in (0..=latest).rev() {
        if let Some(hash) = ctx
            .block_db
            .get_hash_by_number(number)
            .map_err(|e| JsonRpcError::internal_error(e.to_string()))?
        {
            if hash == block_hash {
                return get_block_by_hash_internal(&ctx, block_hash, number, full_txs);
            }
        }
    }

    Ok(Value::Null)
}

/// eth_getTransactionByHash - Get transaction by hash
pub async fn eth_get_transaction_by_hash(
    ctx: Arc<RpcContext>,
    params: Vec<Value>,
) -> Result<Value, JsonRpcError> {
    if params.is_empty() {
        return Err(JsonRpcError::invalid_params("missing transaction hash"));
    }

    let tx_hash = parse_h256(&params[0])?;

    // Check txpool first for pending transactions
    if let Some(pooled) = ctx.txpool.get_by_hash(&tx_hash) {
        let tx = build_rpc_transaction(&pooled.tx, &pooled.sender, &tx_hash, None, None, None);
        return Ok(serde_json::to_value(tx)
            .map_err(|e| JsonRpcError::internal_error(e.to_string()))?);
    }

    // TODO: Search in block database for confirmed transactions
    // For now, return null if not in txpool
    Ok(Value::Null)
}

/// eth_getTransactionReceipt - Get transaction receipt
pub async fn eth_get_transaction_receipt(
    _ctx: Arc<RpcContext>,
    params: Vec<Value>,
) -> Result<Value, JsonRpcError> {
    if params.is_empty() {
        return Err(JsonRpcError::invalid_params("missing transaction hash"));
    }

    let _tx_hash = parse_h256(&params[0])?;

    // TODO: Search in block database for receipt
    // For now, return null
    Ok(Value::Null)
}

// Helper functions

fn resolve_block_number(ctx: &RpcContext, block_id: BlockId) -> Result<u64, JsonRpcError> {
    match block_id {
        BlockId::Number(n) => Ok(n),
        BlockId::Latest | BlockId::Pending => ctx
            .block_db
            .get_latest_block()
            .map_err(|e| JsonRpcError::internal_error(e.to_string()))?
            .ok_or_else(|| JsonRpcError::resource_not_found("no blocks")),
        BlockId::Earliest => Ok(0),
        BlockId::Safe | BlockId::Finalized => ctx
            .block_db
            .get_finalized_block()
            .map_err(|e| JsonRpcError::internal_error(e.to_string()))?
            .ok_or_else(|| JsonRpcError::resource_not_found("no finalized block")),
        BlockId::Hash(h) => {
            // Find block number by hash
            let latest = ctx
                .block_db
                .get_latest_block()
                .map_err(|e| JsonRpcError::internal_error(e.to_string()))?
                .unwrap_or(0);

            for n in (0..=latest).rev() {
                if let Some(hash) = ctx
                    .block_db
                    .get_hash_by_number(n)
                    .map_err(|e| JsonRpcError::internal_error(e.to_string()))?
                {
                    if hash == h {
                        return Ok(n);
                    }
                }
            }
            Err(JsonRpcError::resource_not_found("block not found"))
        }
    }
}

fn get_block_by_hash_internal(
    ctx: &RpcContext,
    _hash: H256,
    number: u64,
    _full_txs: bool,
) -> Result<Value, JsonRpcError> {
    // Get block hash
    let hash = ctx
        .block_db
        .get_hash_by_number(number)
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?
        .ok_or_else(|| JsonRpcError::resource_not_found("block not found"))?;

    // Get header (raw bytes)
    let _header_bytes = ctx
        .block_db
        .get_header(&hash)
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;

    // For now, return a minimal block structure
    // TODO: Decode header bytes and build full RpcBlock
    let block = RpcBlock {
        hash: hash.to_hex(),
        parent_hash: H256::ZERO.to_hex(),
        sha3_uncles: H256::ZERO.to_hex(),
        miner: Address::ZERO.to_hex(),
        state_root: H256::ZERO.to_hex(),
        transactions_root: H256::ZERO.to_hex(),
        receipts_root: H256::ZERO.to_hex(),
        logs_bloom: format_bytes(&[0u8; 256]),
        difficulty: format_u64(0),
        number: format_u64(number),
        gas_limit: format_u64(30_000_000),
        gas_used: format_u64(0),
        timestamp: format_u64(0),
        extra_data: "0x".to_string(),
        mix_hash: H256::ZERO.to_hex(),
        nonce: format_u64(0),
        base_fee_per_gas: Some(format_u64(1_000_000_000)),
        total_difficulty: format_u64(0),
        size: format_u64(0),
        transactions: vec![],
        uncles: vec![],
    };

    serde_json::to_value(block).map_err(|e| JsonRpcError::internal_error(e.to_string()))
}

fn build_rpc_transaction(
    tx: &bach_types::SignedTransaction,
    sender: &Address,
    hash: &H256,
    block_hash: Option<H256>,
    block_number: Option<u64>,
    tx_index: Option<u64>,
) -> RpcTransaction {
    RpcTransaction {
        hash: hash.to_hex(),
        nonce: format_u64(tx.nonce()),
        block_hash: block_hash.map(|h| h.to_hex()),
        block_number: block_number.map(format_u64),
        transaction_index: tx_index.map(format_u64),
        from: sender.to_hex(),
        to: tx.to().map(|a| a.to_hex()),
        value: format_u128(tx.value()),
        gas: format_u64(tx.gas_limit()),
        gas_price: format_u128(tx.gas_price().unwrap_or(0)),
        input: format_bytes(tx.data()),
        v: format_u64(tx.signature.v),
        r: tx.signature.r.to_hex(),
        s: tx.signature.s.to_hex(),
        tx_type: format_u64(tx.tx_type() as u64),
        chain_id: None, // TODO
        max_fee_per_gas: tx.max_fee_per_gas().map(format_u128),
        max_priority_fee_per_gas: tx.max_priority_fee_per_gas().map(format_u128),
    }
}

fn compute_tx_hash(raw_tx: &[u8]) -> H256 {
    keccak256(raw_tx)
}

fn compute_signing_hash(
    tx: &bach_types::SignedTransaction,
    chain_id: u64,
) -> Result<H256, JsonRpcError> {
    // For legacy transactions, the signing hash is computed from RLP([nonce, gasPrice, gasLimit, to, value, data, chainId, 0, 0])
    // This is EIP-155 replay protection
    let mut stream = RlpStream::new_list(9);
    stream.append(&tx.nonce());
    stream.append(&tx.gas_price().unwrap_or(0));
    stream.append(&tx.gas_limit());

    if let Some(to) = tx.to() {
        stream.append(to);
    } else {
        stream.append_empty_data();
    }

    stream.append(&tx.value());
    stream.append(&tx.data().to_vec());
    stream.append(&chain_id);
    stream.append(&0u8);
    stream.append(&0u8);

    Ok(keccak256(&stream.out()))
}

/// Decode a raw RLP-encoded transaction
fn decode_raw_transaction(raw_tx: &[u8]) -> Result<SignedTransaction, String> {
    if raw_tx.is_empty() {
        return Err("empty transaction data".to_string());
    }

    // Check for typed transaction (EIP-2718)
    // If first byte < 0xc0, it's a typed transaction
    // Otherwise it's a legacy transaction (RLP list)
    if raw_tx[0] >= 0xc0 {
        // Legacy transaction
        decode_legacy_transaction(raw_tx)
    } else {
        // Typed transaction (type prefix + RLP payload)
        // For now, only support legacy
        Err("typed transactions not yet supported".to_string())
    }
}

/// Decode a legacy RLP transaction
fn decode_legacy_transaction(raw_tx: &[u8]) -> Result<SignedTransaction, String> {
    let rlp = Rlp::new(raw_tx);

    if !rlp.is_list() {
        return Err("transaction must be an RLP list".to_string());
    }

    let item_count = rlp.item_count().map_err(|e| e.to_string())?;
    if item_count != 9 {
        return Err(format!(
            "legacy transaction must have 9 items, got {}",
            item_count
        ));
    }

    // Decode fields: nonce, gasPrice, gasLimit, to, value, data, v, r, s
    let nonce: u64 = rlp.val_at(0).map_err(|e| format!("invalid nonce: {}", e))?;
    let gas_price: u128 = decode_u128_from_rlp(&rlp, 1)?;
    let gas_limit: u64 = rlp.val_at(2).map_err(|e| format!("invalid gas_limit: {}", e))?;

    // 'to' can be empty (contract creation) or 20 bytes
    let to_bytes: Vec<u8> = rlp.val_at(3).map_err(|e| format!("invalid to: {}", e))?;
    let to = if to_bytes.is_empty() {
        None
    } else if to_bytes.len() == 20 {
        Some(Address::from_slice(&to_bytes).map_err(|e| format!("invalid to address: {}", e))?)
    } else {
        return Err(format!("invalid to address length: {}", to_bytes.len()));
    };

    let value: u128 = decode_u128_from_rlp(&rlp, 4)?;
    let data: Vec<u8> = rlp.val_at(5).map_err(|e| format!("invalid data: {}", e))?;

    let v: u64 = rlp.val_at(6).map_err(|e| format!("invalid v: {}", e))?;
    let r: H256 = rlp.val_at(7).map_err(|e| format!("invalid r: {}", e))?;
    let s: H256 = rlp.val_at(8).map_err(|e| format!("invalid s: {}", e))?;

    let tx = LegacyTx {
        nonce,
        gas_price,
        gas_limit,
        to,
        value,
        data: Bytes::from(data),
    };

    let signature = TxSignature::new(v, r, s);

    Ok(SignedTransaction::new_legacy(tx, signature))
}

/// Helper to decode u128 from RLP (handles variable-length encoding)
fn decode_u128_from_rlp(rlp: &Rlp, index: usize) -> Result<u128, String> {
    let bytes: Vec<u8> = rlp.val_at(index).map_err(|e| format!("invalid u128 at {}: {}", index, e))?;
    if bytes.is_empty() {
        return Ok(0);
    }
    if bytes.len() > 16 {
        return Err(format!("u128 too large at index {}", index));
    }
    let mut arr = [0u8; 16];
    arr[16 - bytes.len()..].copy_from_slice(&bytes);
    Ok(u128::from_be_bytes(arr))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bach_types::TxSignature;

    // ===== Format Function Tests =====

    #[test]
    fn test_format_functions() {
        assert_eq!(format_u64(0), "0x0");
        assert_eq!(format_u64(255), "0xff");
        assert_eq!(format_u128(1_000_000_000), "0x3b9aca00");
        assert_eq!(format_bytes(&[0xab, 0xcd]), "0xabcd");
    }

    #[test]
    fn test_format_u64_edge_cases() {
        assert_eq!(format_u64(u64::MAX), "0xffffffffffffffff");
        assert_eq!(format_u64(1), "0x1");
        assert_eq!(format_u64(16), "0x10");
    }

    #[test]
    fn test_format_u128_edge_cases() {
        assert_eq!(format_u128(0), "0x0");
        assert_eq!(format_u128(u128::MAX), "0xffffffffffffffffffffffffffffffff");
    }

    #[test]
    fn test_format_bytes_edge_cases() {
        assert_eq!(format_bytes(&[]), "0x");
        assert_eq!(format_bytes(&[0x00]), "0x00");
        assert_eq!(format_bytes(&[0xff, 0xff]), "0xffff");
    }

    // ===== Transaction Hash Tests =====

    #[test]
    fn test_compute_tx_hash() {
        let raw_tx = hex::decode("f86c098504a817c800825208943535353535353535353535353535353535353535880de0b6b3a76400008025a028ef61340bd939bc2195fe537567866003e1a15d3c71ff63e1590620aa636276a067cbe9d8997f761aecb703304b3800ccf555c9f3dc64214b297fb1966a3b6d83").unwrap();
        let hash = compute_tx_hash(&raw_tx);
        // Hash should be non-zero
        assert_ne!(hash, H256::ZERO);
    }

    // ===== decode_raw_transaction Tests =====

    #[test]
    fn test_decode_raw_transaction_empty() {
        let result = decode_raw_transaction(&[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));
    }

    #[test]
    fn test_decode_raw_transaction_typed_not_supported() {
        // Typed transaction (starts with 0x01, 0x02, etc.)
        let typed_tx = vec![0x02, 0x01, 0x02, 0x03];
        let result = decode_raw_transaction(&typed_tx);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("typed transactions not yet supported"));
    }

    #[test]
    fn test_decode_legacy_transaction_invalid_rlp() {
        // Invalid RLP that's not a list
        let invalid = vec![0x80]; // RLP empty string
        let result = decode_raw_transaction(&invalid);
        assert!(result.is_err());
    }

    // ===== BlockId Resolution Tests =====

    #[test]
    fn test_block_id_variants() {
        // Test that all BlockId variants exist
        let _ = BlockId::Number(100);
        let _ = BlockId::Hash(H256::ZERO);
        let _ = BlockId::Latest;
        let _ = BlockId::Earliest;
        let _ = BlockId::Pending;
        let _ = BlockId::Safe;
        let _ = BlockId::Finalized;
    }

    // ===== CallRequest Parsing Tests =====

    #[test]
    fn test_call_request_defaults() {
        let req = CallRequest::default();
        assert!(req.from.is_none());
        assert!(req.to.is_none());
        assert!(req.gas.is_none());
        assert!(req.gas_price.is_none());
        assert!(req.value.is_none());
        assert!(req.data.is_none());
        assert!(req.nonce.is_none());
    }

    // ===== RpcTransaction Builder Tests =====

    #[test]
    fn test_build_rpc_transaction_pending() {
        let tx = SignedTransaction::new_legacy(
            LegacyTx {
                nonce: 0,
                gas_price: 1_000_000_000,
                gas_limit: 21000,
                to: Some(Address::ZERO),
                value: 1_000_000_000_000_000_000, // 1 ETH
                data: Bytes::new(),
            },
            TxSignature::new(27, H256::ZERO, H256::ZERO),
        );

        let sender = Address::ZERO;
        let hash = H256::ZERO;

        let rpc_tx = build_rpc_transaction(&tx, &sender, &hash, None, None, None);

        // Pending transaction should have None for block info
        assert!(rpc_tx.block_hash.is_none());
        assert!(rpc_tx.block_number.is_none());
        assert!(rpc_tx.transaction_index.is_none());
        assert_eq!(rpc_tx.nonce, "0x0");
        assert_eq!(rpc_tx.gas, "0x5208"); // 21000 in hex
    }

    #[test]
    fn test_build_rpc_transaction_confirmed() {
        let tx = SignedTransaction::new_legacy(
            LegacyTx {
                nonce: 5,
                gas_price: 2_000_000_000,
                gas_limit: 21000,
                to: Some(Address::ZERO),
                value: 0,
                data: Bytes::from(vec![0xab, 0xcd]),
            },
            TxSignature::new(28, H256::ZERO, H256::ZERO),
        );

        let sender = Address::ZERO;
        let hash = H256::ZERO;
        let block_hash = H256::from_bytes([0x01; 32]);
        let block_number = 100u64;
        let tx_index = 5u64;

        let rpc_tx = build_rpc_transaction(
            &tx,
            &sender,
            &hash,
            Some(block_hash),
            Some(block_number),
            Some(tx_index),
        );

        // Confirmed transaction should have block info
        assert!(rpc_tx.block_hash.is_some());
        assert_eq!(rpc_tx.block_number, Some("0x64".to_string())); // 100 in hex
        assert_eq!(rpc_tx.transaction_index, Some("0x5".to_string()));
        assert_eq!(rpc_tx.nonce, "0x5");
        assert_eq!(rpc_tx.input, "0xabcd");
    }

    // ===== Address Parsing in eth methods =====

    #[test]
    fn test_parse_address_valid() {
        let addr_value = Value::String("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266".to_string());
        let result = parse_address(&addr_value);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_address_invalid_type() {
        let result = parse_address(&Value::Number(123.into()));
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_address_invalid_hex() {
        let result = parse_address(&Value::String("0xGGGG".to_string()));
        assert!(result.is_err());
    }

    // ===== H256 Parsing Tests =====

    #[test]
    fn test_parse_h256_valid() {
        let hash_str = "0x0000000000000000000000000000000000000000000000000000000000000001";
        let result = parse_h256(&Value::String(hash_str.to_string()));
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_h256_short() {
        let result = parse_h256(&Value::String("0x1234".to_string()));
        assert!(result.is_err());
    }

    // ===== Hex Bytes Parsing Tests =====

    #[test]
    fn test_parse_hex_bytes_valid() {
        let result = parse_hex_bytes(&Value::String("0xabcd1234".to_string()));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![0xab, 0xcd, 0x12, 0x34]);
    }

    #[test]
    fn test_parse_hex_bytes_empty() {
        let result = parse_hex_bytes(&Value::String("0x".to_string()));
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_parse_hex_bytes_no_prefix() {
        let result = parse_hex_bytes(&Value::String("abcd".to_string()));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![0xab, 0xcd]);
    }
}
