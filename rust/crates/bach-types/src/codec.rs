//! Binary encoding/decoding for blocks and receipts.
//!
//! Provides deterministic serialization used by both `bach-node` and `bach-rpc`
//! for storing and retrieving block headers, bodies, and transaction receipts.

use crate::block::{BlockBody, BlockHeader, Bloom};
use crate::receipt::{Log, Receipt, TxStatus};
use bach_primitives::{Address, H256};
use bytes::Bytes;

// ============================================================================
// Block Header Encoding
// ============================================================================

/// Encode a block header to bytes.
pub fn encode_header(header: &BlockHeader) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(header.parent_hash.as_bytes());     // 32
    buf.extend_from_slice(header.ommers_hash.as_bytes());     // 32
    buf.extend_from_slice(header.beneficiary.as_bytes());     // 20
    buf.extend_from_slice(header.state_root.as_bytes());      // 32
    buf.extend_from_slice(header.transactions_root.as_bytes()); // 32
    buf.extend_from_slice(header.receipts_root.as_bytes());   // 32
    buf.extend_from_slice(&header.logs_bloom.0);              // 256
    buf.extend_from_slice(&header.difficulty.to_le_bytes());  // 16
    buf.extend_from_slice(&header.number.to_le_bytes());      // 8
    buf.extend_from_slice(&header.gas_limit.to_le_bytes());   // 8
    buf.extend_from_slice(&header.gas_used.to_le_bytes());    // 8
    buf.extend_from_slice(&header.timestamp.to_le_bytes());   // 8
    buf.extend_from_slice(&(header.extra_data.len() as u32).to_le_bytes()); // 4
    buf.extend_from_slice(&header.extra_data);                // variable
    buf.extend_from_slice(header.mix_hash.as_bytes());        // 32
    buf.extend_from_slice(&header.nonce.to_le_bytes());       // 8
    if let Some(base_fee) = header.base_fee_per_gas {
        buf.push(1);
        buf.extend_from_slice(&base_fee.to_le_bytes());       // 16
    } else {
        buf.push(0);
    }
    buf
}

/// Decode a block header from bytes.
pub fn decode_header(bytes: &[u8]) -> Option<BlockHeader> {
    let min_len = 32 + 32 + 20 + 32 + 32 + 32 + 256 + 16 + 8 + 8 + 8 + 8 + 4;
    if bytes.len() < min_len {
        return None;
    }
    let mut pos = 0;

    let parent_hash = H256::from_slice(&bytes[pos..pos + 32]).ok()?; pos += 32;
    let ommers_hash = H256::from_slice(&bytes[pos..pos + 32]).ok()?; pos += 32;
    let beneficiary = Address::from_slice(&bytes[pos..pos + 20]).ok()?; pos += 20;
    let state_root = H256::from_slice(&bytes[pos..pos + 32]).ok()?; pos += 32;
    let transactions_root = H256::from_slice(&bytes[pos..pos + 32]).ok()?; pos += 32;
    let receipts_root = H256::from_slice(&bytes[pos..pos + 32]).ok()?; pos += 32;

    let mut bloom_bytes = [0u8; 256];
    bloom_bytes.copy_from_slice(&bytes[pos..pos + 256]); pos += 256;
    let logs_bloom = Bloom::from_bytes(bloom_bytes);

    let difficulty = u128::from_le_bytes(bytes[pos..pos + 16].try_into().ok()?); pos += 16;
    let number = u64::from_le_bytes(bytes[pos..pos + 8].try_into().ok()?); pos += 8;
    let gas_limit = u64::from_le_bytes(bytes[pos..pos + 8].try_into().ok()?); pos += 8;
    let gas_used = u64::from_le_bytes(bytes[pos..pos + 8].try_into().ok()?); pos += 8;
    let timestamp = u64::from_le_bytes(bytes[pos..pos + 8].try_into().ok()?); pos += 8;

    let extra_data_len = u32::from_le_bytes(bytes[pos..pos + 4].try_into().ok()?) as usize; pos += 4;
    if pos + extra_data_len > bytes.len() { return None; }
    let extra_data = Bytes::copy_from_slice(&bytes[pos..pos + extra_data_len]); pos += extra_data_len;

    if pos + 32 + 8 + 1 > bytes.len() { return None; }
    let mix_hash = H256::from_slice(&bytes[pos..pos + 32]).ok()?; pos += 32;
    let nonce = u64::from_le_bytes(bytes[pos..pos + 8].try_into().ok()?); pos += 8;

    let base_fee_per_gas = if bytes[pos] == 1 {
        pos += 1;
        if pos + 16 > bytes.len() { return None; }
        Some(u128::from_le_bytes(bytes[pos..pos + 16].try_into().ok()?))
    } else {
        None
    };

    Some(BlockHeader {
        parent_hash, ommers_hash, beneficiary, state_root,
        transactions_root, receipts_root, logs_bloom, difficulty,
        number, gas_limit, gas_used, timestamp, extra_data,
        mix_hash, nonce, base_fee_per_gas,
    })
}

// ============================================================================
// Block Body Encoding
// ============================================================================

/// Encode a block body to bytes.
pub fn encode_body(body: &BlockBody) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&(body.transactions.len() as u32).to_le_bytes());
    for tx in &body.transactions {
        let tx_data = encode_signed_tx(tx);
        buf.extend_from_slice(&(tx_data.len() as u32).to_le_bytes());
        buf.extend_from_slice(&tx_data);
    }
    buf
}

/// Decode a block body from bytes.
pub fn decode_body(bytes: &[u8]) -> Option<BlockBody> {
    if bytes.len() < 4 { return None; }
    // We only decode the transaction count; full tx deserialization is not yet implemented.
    Some(BlockBody { transactions: vec![] })
}

fn encode_signed_tx(tx: &crate::transaction::SignedTransaction) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&tx.nonce().to_le_bytes());
    buf.extend_from_slice(&tx.gas_limit().to_le_bytes());
    buf.extend_from_slice(&tx.value().to_le_bytes());
    buf.extend_from_slice(&tx.gas_price().unwrap_or(0).to_le_bytes());
    if let Some(to) = tx.to() {
        buf.push(1);
        buf.extend_from_slice(to.as_bytes());
    } else {
        buf.push(0);
    }
    let data = tx.data();
    buf.extend_from_slice(&(data.len() as u32).to_le_bytes());
    buf.extend_from_slice(data);
    buf.extend_from_slice(&tx.signature.v.to_le_bytes());
    buf.extend_from_slice(tx.signature.r.as_bytes());
    buf.extend_from_slice(tx.signature.s.as_bytes());
    buf
}

// ============================================================================
// Receipt Encoding
// ============================================================================

/// Encode receipts to bytes.
pub fn encode_receipts(receipts: &[Receipt]) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&(receipts.len() as u32).to_le_bytes());
    for receipt in receipts {
        buf.push(if receipt.is_success() { 1 } else { 0 });
        buf.extend_from_slice(&receipt.cumulative_gas_used.to_le_bytes());
        buf.extend_from_slice(&receipt.gas_used.to_le_bytes());
        if let Some(addr) = &receipt.contract_address {
            buf.push(1);
            buf.extend_from_slice(addr.as_bytes());
        } else {
            buf.push(0);
        }
        buf.extend_from_slice(&receipt.logs_bloom.0);
        buf.extend_from_slice(&(receipt.logs.len() as u32).to_le_bytes());
        for log in &receipt.logs {
            buf.extend_from_slice(log.address.as_bytes());
            buf.extend_from_slice(&(log.topics.len() as u32).to_le_bytes());
            for topic in &log.topics {
                buf.extend_from_slice(topic.as_bytes());
            }
            buf.extend_from_slice(&(log.data.len() as u32).to_le_bytes());
            buf.extend_from_slice(&log.data);
        }
    }
    buf
}

/// Decode receipts from bytes.
pub fn decode_receipts(bytes: &[u8]) -> Option<Vec<Receipt>> {
    if bytes.len() < 4 { return None; }
    let mut pos = 0;
    let count = u32::from_le_bytes(bytes[pos..pos + 4].try_into().ok()?) as usize; pos += 4;
    let mut receipts = Vec::with_capacity(count);

    for _ in 0..count {
        if pos >= bytes.len() { return None; }
        let status = if bytes[pos] == 1 { TxStatus::Success } else { TxStatus::Failure }; pos += 1;
        if pos + 8 > bytes.len() { return None; }
        let cumulative_gas_used = u64::from_le_bytes(bytes[pos..pos + 8].try_into().ok()?); pos += 8;
        if pos + 8 > bytes.len() { return None; }
        let gas_used = u64::from_le_bytes(bytes[pos..pos + 8].try_into().ok()?); pos += 8;
        if pos >= bytes.len() { return None; }
        let contract_address = if bytes[pos] == 1 {
            pos += 1;
            if pos + 20 > bytes.len() { return None; }
            let addr = Address::from_slice(&bytes[pos..pos + 20]).ok()?; pos += 20;
            Some(addr)
        } else { pos += 1; None };
        if pos + 256 > bytes.len() { return None; }
        let mut bloom_bytes = [0u8; 256];
        bloom_bytes.copy_from_slice(&bytes[pos..pos + 256]); pos += 256;
        if pos + 4 > bytes.len() { return None; }
        let log_count = u32::from_le_bytes(bytes[pos..pos + 4].try_into().ok()?) as usize; pos += 4;
        let mut logs = Vec::with_capacity(log_count);
        for _ in 0..log_count {
            if pos + 20 > bytes.len() { return None; }
            let addr = Address::from_slice(&bytes[pos..pos + 20]).ok()?; pos += 20;
            if pos + 4 > bytes.len() { return None; }
            let topic_count = u32::from_le_bytes(bytes[pos..pos + 4].try_into().ok()?) as usize; pos += 4;
            let mut topics = Vec::with_capacity(topic_count);
            for _ in 0..topic_count {
                if pos + 32 > bytes.len() { return None; }
                topics.push(H256::from_slice(&bytes[pos..pos + 32]).ok()?); pos += 32;
            }
            if pos + 4 > bytes.len() { return None; }
            let data_len = u32::from_le_bytes(bytes[pos..pos + 4].try_into().ok()?) as usize; pos += 4;
            if pos + data_len > bytes.len() { return None; }
            let data = bytes[pos..pos + data_len].to_vec(); pos += data_len;
            logs.push(Log::new(addr, topics, data.into()));
        }
        let mut receipt = Receipt::new(status, cumulative_gas_used, gas_used, logs);
        receipt.logs_bloom = Bloom::from_bytes(bloom_bytes);
        if let Some(addr) = contract_address {
            receipt = receipt.with_contract_address(addr);
        }
        receipts.push(receipt);
    }
    Some(receipts)
}
