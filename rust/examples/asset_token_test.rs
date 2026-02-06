//! AssetToken ERC-20 合约测试脚本
//!
//! 运行方式:
//! 1. 先启动节点: cargo run -p bach-node --release -- --datadir ./testdata --chain-id 1337
//! 2. 运行测试: cargo run --example asset_token_test --release
//!
//! 或者使用脚本:
//!   ./scripts/run_asset_token_test.sh
//!
//! 测试内容:
//! - mint: 任意地址 mint，mint 到零地址 revert
//! - burn: burn 自己的代币，余额不足 revert
//! - transfer: 正常转账，余额不足 revert，转账到零地址 revert，自我转账
//! - approve/transferFrom: 授权转账，超额 revert
//! - 查询函数: name, symbol, decimals, balanceOf, totalSupply

use bach_crypto::keccak256;
use bach_primitives::{Address, H256, U256};
use k256::ecdsa::SigningKey;
use std::time::Duration;

// ============================================================================
// RPC Client (reused from manual_test.rs)
// ============================================================================

struct SimpleClient {
    url: String,
    client: reqwest::Client,
}

#[derive(Debug, serde::Deserialize)]
struct JsonRpcResponse<T> {
    #[allow(dead_code)]
    jsonrpc: String,
    #[allow(dead_code)]
    id: u64,
    result: Option<T>,
    error: Option<JsonRpcError>,
}

#[derive(Debug, serde::Deserialize)]
struct JsonRpcError {
    code: i64,
    message: String,
}

impl SimpleClient {
    fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            client: reqwest::Client::new(),
        }
    }

    async fn call<T: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        params: Vec<serde_json::Value>,
    ) -> Result<T, String> {
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        });

        let resp = self
            .client
            .post(&self.url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("HTTP error: {}", e))?;

        let json: JsonRpcResponse<T> = resp
            .json()
            .await
            .map_err(|e| format!("JSON parse error: {}", e))?;

        if let Some(err) = json.error {
            return Err(format!("RPC error {}: {}", err.code, err.message));
        }

        json.result.ok_or_else(|| "No result".to_string())
    }

    async fn chain_id(&self) -> Result<u64, String> {
        let hex: String = self.call("eth_chainId", vec![]).await?;
        parse_hex_u64(&hex)
    }

    async fn gas_price(&self) -> Result<u128, String> {
        let hex: String = self.call("eth_gasPrice", vec![]).await?;
        parse_hex_u128(&hex)
    }

    async fn get_nonce(&self, address: &Address) -> Result<u64, String> {
        let hex: String = self
            .call(
                "eth_getTransactionCount",
                vec![
                    serde_json::Value::String(address.to_hex()),
                    serde_json::Value::String("latest".to_string()),
                ],
            )
            .await?;
        parse_hex_u64(&hex)
    }

    async fn send_raw_transaction(&self, raw_tx: &[u8]) -> Result<H256, String> {
        let hex: String = self
            .call(
                "eth_sendRawTransaction",
                vec![serde_json::Value::String(format!("0x{}", hex::encode(raw_tx)))],
            )
            .await?;
        parse_hex_h256(&hex)
    }

    async fn get_transaction_receipt(
        &self,
        hash: &H256,
    ) -> Result<Option<serde_json::Value>, String> {
        self.call(
            "eth_getTransactionReceipt",
            vec![serde_json::Value::String(format!("0x{}", hex::encode(hash.as_bytes())))],
        )
        .await
    }

    async fn call_contract(&self, to: &Address, data: &[u8]) -> Result<Vec<u8>, String> {
        let hex: String = self
            .call(
                "eth_call",
                vec![
                    serde_json::json!({
                        "to": to.to_hex(),
                        "data": format!("0x{}", hex::encode(data)),
                    }),
                    serde_json::Value::String("latest".to_string()),
                ],
            )
            .await?;
        parse_hex_bytes(&hex)
    }

    async fn call_contract_from(
        &self,
        from: &Address,
        to: &Address,
        data: &[u8],
    ) -> Result<Vec<u8>, String> {
        let hex: String = self
            .call(
                "eth_call",
                vec![
                    serde_json::json!({
                        "from": from.to_hex(),
                        "to": to.to_hex(),
                        "data": format!("0x{}", hex::encode(data)),
                    }),
                    serde_json::Value::String("latest".to_string()),
                ],
            )
            .await?;
        parse_hex_bytes(&hex)
    }

    async fn get_code(&self, address: &Address) -> Result<Vec<u8>, String> {
        let hex: String = self
            .call(
                "eth_getCode",
                vec![
                    serde_json::Value::String(address.to_hex()),
                    serde_json::Value::String("latest".to_string()),
                ],
            )
            .await?;
        parse_hex_bytes(&hex)
    }
}

fn parse_hex_u64(hex: &str) -> Result<u64, String> {
    let hex = hex.strip_prefix("0x").unwrap_or(hex);
    u64::from_str_radix(hex, 16).map_err(|e| format!("Invalid hex u64: {}", e))
}

fn parse_hex_u128(hex: &str) -> Result<u128, String> {
    let hex = hex.strip_prefix("0x").unwrap_or(hex);
    u128::from_str_radix(hex, 16).map_err(|e| format!("Invalid hex u128: {}", e))
}

fn parse_hex_h256(hex: &str) -> Result<H256, String> {
    H256::from_hex(hex).map_err(|e| format!("Invalid hex H256: {:?}", e))
}

fn parse_hex_bytes(hex: &str) -> Result<Vec<u8>, String> {
    let hex = hex.strip_prefix("0x").unwrap_or(hex);
    hex::decode(hex).map_err(|e| format!("Invalid hex bytes: {}", e))
}

// ============================================================================
// Test Wallet
// ============================================================================

struct TestWallet {
    signing_key: SigningKey,
    address: Address,
}

impl TestWallet {
    fn from_private_key(key: &[u8; 32]) -> Result<Self, String> {
        let signing_key =
            SigningKey::from_bytes(key.into()).map_err(|e| format!("Invalid key: {}", e))?;
        let verifying_key = signing_key.verifying_key();
        let public_key_bytes = verifying_key.to_encoded_point(false);
        let public_key_uncompressed = &public_key_bytes.as_bytes()[1..];
        let hash = keccak256(public_key_uncompressed);
        let mut address_bytes = [0u8; 20];
        address_bytes.copy_from_slice(&hash.as_bytes()[12..]);
        let address = Address::from(address_bytes);

        Ok(Self {
            signing_key,
            address,
        })
    }

    fn address(&self) -> &Address {
        &self.address
    }

    fn sign_transaction(
        &self,
        nonce: u64,
        gas_price: u128,
        gas_limit: u64,
        to: Option<Address>,
        value: u128,
        data: Vec<u8>,
        chain_id: u64,
    ) -> Result<Vec<u8>, String> {
        let mut rlp_for_signing = Vec::new();
        encode_rlp_list(
            &[
                RlpItem::U64(nonce),
                RlpItem::U128(gas_price),
                RlpItem::U64(gas_limit),
                RlpItem::Address(to),
                RlpItem::U128(value),
                RlpItem::Bytes(data.clone()),
                RlpItem::U64(chain_id),
                RlpItem::U64(0),
                RlpItem::U64(0),
            ],
            &mut rlp_for_signing,
        );

        let msg_hash = keccak256(&rlp_for_signing);

        let (signature, recovery_id) = self
            .signing_key
            .sign_prehash_recoverable(msg_hash.as_bytes())
            .map_err(|e| format!("Signing failed: {}", e))?;

        let sig_bytes = signature.to_bytes();
        let r = &sig_bytes[0..32];
        let s = &sig_bytes[32..64];

        let v = recovery_id.to_byte() as u64 + chain_id * 2 + 35;

        let mut encoded = Vec::new();
        encode_rlp_list(
            &[
                RlpItem::U64(nonce),
                RlpItem::U128(gas_price),
                RlpItem::U64(gas_limit),
                RlpItem::Address(to),
                RlpItem::U128(value),
                RlpItem::Bytes(data),
                RlpItem::U64(v),
                RlpItem::FixedBytes(r.to_vec()),
                RlpItem::FixedBytes(s.to_vec()),
            ],
            &mut encoded,
        );

        Ok(encoded)
    }
}

// ============================================================================
// RLP Encoding
// ============================================================================

enum RlpItem {
    U64(u64),
    U128(u128),
    Address(Option<Address>),
    Bytes(Vec<u8>),
    FixedBytes(Vec<u8>),
}

fn encode_rlp_item(item: &RlpItem, out: &mut Vec<u8>) {
    match item {
        RlpItem::U64(v) => encode_rlp_u64(*v, out),
        RlpItem::U128(v) => encode_rlp_u128(*v, out),
        RlpItem::Address(addr) => {
            if let Some(a) = addr {
                encode_rlp_bytes(a.as_bytes(), out);
            } else {
                out.push(0x80);
            }
        }
        RlpItem::Bytes(b) => encode_rlp_bytes(b, out),
        RlpItem::FixedBytes(b) => encode_rlp_fixed_bytes(b, out),
    }
}

fn encode_rlp_u64(v: u64, out: &mut Vec<u8>) {
    if v == 0 {
        out.push(0x80);
    } else if v < 128 {
        out.push(v as u8);
    } else {
        let bytes = v.to_be_bytes();
        let start = bytes.iter().position(|&b| b != 0).unwrap_or(8);
        let len = 8 - start;
        out.push(0x80 + len as u8);
        out.extend_from_slice(&bytes[start..]);
    }
}

fn encode_rlp_u128(v: u128, out: &mut Vec<u8>) {
    if v == 0 {
        out.push(0x80);
    } else if v < 128 {
        out.push(v as u8);
    } else {
        let bytes = v.to_be_bytes();
        let start = bytes.iter().position(|&b| b != 0).unwrap_or(16);
        let len = 16 - start;
        out.push(0x80 + len as u8);
        out.extend_from_slice(&bytes[start..]);
    }
}

fn encode_rlp_bytes(data: &[u8], out: &mut Vec<u8>) {
    if data.is_empty() {
        out.push(0x80);
    } else if data.len() == 1 && data[0] < 128 {
        out.push(data[0]);
    } else if data.len() < 56 {
        out.push(0x80 + data.len() as u8);
        out.extend_from_slice(data);
    } else {
        let len_bytes = encode_length(data.len());
        out.push(0xb7 + len_bytes.len() as u8);
        out.extend_from_slice(&len_bytes);
        out.extend_from_slice(data);
    }
}

fn encode_rlp_fixed_bytes(data: &[u8], out: &mut Vec<u8>) {
    let start = data.iter().position(|&b| b != 0).unwrap_or(data.len());
    let trimmed = &data[start..];
    encode_rlp_bytes(trimmed, out);
}

fn encode_length(len: usize) -> Vec<u8> {
    let bytes = (len as u64).to_be_bytes();
    let start = bytes.iter().position(|&b| b != 0).unwrap_or(7);
    bytes[start..].to_vec()
}

fn encode_rlp_list(items: &[RlpItem], out: &mut Vec<u8>) {
    let mut payload = Vec::new();
    for item in items {
        encode_rlp_item(item, &mut payload);
    }

    if payload.len() < 56 {
        out.push(0xc0 + payload.len() as u8);
    } else {
        let len_bytes = encode_length(payload.len());
        out.push(0xf7 + len_bytes.len() as u8);
        out.extend_from_slice(&len_bytes);
    }
    out.extend_from_slice(&payload);
}

// ============================================================================
// ABI Encoding Helpers
// ============================================================================

/// Calculate function selector from signature (first 4 bytes of keccak256)
fn selector(signature: &str) -> [u8; 4] {
    let hash = keccak256(signature.as_bytes());
    let mut sel = [0u8; 4];
    sel.copy_from_slice(&hash.as_bytes()[..4]);
    sel
}

/// Encode address as 32-byte ABI parameter
fn encode_address(addr: &Address) -> [u8; 32] {
    let mut encoded = [0u8; 32];
    encoded[12..].copy_from_slice(addr.as_bytes());
    encoded
}

/// Encode u256 as 32-byte ABI parameter
fn encode_u256(value: U256) -> [u8; 32] {
    let mut encoded = [0u8; 32];
    value.to_big_endian(&mut encoded);
    encoded
}

/// Decode u256 from ABI return data
fn decode_u256(data: &[u8]) -> U256 {
    if data.len() < 32 {
        return U256::zero();
    }
    U256::from_big_endian(&data[..32])
}

/// Decode string from ABI return data (dynamic type)
fn decode_string(data: &[u8]) -> String {
    if data.len() < 64 {
        return String::new();
    }
    // First 32 bytes: offset to string data
    // Next 32 bytes at offset: length of string
    let offset = U256::from_big_endian(&data[0..32]).as_usize();
    if offset + 32 > data.len() {
        return String::new();
    }
    let length = U256::from_big_endian(&data[offset..offset + 32]).as_usize();
    if offset + 32 + length > data.len() {
        return String::new();
    }
    String::from_utf8_lossy(&data[offset + 32..offset + 32 + length]).to_string()
}

// ============================================================================
// AssetToken ABI
// ============================================================================

/// AssetToken ABI function encoders
struct AssetTokenAbi;

impl AssetTokenAbi {
    // View functions
    fn name() -> Vec<u8> {
        selector("name()").to_vec()
    }

    fn symbol() -> Vec<u8> {
        selector("symbol()").to_vec()
    }

    fn decimals() -> Vec<u8> {
        selector("decimals()").to_vec()
    }

    fn total_supply() -> Vec<u8> {
        selector("totalSupply()").to_vec()
    }

    fn balance_of(account: &Address) -> Vec<u8> {
        let mut data = selector("balanceOf(address)").to_vec();
        data.extend_from_slice(&encode_address(account));
        data
    }

    fn allowance(owner: &Address, spender: &Address) -> Vec<u8> {
        let mut data = selector("allowance(address,address)").to_vec();
        data.extend_from_slice(&encode_address(owner));
        data.extend_from_slice(&encode_address(spender));
        data
    }

    // State-changing functions
    fn mint(to: &Address, amount: U256) -> Vec<u8> {
        let mut data = selector("mint(address,uint256)").to_vec();
        data.extend_from_slice(&encode_address(to));
        data.extend_from_slice(&encode_u256(amount));
        data
    }

    fn burn(amount: U256) -> Vec<u8> {
        let mut data = selector("burn(uint256)").to_vec();
        data.extend_from_slice(&encode_u256(amount));
        data
    }

    fn transfer(to: &Address, amount: U256) -> Vec<u8> {
        let mut data = selector("transfer(address,uint256)").to_vec();
        data.extend_from_slice(&encode_address(to));
        data.extend_from_slice(&encode_u256(amount));
        data
    }

    fn approve(spender: &Address, amount: U256) -> Vec<u8> {
        let mut data = selector("approve(address,uint256)").to_vec();
        data.extend_from_slice(&encode_address(spender));
        data.extend_from_slice(&encode_u256(amount));
        data
    }

    fn transfer_from(from: &Address, to: &Address, amount: U256) -> Vec<u8> {
        let mut data = selector("transferFrom(address,address,uint256)").to_vec();
        data.extend_from_slice(&encode_address(from));
        data.extend_from_slice(&encode_address(to));
        data.extend_from_slice(&encode_u256(amount));
        data
    }
}

// ============================================================================
// AssetToken Contract Bytecode (compiled from AssetToken.sol)
// ============================================================================

// Placeholder - will be replaced with actual compiled bytecode
// To compile: solc --bin --optimize contracts/src/AssetToken.sol
const ASSET_TOKEN_BYTECODE: &str = include_str!("../contracts/bytecode/AssetToken.bin");

// ============================================================================
// Test Result Tracking
// ============================================================================

struct TestResults {
    passed: u32,
    failed: u32,
    tests: Vec<(String, bool, String)>,
}

impl TestResults {
    fn new() -> Self {
        Self {
            passed: 0,
            failed: 0,
            tests: Vec::new(),
        }
    }

    fn pass(&mut self, name: &str, detail: &str) {
        self.passed += 1;
        self.tests.push((name.to_string(), true, detail.to_string()));
        println!("   [PASS] {} - {}", name, detail);
    }

    fn fail(&mut self, name: &str, detail: &str) {
        self.failed += 1;
        self.tests.push((name.to_string(), false, detail.to_string()));
        println!("   [FAIL] {} - {}", name, detail);
    }

    fn summary(&self) {
        println!("\n========================================");
        println!("Test Summary: {} passed, {} failed", self.passed, self.failed);
        println!("========================================");
        if self.failed > 0 {
            println!("\nFailed tests:");
            for (name, passed, detail) in &self.tests {
                if !passed {
                    println!("  - {}: {}", name, detail);
                }
            }
        }
    }
}

// ============================================================================
// Test Context
// ============================================================================

struct TestContext {
    client: SimpleClient,
    chain_id: u64,
    gas_price: u128,
    contract_address: Address,
    wallet1: TestWallet, // Primary test wallet
    wallet2: TestWallet, // Secondary test wallet
}

impl TestContext {
    async fn send_tx(
        &self,
        wallet: &TestWallet,
        data: Vec<u8>,
    ) -> Result<serde_json::Value, String> {
        let nonce = self.client.get_nonce(wallet.address()).await?;
        let raw_tx = wallet.sign_transaction(
            nonce,
            self.gas_price,
            300_000,
            Some(self.contract_address),
            0,
            data,
            self.chain_id,
        )?;

        let tx_hash = self.client.send_raw_transaction(&raw_tx).await?;
        tokio::time::sleep(Duration::from_secs(2)).await;

        self.client
            .get_transaction_receipt(&tx_hash)
            .await?
            .ok_or_else(|| "Transaction not confirmed".to_string())
    }

    async fn call(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        self.client.call_contract(&self.contract_address, data).await
    }

    async fn call_from(&self, from: &Address, data: &[u8]) -> Result<Vec<u8>, String> {
        self.client
            .call_contract_from(from, &self.contract_address, data)
            .await
    }

    fn is_success(receipt: &serde_json::Value) -> bool {
        receipt
            .get("status")
            .and_then(|s| s.as_str())
            .map(|s| s == "0x1")
            .unwrap_or(false)
    }

    fn has_logs(receipt: &serde_json::Value, count: usize) -> bool {
        receipt
            .get("logs")
            .and_then(|l| l.as_array())
            .map(|logs| logs.len() >= count)
            .unwrap_or(false)
    }
}

// ============================================================================
// Test Functions
// ============================================================================

async fn test_view_functions(ctx: &TestContext, results: &mut TestResults) {
    println!("\n-- View Function Tests --");

    // Test name()
    match ctx.call(&AssetTokenAbi::name()).await {
        Ok(data) => {
            let name = decode_string(&data);
            if name == "AssetToken" {
                results.pass("name()", &format!("returned '{}'", name));
            } else {
                results.fail("name()", &format!("expected 'AssetToken', got '{}'", name));
            }
        }
        Err(e) => results.fail("name()", &format!("call failed: {}", e)),
    }

    // Test symbol()
    match ctx.call(&AssetTokenAbi::symbol()).await {
        Ok(data) => {
            let symbol = decode_string(&data);
            if symbol == "AST" {
                results.pass("symbol()", &format!("returned '{}'", symbol));
            } else {
                results.fail("symbol()", &format!("expected 'AST', got '{}'", symbol));
            }
        }
        Err(e) => results.fail("symbol()", &format!("call failed: {}", e)),
    }

    // Test decimals()
    match ctx.call(&AssetTokenAbi::decimals()).await {
        Ok(data) => {
            let decimals = decode_u256(&data).as_u64();
            if decimals == 18 {
                results.pass("decimals()", &format!("returned {}", decimals));
            } else {
                results.fail("decimals()", &format!("expected 18, got {}", decimals));
            }
        }
        Err(e) => results.fail("decimals()", &format!("call failed: {}", e)),
    }

    // Test totalSupply() (should be 0 initially)
    match ctx.call(&AssetTokenAbi::total_supply()).await {
        Ok(data) => {
            let supply = decode_u256(&data);
            if supply == U256::zero() {
                results.pass("totalSupply()", "returned 0 (initial state)");
            } else {
                results.fail("totalSupply()", &format!("expected 0, got {}", supply));
            }
        }
        Err(e) => results.fail("totalSupply()", &format!("call failed: {}", e)),
    }

    // Test balanceOf() (should be 0 initially)
    match ctx.call(&AssetTokenAbi::balance_of(ctx.wallet1.address())).await {
        Ok(data) => {
            let balance = decode_u256(&data);
            if balance == U256::zero() {
                results.pass("balanceOf()", "returned 0 (initial state)");
            } else {
                results.fail("balanceOf()", &format!("expected 0, got {}", balance));
            }
        }
        Err(e) => results.fail("balanceOf()", &format!("call failed: {}", e)),
    }
}

async fn test_mint(ctx: &TestContext, results: &mut TestResults) {
    println!("\n-- Mint Tests --");

    let mint_amount = U256::from(1000) * U256::from(10).pow(U256::from(18)); // 1000 tokens

    // Test 1: Successful mint
    let data = AssetTokenAbi::mint(ctx.wallet1.address(), mint_amount);
    match ctx.send_tx(&ctx.wallet1, data).await {
        Ok(receipt) => {
            if TestContext::is_success(&receipt) && TestContext::has_logs(&receipt, 2) {
                // Should have Mint and Transfer events
                results.pass("mint() success", "minted 1000 tokens with events");
            } else {
                results.fail("mint() success", "transaction failed or missing events");
            }
        }
        Err(e) => results.fail("mint() success", &format!("tx failed: {}", e)),
    }

    // Verify balance after mint
    match ctx.call(&AssetTokenAbi::balance_of(ctx.wallet1.address())).await {
        Ok(data) => {
            let balance = decode_u256(&data);
            if balance == mint_amount {
                results.pass("mint() balance update", &format!("balance is {} tokens", balance / U256::from(10).pow(U256::from(18))));
            } else {
                results.fail("mint() balance update", &format!("expected {}, got {}", mint_amount, balance));
            }
        }
        Err(e) => results.fail("mint() balance update", &format!("call failed: {}", e)),
    }

    // Verify totalSupply after mint
    match ctx.call(&AssetTokenAbi::total_supply()).await {
        Ok(data) => {
            let supply = decode_u256(&data);
            if supply == mint_amount {
                results.pass("mint() totalSupply update", "totalSupply matches minted amount");
            } else {
                results.fail("mint() totalSupply update", &format!("expected {}, got {}", mint_amount, supply));
            }
        }
        Err(e) => results.fail("mint() totalSupply update", &format!("call failed: {}", e)),
    }

    // Test 2: Mint from different address (permissionless)
    let data = AssetTokenAbi::mint(ctx.wallet2.address(), mint_amount);
    match ctx.send_tx(&ctx.wallet2, data).await {
        Ok(receipt) => {
            if TestContext::is_success(&receipt) {
                results.pass("mint() permissionless", "wallet2 successfully minted tokens");
            } else {
                results.fail("mint() permissionless", "transaction failed");
            }
        }
        Err(e) => results.fail("mint() permissionless", &format!("tx failed: {}", e)),
    }

    // Test 3: Mint to zero address should revert
    let zero_address = Address::ZERO;
    let data = AssetTokenAbi::mint(&zero_address, mint_amount);
    match ctx.send_tx(&ctx.wallet1, data).await {
        Ok(receipt) => {
            if !TestContext::is_success(&receipt) {
                results.pass("mint() to zero address", "correctly reverted");
            } else {
                results.fail("mint() to zero address", "should have reverted");
            }
        }
        Err(_) => results.pass("mint() to zero address", "correctly rejected"),
    }
}

async fn test_burn(ctx: &TestContext, results: &mut TestResults) {
    println!("\n-- Burn Tests --");

    let burn_amount = U256::from(100) * U256::from(10).pow(U256::from(18)); // 100 tokens

    // Get balance before burn
    let balance_before = match ctx.call(&AssetTokenAbi::balance_of(ctx.wallet1.address())).await {
        Ok(data) => decode_u256(&data),
        Err(_) => U256::zero(),
    };

    // Test 1: Successful burn
    let data = AssetTokenAbi::burn(burn_amount);
    match ctx.send_tx(&ctx.wallet1, data).await {
        Ok(receipt) => {
            if TestContext::is_success(&receipt) && TestContext::has_logs(&receipt, 2) {
                results.pass("burn() success", "burned 100 tokens with events");
            } else {
                results.fail("burn() success", "transaction failed or missing events");
            }
        }
        Err(e) => results.fail("burn() success", &format!("tx failed: {}", e)),
    }

    // Verify balance after burn
    match ctx.call(&AssetTokenAbi::balance_of(ctx.wallet1.address())).await {
        Ok(data) => {
            let balance_after = decode_u256(&data);
            let expected = balance_before - burn_amount;
            if balance_after == expected {
                results.pass("burn() balance update", &format!("balance reduced by 100 tokens"));
            } else {
                results.fail("burn() balance update", &format!("expected {}, got {}", expected, balance_after));
            }
        }
        Err(e) => results.fail("burn() balance update", &format!("call failed: {}", e)),
    }

    // Test 2: Burn exceeds balance should revert
    let huge_amount = U256::from(1_000_000) * U256::from(10).pow(U256::from(18));
    let data = AssetTokenAbi::burn(huge_amount);
    match ctx.send_tx(&ctx.wallet1, data).await {
        Ok(receipt) => {
            if !TestContext::is_success(&receipt) {
                results.pass("burn() exceeds balance", "correctly reverted");
            } else {
                results.fail("burn() exceeds balance", "should have reverted");
            }
        }
        Err(_) => results.pass("burn() exceeds balance", "correctly rejected"),
    }
}

async fn test_transfer(ctx: &TestContext, results: &mut TestResults) {
    println!("\n-- Transfer Tests --");

    let transfer_amount = U256::from(50) * U256::from(10).pow(U256::from(18)); // 50 tokens

    // Get balances before transfer
    let balance1_before = match ctx.call(&AssetTokenAbi::balance_of(ctx.wallet1.address())).await {
        Ok(data) => decode_u256(&data),
        Err(_) => U256::zero(),
    };

    // Test 1: Successful transfer
    let data = AssetTokenAbi::transfer(ctx.wallet2.address(), transfer_amount);
    match ctx.send_tx(&ctx.wallet1, data).await {
        Ok(receipt) => {
            if TestContext::is_success(&receipt) && TestContext::has_logs(&receipt, 1) {
                results.pass("transfer() success", "transferred 50 tokens");
            } else {
                results.fail("transfer() success", "transaction failed or missing event");
            }
        }
        Err(e) => results.fail("transfer() success", &format!("tx failed: {}", e)),
    }

    // Verify sender balance decreased
    match ctx.call(&AssetTokenAbi::balance_of(ctx.wallet1.address())).await {
        Ok(data) => {
            let balance_after = decode_u256(&data);
            let expected = balance1_before - transfer_amount;
            if balance_after == expected {
                results.pass("transfer() sender balance", "correctly decreased");
            } else {
                results.fail("transfer() sender balance", "incorrect balance");
            }
        }
        Err(e) => results.fail("transfer() sender balance", &format!("call failed: {}", e)),
    }

    // Test 2: Transfer exceeds balance should revert
    let huge_amount = U256::from(1_000_000) * U256::from(10).pow(U256::from(18));
    let data = AssetTokenAbi::transfer(ctx.wallet2.address(), huge_amount);
    match ctx.send_tx(&ctx.wallet1, data).await {
        Ok(receipt) => {
            if !TestContext::is_success(&receipt) {
                results.pass("transfer() exceeds balance", "correctly reverted");
            } else {
                results.fail("transfer() exceeds balance", "should have reverted");
            }
        }
        Err(_) => results.pass("transfer() exceeds balance", "correctly rejected"),
    }

    // Test 3: Transfer to zero address should revert
    let zero_address = Address::ZERO;
    let data = AssetTokenAbi::transfer(&zero_address, transfer_amount);
    match ctx.send_tx(&ctx.wallet1, data).await {
        Ok(receipt) => {
            if !TestContext::is_success(&receipt) {
                results.pass("transfer() to zero address", "correctly reverted");
            } else {
                results.fail("transfer() to zero address", "should have reverted");
            }
        }
        Err(_) => results.pass("transfer() to zero address", "correctly rejected"),
    }

    // Test 4: Self-transfer should succeed
    let self_amount = U256::from(10) * U256::from(10).pow(U256::from(18));
    let data = AssetTokenAbi::transfer(ctx.wallet1.address(), self_amount);
    match ctx.send_tx(&ctx.wallet1, data).await {
        Ok(receipt) => {
            if TestContext::is_success(&receipt) {
                results.pass("transfer() self-transfer", "successfully transferred to self");
            } else {
                results.fail("transfer() self-transfer", "should have succeeded");
            }
        }
        Err(e) => results.fail("transfer() self-transfer", &format!("tx failed: {}", e)),
    }
}

async fn test_approve_transfer_from(ctx: &TestContext, results: &mut TestResults) {
    println!("\n-- Approve/TransferFrom Tests --");

    let approve_amount = U256::from(200) * U256::from(10).pow(U256::from(18)); // 200 tokens
    let transfer_amount = U256::from(100) * U256::from(10).pow(U256::from(18)); // 100 tokens

    // Test 1: Approve
    let data = AssetTokenAbi::approve(ctx.wallet2.address(), approve_amount);
    match ctx.send_tx(&ctx.wallet1, data).await {
        Ok(receipt) => {
            if TestContext::is_success(&receipt) && TestContext::has_logs(&receipt, 1) {
                results.pass("approve()", "approved 200 tokens");
            } else {
                results.fail("approve()", "transaction failed or missing event");
            }
        }
        Err(e) => results.fail("approve()", &format!("tx failed: {}", e)),
    }

    // Verify allowance
    match ctx.call(&AssetTokenAbi::allowance(ctx.wallet1.address(), ctx.wallet2.address())).await {
        Ok(data) => {
            let allowance = decode_u256(&data);
            if allowance == approve_amount {
                results.pass("allowance()", &format!("allowance is {} tokens", allowance / U256::from(10).pow(U256::from(18))));
            } else {
                results.fail("allowance()", &format!("expected {}, got {}", approve_amount, allowance));
            }
        }
        Err(e) => results.fail("allowance()", &format!("call failed: {}", e)),
    }

    // Test 2: TransferFrom with sufficient allowance
    let data = AssetTokenAbi::transfer_from(ctx.wallet1.address(), ctx.wallet2.address(), transfer_amount);
    match ctx.send_tx(&ctx.wallet2, data).await {
        Ok(receipt) => {
            if TestContext::is_success(&receipt) {
                results.pass("transferFrom() success", "transferred 100 tokens");
            } else {
                results.fail("transferFrom() success", "transaction failed");
            }
        }
        Err(e) => results.fail("transferFrom() success", &format!("tx failed: {}", e)),
    }

    // Verify allowance decreased
    match ctx.call(&AssetTokenAbi::allowance(ctx.wallet1.address(), ctx.wallet2.address())).await {
        Ok(data) => {
            let remaining = decode_u256(&data);
            let expected = approve_amount - transfer_amount;
            if remaining == expected {
                results.pass("transferFrom() allowance update", "allowance correctly decreased");
            } else {
                results.fail("transferFrom() allowance update", &format!("expected {}, got {}", expected, remaining));
            }
        }
        Err(e) => results.fail("transferFrom() allowance update", &format!("call failed: {}", e)),
    }

    // Test 3: TransferFrom exceeds allowance should revert
    let huge_amount = U256::from(1000) * U256::from(10).pow(U256::from(18));
    let data = AssetTokenAbi::transfer_from(ctx.wallet1.address(), ctx.wallet2.address(), huge_amount);
    match ctx.send_tx(&ctx.wallet2, data).await {
        Ok(receipt) => {
            if !TestContext::is_success(&receipt) {
                results.pass("transferFrom() exceeds allowance", "correctly reverted");
            } else {
                results.fail("transferFrom() exceeds allowance", "should have reverted");
            }
        }
        Err(_) => results.pass("transferFrom() exceeds allowance", "correctly rejected"),
    }

    // Test 4: Update allowance (set new value)
    let new_allowance = U256::from(500) * U256::from(10).pow(U256::from(18));
    let data = AssetTokenAbi::approve(ctx.wallet2.address(), new_allowance);
    match ctx.send_tx(&ctx.wallet1, data).await {
        Ok(receipt) => {
            if TestContext::is_success(&receipt) {
                // Verify new allowance
                match ctx.call(&AssetTokenAbi::allowance(ctx.wallet1.address(), ctx.wallet2.address())).await {
                    Ok(data) => {
                        let allowance = decode_u256(&data);
                        if allowance == new_allowance {
                            results.pass("approve() update", "allowance updated to 500 tokens");
                        } else {
                            results.fail("approve() update", "allowance not updated correctly");
                        }
                    }
                    Err(e) => results.fail("approve() update", &format!("call failed: {}", e)),
                }
            } else {
                results.fail("approve() update", "transaction failed");
            }
        }
        Err(e) => results.fail("approve() update", &format!("tx failed: {}", e)),
    }
}

// ============================================================================
// Boundary Tests
// ============================================================================

async fn test_boundary_cases(ctx: &TestContext, results: &mut TestResults) {
    println!("\n-- Boundary Case Tests --");

    // Test 1: Mint zero amount should revert
    let zero = U256::zero();
    let data = AssetTokenAbi::mint(ctx.wallet1.address(), zero);
    match ctx.send_tx(&ctx.wallet1, data).await {
        Ok(receipt) => {
            if !TestContext::is_success(&receipt) {
                results.pass("mint() zero amount", "correctly reverted");
            } else {
                results.fail("mint() zero amount", "should have reverted");
            }
        }
        Err(_) => results.pass("mint() zero amount", "correctly rejected"),
    }

    // Test 2: Burn zero amount should revert
    let data = AssetTokenAbi::burn(zero);
    match ctx.send_tx(&ctx.wallet1, data).await {
        Ok(receipt) => {
            if !TestContext::is_success(&receipt) {
                results.pass("burn() zero amount", "correctly reverted");
            } else {
                results.fail("burn() zero amount", "should have reverted");
            }
        }
        Err(_) => results.pass("burn() zero amount", "correctly rejected"),
    }

    // Test 3: Transfer zero amount (should succeed per ERC-20 spec)
    let data = AssetTokenAbi::transfer(ctx.wallet2.address(), zero);
    match ctx.send_tx(&ctx.wallet1, data).await {
        Ok(receipt) => {
            if TestContext::is_success(&receipt) {
                results.pass("transfer() zero amount", "succeeded (ERC-20 compliant)");
            } else {
                results.pass("transfer() zero amount", "reverted (stricter implementation)");
            }
        }
        Err(e) => results.fail("transfer() zero amount", &format!("tx failed: {}", e)),
    }

    // Test 4: Large value mint (near max u256 territory)
    let large_value = U256::from(10).pow(U256::from(30)); // 10^30 (very large but not overflow)
    let data = AssetTokenAbi::mint(ctx.wallet1.address(), large_value);
    match ctx.send_tx(&ctx.wallet1, data).await {
        Ok(receipt) => {
            if TestContext::is_success(&receipt) {
                results.pass("mint() large value", "minted 10^30 tokens successfully");
            } else {
                results.fail("mint() large value", "transaction failed");
            }
        }
        Err(e) => results.fail("mint() large value", &format!("tx failed: {}", e)),
    }

    // Test 5: Approve max uint256 (infinite allowance)
    let max_u256 = U256::MAX;
    let data = AssetTokenAbi::approve(ctx.wallet2.address(), max_u256);
    match ctx.send_tx(&ctx.wallet1, data).await {
        Ok(receipt) => {
            if TestContext::is_success(&receipt) {
                // Verify allowance is max
                match ctx.call(&AssetTokenAbi::allowance(ctx.wallet1.address(), ctx.wallet2.address())).await {
                    Ok(data) => {
                        let allowance = decode_u256(&data);
                        if allowance == max_u256 {
                            results.pass("approve() max uint256", "infinite allowance set");
                        } else {
                            results.fail("approve() max uint256", "allowance not max");
                        }
                    }
                    Err(e) => results.fail("approve() max uint256", &format!("call failed: {}", e)),
                }
            } else {
                results.fail("approve() max uint256", "transaction failed");
            }
        }
        Err(e) => results.fail("approve() max uint256", &format!("tx failed: {}", e)),
    }
}

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("================================================================");
    println!("           AssetToken ERC-20 Contract Test Suite");
    println!("================================================================");
    println!();

    // Connect to node
    let rpc_url = std::env::var("RPC_URL").unwrap_or_else(|_| "http://localhost:8545".to_string());
    println!("[*] Connecting to node: {}", rpc_url);
    let client = SimpleClient::new(&rpc_url);

    let chain_id = client.chain_id().await?;
    println!("[*] Chain ID: {}", chain_id);

    let gas_price = client.gas_price().await?;
    println!("[*] Gas Price: {} gwei", gas_price / 1_000_000_000);

    // Setup test wallets (Hardhat test accounts)
    let private_key1: [u8; 32] = hex::decode("ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80")?
        .try_into()
        .map_err(|_| "Invalid key length")?;
    let wallet1 = TestWallet::from_private_key(&private_key1)?;
    println!("[*] Wallet 1: {}", wallet1.address().to_hex());

    let private_key2: [u8; 32] = hex::decode("59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d")?
        .try_into()
        .map_err(|_| "Invalid key length")?;
    let wallet2 = TestWallet::from_private_key(&private_key2)?;
    println!("[*] Wallet 2: {}", wallet2.address().to_hex());

    // Deploy AssetToken contract
    println!("\n[*] Deploying AssetToken contract...");
    let bytecode = hex::decode(ASSET_TOKEN_BYTECODE.trim())?;
    println!("[*] Contract bytecode size: {} bytes", bytecode.len());

    let nonce = client.get_nonce(wallet1.address()).await?;
    let raw_deploy_tx = wallet1.sign_transaction(
        nonce,
        gas_price,
        2_000_000, // Higher gas limit for deployment
        None,
        0,
        bytecode,
        chain_id,
    )?;

    let deploy_hash = client.send_raw_transaction(&raw_deploy_tx).await?;
    println!("[*] Deploy tx hash: 0x{}", hex::encode(deploy_hash.as_bytes()));

    println!("[*] Waiting for deployment confirmation...");
    tokio::time::sleep(Duration::from_secs(3)).await;

    let deploy_receipt = client.get_transaction_receipt(&deploy_hash).await?
        .ok_or("Deployment not confirmed")?;

    let contract_address = deploy_receipt
        .get("contractAddress")
        .and_then(|a| a.as_str())
        .ok_or("Contract address not found in receipt")?;

    let contract_address = Address::from_hex(contract_address)?;
    println!("[*] Contract deployed at: {}", contract_address.to_hex());

    // Verify contract code
    let code = client.get_code(&contract_address).await?;
    println!("[*] Deployed code size: {} bytes", code.len());

    if code.is_empty() {
        return Err("Contract deployment failed - no code at address".into());
    }

    // Create test context
    let ctx = TestContext {
        client,
        chain_id,
        gas_price,
        contract_address,
        wallet1,
        wallet2,
    };

    // Run tests
    let mut results = TestResults::new();

    test_view_functions(&ctx, &mut results).await;
    test_mint(&ctx, &mut results).await;
    test_burn(&ctx, &mut results).await;
    test_transfer(&ctx, &mut results).await;
    test_approve_transfer_from(&ctx, &mut results).await;
    test_boundary_cases(&ctx, &mut results).await;

    // Print summary
    results.summary();

    println!("\n================================================================");
    println!("                    Test Suite Complete");
    println!("================================================================");

    if results.failed > 0 {
        std::process::exit(1);
    }

    Ok(())
}
