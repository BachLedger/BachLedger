//! 手动测试脚本 - 完整的端到端测试流程
//!
//! 运行方式:
//! 1. 先启动节点: cargo run -p bach-node --release -- --datadir ./testdata --chain-id 1337
//! 2. 运行测试: cargo run --example manual_test --release
//!
//! 或者使用脚本:
//!   ./scripts/run_manual_test.sh
//!
//! 测试内容:
//! - 连接节点
//! - 查询账户状态
//! - 发送 ETH 转账
//! - 部署合约
//! - 调用合约
//! - 查询事件日志

use bach_crypto::keccak256;
use bach_primitives::{Address, H256, U256};
use k256::ecdsa::SigningKey;
use std::time::Duration;

/// RPC 客户端 (简化版)
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

    async fn block_number(&self) -> Result<u64, String> {
        let hex: String = self.call("eth_blockNumber", vec![]).await?;
        parse_hex_u64(&hex)
    }

    async fn gas_price(&self) -> Result<u128, String> {
        let hex: String = self.call("eth_gasPrice", vec![]).await?;
        parse_hex_u128(&hex)
    }

    async fn get_balance(&self, address: &Address) -> Result<U256, String> {
        let hex: String = self
            .call(
                "eth_getBalance",
                vec![
                    serde_json::Value::String(address.to_hex()),
                    serde_json::Value::String("latest".to_string()),
                ],
            )
            .await?;
        parse_hex_u256(&hex)
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

fn parse_hex_u256(hex: &str) -> Result<U256, String> {
    let hex = hex.strip_prefix("0x").unwrap_or(hex);
    U256::from_str_radix(hex, 16).map_err(|e| format!("Invalid hex U256: {:?}", e))
}

fn parse_hex_h256(hex: &str) -> Result<H256, String> {
    H256::from_hex(hex).map_err(|e| format!("Invalid hex H256: {:?}", e))
}

fn parse_hex_bytes(hex: &str) -> Result<Vec<u8>, String> {
    let hex = hex.strip_prefix("0x").unwrap_or(hex);
    hex::decode(hex).map_err(|e| format!("Invalid hex bytes: {}", e))
}

/// 测试钱包
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
        let public_key_uncompressed = &public_key_bytes.as_bytes()[1..]; // Skip 0x04 prefix
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

    /// Sign and encode a legacy transaction (EIP-155)
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
        // Build RLP for signing: [nonce, gas_price, gas_limit, to, value, data, chain_id, 0, 0]
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

        // Sign
        let (signature, recovery_id) = self
            .signing_key
            .sign_prehash_recoverable(msg_hash.as_bytes())
            .map_err(|e| format!("Signing failed: {}", e))?;

        let sig_bytes = signature.to_bytes();
        let r = &sig_bytes[0..32];
        let s = &sig_bytes[32..64];

        // EIP-155: v = recovery_id + chain_id * 2 + 35
        let v = recovery_id.to_byte() as u64 + chain_id * 2 + 35;

        // Encode signed transaction: [nonce, gas_price, gas_limit, to, value, data, v, r, s]
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

/// RLP encoding helper
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
                out.push(0x80); // empty string
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

/// Encode fixed bytes (like r, s) - strip leading zeros
fn encode_rlp_fixed_bytes(data: &[u8], out: &mut Vec<u8>) {
    // Strip leading zeros for signature components
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

/// SimpleStorage 合约字节码
/// contract SimpleStorage {
///     uint256 private value;
///     event ValueChanged(uint256 newValue);
///     function set(uint256 _value) public { value = _value; emit ValueChanged(_value); }
///     function get() public view returns (uint256) { return value; }
/// }
const SIMPLE_STORAGE_BYTECODE: &str = "608060405234801561001057600080fd5b5060df8061001f6000396000f3fe6080604052348015600f57600080fd5b5060043610603c5760003560e01c806360fe47b11460415780636d4ce63c146053575b600080fd5b6051604c3660046085565b606d565b005b60005460405190815260200160405180910390f35b60008190556040518181527f93fe6d397c74fdf1402a8b72e47b68512f0510d7b98a4bc4cbdf6ac7108b3c599060200160405180910390a150565b600060208284031215609657600080fd5b503591905056fea2646970667358221220";

/// 编码 set(uint256) 函数调用
fn encode_set_call(value: u64) -> Vec<u8> {
    // function selector: keccak256("set(uint256)")[:4] = 0x60fe47b1
    let mut data = vec![0x60, 0xfe, 0x47, 0xb1];
    // uint256 参数 (32 bytes, big endian)
    let mut value_bytes = [0u8; 32];
    value_bytes[24..].copy_from_slice(&value.to_be_bytes());
    data.extend_from_slice(&value_bytes);
    data
}

/// 编码 get() 函数调用
fn encode_get_call() -> Vec<u8> {
    // function selector: keccak256("get()")[:4] = 0x6d4ce63c
    vec![0x6d, 0x4c, 0xe6, 0x3c]
}

/// 解码 uint256 返回值
fn decode_uint256(data: &[u8]) -> u64 {
    if data.len() < 32 {
        return 0;
    }
    // 取最后 8 字节作为 u64
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&data[24..32]);
    u64::from_be_bytes(bytes)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║           BachLedger 手动测试脚本                            ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    // 连接到节点
    let rpc_url = std::env::var("RPC_URL").unwrap_or_else(|_| "http://localhost:8545".to_string());
    println!("🔗 连接到节点: {}", rpc_url);
    let client = SimpleClient::new(&rpc_url);

    // ==================== 1. 基本连接测试 ====================
    println!("\n📋 1. 基本连接测试");
    println!("─────────────────────────────────────────");

    let chain_id = client.chain_id().await?;
    println!("   Chain ID: {}", chain_id);

    let block_number = client.block_number().await?;
    println!("   区块高度: {}", block_number);

    let gas_price = client.gas_price().await?;
    println!("   Gas 价格: {} wei ({} gwei)", gas_price, gas_price / 1_000_000_000);

    // ==================== 2. 账户状态查询 ====================
    println!("\n📋 2. 账户状态查询");
    println!("─────────────────────────────────────────");

    // 使用 Hardhat 测试账户 #0
    let private_key_hex = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    let private_key_bytes: [u8; 32] = hex::decode(private_key_hex)?
        .try_into()
        .map_err(|_| "Invalid key length")?;

    let wallet = TestWallet::from_private_key(&private_key_bytes)?;
    println!("   测试账户: {}", wallet.address().to_hex());

    let balance = client.get_balance(wallet.address()).await?;
    let balance_eth = balance.as_u128() as f64 / 1e18;
    println!("   余额: {} ETH", balance_eth);

    let nonce = client.get_nonce(wallet.address()).await?;
    println!("   Nonce: {}", nonce);

    // ==================== 3. ETH 转账测试 ====================
    println!("\n📋 3. ETH 转账测试");
    println!("─────────────────────────────────────────");

    // 目标地址: Hardhat 测试账户 #1
    let to_address = Address::from_hex("0x70997970C51812dc3A010C7d01b50e0d17dc79C8")?;
    let transfer_value = 1_000_000_000_000_000_000u128; // 1 ETH

    println!("   发送 1 ETH 到: {}", to_address.to_hex());

    let raw_tx = wallet.sign_transaction(
        nonce,
        gas_price,
        21000,
        Some(to_address),
        transfer_value,
        vec![],
        chain_id,
    )?;
    println!("   交易已签名, 大小: {} bytes", raw_tx.len());

    let tx_hash = client.send_raw_transaction(&raw_tx).await?;
    println!("   交易哈希: 0x{}", hex::encode(tx_hash.as_bytes()));

    // 等待交易被打包
    println!("   等待交易确认...");
    tokio::time::sleep(Duration::from_secs(2)).await;

    let receipt = client.get_transaction_receipt(&tx_hash).await?;
    if let Some(receipt) = receipt {
        println!("   ✅ 交易已确认!");
        if let Some(status) = receipt.get("status") {
            println!("   状态: {}", status);
        }
        if let Some(gas_used) = receipt.get("gasUsed") {
            println!("   Gas 使用: {}", gas_used);
        }
    } else {
        println!("   ⏳ 交易待确认");
    }

    // 验证余额变化
    let new_balance = client.get_balance(&to_address).await?;
    println!("   接收方新余额: {} ETH", new_balance.as_u128() as f64 / 1e18);

    // ==================== 4. 合约部署 ====================
    println!("\n📋 4. 合约部署");
    println!("─────────────────────────────────────────");

    let bytecode = hex::decode(SIMPLE_STORAGE_BYTECODE)?;
    println!("   合约字节码大小: {} bytes", bytecode.len());

    let nonce = client.get_nonce(wallet.address()).await?;

    let raw_deploy_tx = wallet.sign_transaction(
        nonce,
        gas_price,
        500_000,
        None, // 合约创建
        0,
        bytecode,
        chain_id,
    )?;
    println!("   部署交易已签名");

    let deploy_hash = client.send_raw_transaction(&raw_deploy_tx).await?;
    println!("   部署交易哈希: 0x{}", hex::encode(deploy_hash.as_bytes()));

    // 等待部署完成
    println!("   等待部署确认...");
    tokio::time::sleep(Duration::from_secs(2)).await;

    let deploy_receipt = client.get_transaction_receipt(&deploy_hash).await?;
    let contract_address = if let Some(receipt) = deploy_receipt {
        if let Some(addr) = receipt.get("contractAddress") {
            let addr_str = addr.as_str().unwrap_or("");
            let addr = Address::from_hex(addr_str)?;
            println!("   ✅ 合约部署成功!");
            println!("   合约地址: {}", addr_str);
            Some(addr)
        } else {
            println!("   ❌ 合约地址未找到");
            None
        }
    } else {
        println!("   ⏳ 部署交易待确认");
        None
    };

    // ==================== 5. 合约交互 ====================
    if let Some(contract_addr) = contract_address {
        println!("\n📋 5. 合约交互");
        println!("─────────────────────────────────────────");

        // 验证合约代码已部署
        let code = client.get_code(&contract_addr).await?;
        println!("   合约代码大小: {} bytes", code.len());

        // 调用 set(42)
        println!("   调用 set(42)...");
        let nonce = client.get_nonce(wallet.address()).await?;
        let set_data = encode_set_call(42);

        let raw_set_tx = wallet.sign_transaction(
            nonce,
            gas_price,
            100_000,
            Some(contract_addr),
            0,
            set_data,
            chain_id,
        )?;

        let set_hash = client.send_raw_transaction(&raw_set_tx).await?;
        println!("   set() 交易哈希: 0x{}", hex::encode(set_hash.as_bytes()));

        tokio::time::sleep(Duration::from_secs(2)).await;

        let set_receipt = client.get_transaction_receipt(&set_hash).await?;
        if let Some(receipt) = set_receipt {
            println!("   ✅ set() 调用成功!");
            if let Some(logs) = receipt.get("logs") {
                if let Some(logs_arr) = logs.as_array() {
                    println!("   事件日志数量: {}", logs_arr.len());
                }
            }
        }

        // 调用 get() (只读)
        println!("   调用 get()...");
        let get_data = encode_get_call();
        let result = client.call_contract(&contract_addr, &get_data).await?;
        let stored_value = decode_uint256(&result);
        println!("   ✅ get() 返回值: {}", stored_value);

        // 再次设置新值
        println!("   调用 set(100)...");
        let nonce = client.get_nonce(wallet.address()).await?;
        let set_data = encode_set_call(100);

        let raw_set_tx = wallet.sign_transaction(
            nonce,
            gas_price,
            100_000,
            Some(contract_addr),
            0,
            set_data,
            chain_id,
        )?;

        let _ = client.send_raw_transaction(&raw_set_tx).await?;

        tokio::time::sleep(Duration::from_secs(2)).await;

        // 验证新值
        let result = client.call_contract(&contract_addr, &encode_get_call()).await?;
        let stored_value = decode_uint256(&result);
        println!("   ✅ 新的 get() 返回值: {}", stored_value);
    }

    // ==================== 6. 最终状态 ====================
    println!("\n📋 6. 最终状态");
    println!("─────────────────────────────────────────");

    let final_block = client.block_number().await?;
    println!("   最终区块高度: {}", final_block);

    let final_balance = client.get_balance(wallet.address()).await?;
    println!("   测试账户最终余额: {} ETH", final_balance.as_u128() as f64 / 1e18);

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                    测试完成!                                 ║");
    println!("╚══════════════════════════════════════════════════════════════╝");

    Ok(())
}
