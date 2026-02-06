//! Cryptographic material generation commands
//!
//! Generates all cryptographic materials needed for BachLedger clients and nodes:
//! - Private keys (secp256k1)
//! - Public keys (compressed and uncompressed)
//! - Ethereum-compatible addresses
//! - Encrypted keystore files (Web3 standard format)
//! - Node identity keys for P2P networking

use aes::cipher::{KeyIvInit, StreamCipher};
use bach_crypto::{keccak256, public_key_to_address};
use bach_primitives::Address;
use clap::Subcommand;
use k256::ecdsa::SigningKey;
use rand::{rngs::OsRng, RngCore};
use scrypt::{scrypt, Params as ScryptParams};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

use crate::{config::Config, output::Output, CliError};

/// Keygen subcommands
#[derive(Debug, Subcommand)]
pub enum KeygenCommand {
    /// Generate a new random keypair
    New {
        /// Output file path (saves keystore JSON if password provided, otherwise plain JSON)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Password for keystore encryption (enables encrypted keystore format)
        #[arg(short, long)]
        password: Option<String>,

        /// Account name/label
        #[arg(short, long)]
        name: Option<String>,

        /// Show private key in output (WARNING: security risk)
        #[arg(long)]
        show_private_key: bool,
    },

    /// Generate a node identity key for P2P networking
    NodeKey {
        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Batch generate multiple keypairs
    Batch {
        /// Number of keypairs to generate
        #[arg(short, long, default_value = "5")]
        count: usize,

        /// Output directory
        #[arg(short, long)]
        output_dir: Option<PathBuf>,

        /// Password for keystore encryption
        #[arg(short, long)]
        password: Option<String>,
    },

    /// Inspect a keystore file
    Inspect {
        /// Path to keystore file
        path: PathBuf,
    },

    /// Derive address from private key (for verification)
    Derive {
        /// Private key (hex)
        #[arg(short, long)]
        key: String,
    },

    /// Decrypt a keystore file and show the private key
    Decrypt {
        /// Path to keystore file
        path: PathBuf,

        /// Password for decryption
        #[arg(short, long)]
        password: String,
    },
}

impl KeygenCommand {
    pub async fn execute(self, config: &Config, json: bool) -> Result<(), CliError> {
        match self {
            KeygenCommand::New {
                output,
                password,
                name,
                show_private_key,
            } => generate_keypair(config, output, password, name, show_private_key, json).await,
            KeygenCommand::NodeKey { output } => generate_node_key(output, json).await,
            KeygenCommand::Batch {
                count,
                output_dir,
                password,
            } => batch_generate(count, output_dir, password, json).await,
            KeygenCommand::Inspect { path } => inspect_keystore(path, json).await,
            KeygenCommand::Derive { key } => derive_from_private_key(&key, json).await,
            KeygenCommand::Decrypt { path, password } => decrypt_keystore(path, &password, json).await,
        }
    }
}

/// Generated keypair with all derived materials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMaterial {
    /// Private key (32 bytes, hex)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_key: Option<String>,

    /// Public key (uncompressed, 65 bytes, hex)
    pub public_key_uncompressed: String,

    /// Public key (compressed, 33 bytes, hex)
    pub public_key_compressed: String,

    /// Ethereum-compatible address
    pub address: String,

    /// Checksum address (EIP-55)
    pub address_checksum: String,

    /// Node ID (for P2P, derived from public key)
    pub node_id: String,

    /// Optional name/label
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Web3 Keystore format (V3)
#[derive(Debug, Serialize, Deserialize)]
pub struct KeystoreV3 {
    /// Keystore version
    pub version: u32,

    /// Unique identifier
    pub id: String,

    /// Ethereum address (without 0x prefix)
    pub address: String,

    /// Crypto parameters
    pub crypto: KeystoreCrypto,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeystoreCrypto {
    /// Cipher algorithm (aes-128-ctr)
    pub cipher: String,

    /// Ciphertext (hex)
    pub ciphertext: String,

    /// Cipher parameters
    pub cipherparams: CipherParams,

    /// Key derivation function
    pub kdf: String,

    /// KDF parameters
    pub kdfparams: KdfParams,

    /// MAC (message authentication code)
    pub mac: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CipherParams {
    /// Initialization vector (hex)
    pub iv: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KdfParams {
    /// Derived key length
    pub dklen: u32,

    /// Scrypt N parameter (CPU/memory cost)
    pub n: u32,

    /// Scrypt r parameter (block size)
    pub r: u32,

    /// Scrypt p parameter (parallelization)
    pub p: u32,

    /// Salt (hex)
    pub salt: String,
}

/// Node identity key
#[derive(Debug, Serialize, Deserialize)]
pub struct NodeIdentity {
    /// Private key (hex)
    pub private_key: String,

    /// Public key (hex, compressed)
    pub public_key: String,

    /// Node ID (derived from public key, used for P2P)
    pub node_id: String,

    /// ENR (Ethereum Node Record) compatible ID
    pub enr_id: String,
}

/// Generate keypair with private key returned
fn generate_key_material_with_private_key(name: Option<String>) -> (KeyMaterial, [u8; 32]) {
    let private_key = SigningKey::random(&mut OsRng);
    let private_key_bytes: [u8; 32] = private_key.to_bytes().into();
    let public_key = private_key.verifying_key();

    // Get public key bytes
    let uncompressed = public_key.to_encoded_point(false);
    let compressed = public_key.to_encoded_point(true);

    // Derive address
    let address = public_key_to_address(public_key);

    // Node ID
    let node_id = keccak256(&uncompressed.as_bytes()[1..]);

    let material = KeyMaterial {
        private_key: None,
        public_key_uncompressed: format!("0x{}", hex::encode(uncompressed.as_bytes())),
        public_key_compressed: format!("0x{}", hex::encode(compressed.as_bytes())),
        address: address.to_hex(),
        address_checksum: to_checksum_address(&address),
        node_id: format!("0x{}", hex::encode(node_id.as_bytes())),
        name,
    };

    (material, private_key_bytes)
}

/// Create an encrypted keystore (Web3 V3 format)
fn create_keystore(private_key: &[u8; 32], password: &str, address: &Address) -> Result<KeystoreV3, CliError> {
    // Generate random salt and IV
    let mut salt = [0u8; 32];
    let mut iv = [0u8; 16];
    OsRng.fill_bytes(&mut salt);
    OsRng.fill_bytes(&mut iv);

    // Scrypt parameters (standard Web3 values)
    // N=262144 (2^18), r=8, p=1 for production
    // Using lower values for faster testing: N=8192 (2^13)
    let n = 8192u32; // Can be increased to 262144 for production
    let r = 8u32;
    let p = 1u32;
    let dklen = 32u32;

    // Derive key using scrypt
    let params = ScryptParams::new((n as f64).log2() as u8, r, p, dklen as usize)
        .map_err(|e| CliError::Crypto(format!("Invalid scrypt params: {}", e)))?;

    let mut derived_key = [0u8; 32];
    scrypt(password.as_bytes(), &salt, &params, &mut derived_key)
        .map_err(|e| CliError::Crypto(format!("Scrypt failed: {}", e)))?;

    // Split derived key: first 16 bytes for AES, last 16 bytes for MAC
    let aes_key = &derived_key[..16];
    let mac_key = &derived_key[16..];

    // Encrypt private key with AES-128-CTR
    let mut ciphertext = private_key.to_vec();
    type Aes128Ctr = ctr::Ctr64BE<aes::Aes128>;
    let mut cipher = Aes128Ctr::new(aes_key.into(), iv.as_slice().into());
    cipher.apply_keystream(&mut ciphertext);

    // Calculate MAC: keccak256(mac_key || ciphertext)
    let mut mac_data = Vec::with_capacity(16 + ciphertext.len());
    mac_data.extend_from_slice(mac_key);
    mac_data.extend_from_slice(&ciphertext);
    let mac = keccak256(&mac_data);

    Ok(KeystoreV3 {
        version: 3,
        id: Uuid::new_v4().to_string(),
        address: address.to_hex()[2..].to_lowercase(), // Without 0x prefix
        crypto: KeystoreCrypto {
            cipher: "aes-128-ctr".to_string(),
            ciphertext: hex::encode(&ciphertext),
            cipherparams: CipherParams {
                iv: hex::encode(iv),
            },
            kdf: "scrypt".to_string(),
            kdfparams: KdfParams {
                dklen,
                n,
                r,
                p,
                salt: hex::encode(salt),
            },
            mac: hex::encode(mac.as_bytes()),
        },
    })
}

/// Decrypt a keystore file
fn decrypt_keystore_file(keystore: &KeystoreV3, password: &str) -> Result<[u8; 32], CliError> {
    let salt = hex::decode(&keystore.crypto.kdfparams.salt)
        .map_err(|e| CliError::Crypto(format!("Invalid salt: {}", e)))?;
    let iv = hex::decode(&keystore.crypto.cipherparams.iv)
        .map_err(|e| CliError::Crypto(format!("Invalid IV: {}", e)))?;
    let ciphertext = hex::decode(&keystore.crypto.ciphertext)
        .map_err(|e| CliError::Crypto(format!("Invalid ciphertext: {}", e)))?;
    let expected_mac = hex::decode(&keystore.crypto.mac)
        .map_err(|e| CliError::Crypto(format!("Invalid MAC: {}", e)))?;

    // Derive key using scrypt
    let params = ScryptParams::new(
        (keystore.crypto.kdfparams.n as f64).log2() as u8,
        keystore.crypto.kdfparams.r,
        keystore.crypto.kdfparams.p,
        keystore.crypto.kdfparams.dklen as usize,
    )
    .map_err(|e| CliError::Crypto(format!("Invalid scrypt params: {}", e)))?;

    let mut derived_key = [0u8; 32];
    scrypt(password.as_bytes(), &salt, &params, &mut derived_key)
        .map_err(|e| CliError::Crypto(format!("Scrypt failed: {}", e)))?;

    // Verify MAC
    let mac_key = &derived_key[16..];
    let mut mac_data = Vec::with_capacity(16 + ciphertext.len());
    mac_data.extend_from_slice(mac_key);
    mac_data.extend_from_slice(&ciphertext);
    let computed_mac = keccak256(&mac_data);

    if computed_mac.as_bytes() != expected_mac.as_slice() {
        return Err(CliError::Crypto("Invalid password or corrupted keystore".to_string()));
    }

    // Decrypt
    let aes_key = &derived_key[..16];
    let mut plaintext = ciphertext;
    type Aes128Ctr = ctr::Ctr64BE<aes::Aes128>;
    let mut cipher = Aes128Ctr::new(aes_key.into(), iv.as_slice().into());
    cipher.apply_keystream(&mut plaintext);

    if plaintext.len() != 32 {
        return Err(CliError::Crypto("Decrypted key has invalid length".to_string()));
    }

    let mut private_key = [0u8; 32];
    private_key.copy_from_slice(&plaintext);
    Ok(private_key)
}

/// Convert address to EIP-55 checksum format
fn to_checksum_address(address: &Address) -> String {
    let addr_hex = hex::encode(address.as_bytes());
    let hash = keccak256(addr_hex.as_bytes());
    let hash_hex = hex::encode(hash.as_bytes());

    let mut result = String::with_capacity(42);
    result.push_str("0x");

    for (i, c) in addr_hex.chars().enumerate() {
        if c.is_ascii_digit() {
            result.push(c);
        } else {
            // Get the corresponding nibble from the hash
            let hash_char = hash_hex.chars().nth(i).unwrap();
            let nibble = u8::from_str_radix(&hash_char.to_string(), 16).unwrap();
            if nibble >= 8 {
                result.push(c.to_ascii_uppercase());
            } else {
                result.push(c.to_ascii_lowercase());
            }
        }
    }

    result
}

async fn generate_keypair(
    config: &Config,
    output: Option<PathBuf>,
    password: Option<String>,
    name: Option<String>,
    show_private_key: bool,
    json: bool,
) -> Result<(), CliError> {
    let (mut material, private_key_bytes) = generate_key_material_with_private_key(name.clone());
    let address = Address::from_hex(&material.address)
        .map_err(|e| CliError::Crypto(format!("Invalid address: {}", e)))?;

    // If output path is provided, save to file
    if let Some(path) = output {
        if let Some(password) = &password {
            // Save as encrypted keystore
            let keystore = create_keystore(&private_key_bytes, password, &address)?;
            let content = serde_json::to_string_pretty(&keystore)?;
            std::fs::write(&path, content)?;

            Output::new(json)
                .field("keystore_path", &path.display().to_string())
                .field("address", &material.address)
                .field("address_checksum", &material.address_checksum)
                .message(&format!(
                    "Keystore saved to: {}\nAddress: {}",
                    path.display(),
                    material.address_checksum
                ))
                .print();
        } else {
            // Save as plain JSON (includes private key)
            material.private_key = Some(format!("0x{}", hex::encode(private_key_bytes)));
            let content = serde_json::to_string_pretty(&material)?;
            std::fs::write(&path, content)?;

            Output::new(json)
                .field("path", &path.display().to_string())
                .field("address", &material.address)
                .message(&format!(
                    "Key material saved to: {}\nWARNING: File contains unencrypted private key!",
                    path.display()
                ))
                .print();
        }
    } else {
        // Output to console
        if show_private_key {
            material.private_key = Some(format!("0x{}", hex::encode(private_key_bytes)));
        }

        if json {
            println!("{}", serde_json::to_string_pretty(&material)?);
        } else {
            println!("╔══════════════════════════════════════════════════════════════════════╗");
            println!("║                    BachLedger Key Material                            ║");
            println!("╠══════════════════════════════════════════════════════════════════════╣");
            if let Some(ref name) = material.name {
                println!("║ Name:              {}  ", name);
            }
            if let Some(ref pk) = material.private_key {
                println!("║ Private Key:       {}  ", pk);
            } else {
                println!("║ Private Key:       [hidden - use --show-private-key to reveal]     ║");
            }
            println!("║                                                                       ║");
            println!("║ Public Key (uncompressed):                                            ║");
            println!("║   {}  ", &material.public_key_uncompressed[..68]);
            println!("║   {}  ", &material.public_key_uncompressed[68..]);
            println!("║                                                                       ║");
            println!("║ Public Key (compressed):                                              ║");
            println!("║   {}  ", material.public_key_compressed);
            println!("║                                                                       ║");
            println!("║ Address:           {}  ", material.address);
            println!("║ Address (EIP-55):  {}  ", material.address_checksum);
            println!("║                                                                       ║");
            println!("║ Node ID:           {}  ", &material.node_id[..50]);
            println!("║                    {}  ", &material.node_id[50..]);
            println!("╚══════════════════════════════════════════════════════════════════════╝");

            if material.private_key.is_some() {
                println!("\n⚠️  WARNING: Private key is displayed. Keep it secure!");
            }
        }
    }

    // Also save to keystore if configured
    if let Some(keystore_dir) = config.keystore_dir() {
        if let Some(name) = name {
            std::fs::create_dir_all(&keystore_dir)?;
            let path = keystore_dir.join(format!("{}.json", name));

            // Save metadata (not the encrypted key, just address info)
            let metadata = serde_json::json!({
                "name": name,
                "address": material.address,
                "address_checksum": material.address_checksum,
                "public_key_compressed": material.public_key_compressed,
            });
            std::fs::write(&path, serde_json::to_string_pretty(&metadata)?)?;
        }
    }

    Ok(())
}

async fn generate_node_key(output: Option<PathBuf>, json: bool) -> Result<(), CliError> {
    let private_key = SigningKey::random(&mut OsRng);
    let public_key = private_key.verifying_key();

    // Get compressed public key
    let compressed = public_key.to_encoded_point(true);
    let uncompressed = public_key.to_encoded_point(false);

    // Node ID is keccak256 of the uncompressed public key (without 04 prefix)
    let node_id = keccak256(&uncompressed.as_bytes()[1..]);

    // ENR compatible ID (first 8 bytes of node_id in hex)
    let enr_id = hex::encode(&node_id.as_bytes()[..8]);

    let identity = NodeIdentity {
        private_key: hex::encode(private_key.to_bytes()),
        public_key: hex::encode(compressed.as_bytes()),
        node_id: hex::encode(node_id.as_bytes()),
        enr_id,
    };

    if let Some(path) = output {
        let content = serde_json::to_string_pretty(&identity)?;
        std::fs::write(&path, content)?;

        Output::new(json)
            .field("path", &path.display().to_string())
            .field("node_id", &identity.node_id)
            .message(&format!("Node key saved to: {}", path.display()))
            .print();
    } else {
        if json {
            println!("{}", serde_json::to_string_pretty(&identity)?);
        } else {
            println!("╔══════════════════════════════════════════════════════════════════════╗");
            println!("║                    BachLedger Node Identity                           ║");
            println!("╠══════════════════════════════════════════════════════════════════════╣");
            println!("║ Private Key:       {}  ", identity.private_key);
            println!("║                                                                       ║");
            println!("║ Public Key:        {}  ", identity.public_key);
            println!("║                                                                       ║");
            println!("║ Node ID:           {}  ", &identity.node_id[..48]);
            println!("║                    {}  ", &identity.node_id[48..]);
            println!("║                                                                       ║");
            println!("║ ENR ID:            {}  ", identity.enr_id);
            println!("╚══════════════════════════════════════════════════════════════════════╝");
            println!("\n⚠️  WARNING: Private key is displayed. Keep it secure!");
        }
    }

    Ok(())
}

async fn batch_generate(
    count: usize,
    output_dir: Option<PathBuf>,
    password: Option<String>,
    json: bool,
) -> Result<(), CliError> {
    let mut results = Vec::with_capacity(count);

    for i in 0..count {
        let name = format!("account-{:03}", i);
        let (material, private_key_bytes) = generate_key_material_with_private_key(Some(name.clone()));
        let address = Address::from_hex(&material.address)
            .map_err(|e| CliError::Crypto(format!("Invalid address: {}", e)))?;

        if let Some(ref output_dir) = output_dir {
            std::fs::create_dir_all(output_dir)?;

            if let Some(ref password) = password {
                // Save as encrypted keystore
                let keystore = create_keystore(&private_key_bytes, password, &address)?;
                let filename = format!("{}--{}.json", chrono_timestamp(), material.address[2..].to_lowercase());
                let path = output_dir.join(&filename);
                let content = serde_json::to_string_pretty(&keystore)?;
                std::fs::write(&path, content)?;
            } else {
                // Save as plain JSON
                let mut material_with_key = material.clone();
                material_with_key.private_key = Some(format!("0x{}", hex::encode(private_key_bytes)));
                let path = output_dir.join(format!("{}.json", name));
                let content = serde_json::to_string_pretty(&material_with_key)?;
                std::fs::write(&path, content)?;
            }
        }

        results.push(serde_json::json!({
            "name": name,
            "address": material.address,
            "address_checksum": material.address_checksum,
        }));
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({
            "count": count,
            "accounts": results,
            "output_dir": output_dir.map(|p| p.display().to_string()),
        }))?);
    } else {
        println!("Generated {} accounts:", count);
        for account in &results {
            println!(
                "  {} - {}",
                account.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                account.get("address_checksum").and_then(|v| v.as_str()).unwrap_or("")
            );
        }
        if let Some(ref dir) = output_dir {
            println!("\nSaved to: {}", dir.display());
        }
    }

    Ok(())
}

/// Get timestamp in format used by Web3 keystores
fn chrono_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("UTC--{}Z", duration.as_secs())
}

async fn inspect_keystore(path: PathBuf, json: bool) -> Result<(), CliError> {
    let content = std::fs::read_to_string(&path)?;
    let keystore: KeystoreV3 = serde_json::from_str(&content)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({
            "version": keystore.version,
            "id": keystore.id,
            "address": format!("0x{}", keystore.address),
            "cipher": keystore.crypto.cipher,
            "kdf": keystore.crypto.kdf,
            "kdf_n": keystore.crypto.kdfparams.n,
            "kdf_r": keystore.crypto.kdfparams.r,
            "kdf_p": keystore.crypto.kdfparams.p,
        }))?);
    } else {
        println!("╔══════════════════════════════════════════════════════════════════════╗");
        println!("║                    Keystore Information                               ║");
        println!("╠══════════════════════════════════════════════════════════════════════╣");
        println!("║ Version:           {}  ", keystore.version);
        println!("║ ID:                {}  ", keystore.id);
        println!("║ Address:           0x{}  ", keystore.address);
        println!("║                                                                       ║");
        println!("║ Cipher:            {}  ", keystore.crypto.cipher);
        println!("║ KDF:               {}  ", keystore.crypto.kdf);
        println!("║ KDF Parameters:    n={}, r={}, p={}  ",
            keystore.crypto.kdfparams.n,
            keystore.crypto.kdfparams.r,
            keystore.crypto.kdfparams.p
        );
        println!("╚══════════════════════════════════════════════════════════════════════╝");
    }

    Ok(())
}

async fn derive_from_private_key(key: &str, json: bool) -> Result<(), CliError> {
    let key = key.strip_prefix("0x").unwrap_or(key);
    let key_bytes = hex::decode(key)
        .map_err(|e| CliError::InvalidKey(format!("Invalid hex: {}", e)))?;

    if key_bytes.len() != 32 {
        return Err(CliError::InvalidKey(format!(
            "Expected 32 bytes, got {}",
            key_bytes.len()
        )));
    }

    let private_key = SigningKey::from_slice(&key_bytes)
        .map_err(|e| CliError::InvalidKey(format!("Invalid key: {}", e)))?;
    let public_key = private_key.verifying_key();

    let uncompressed = public_key.to_encoded_point(false);
    let compressed = public_key.to_encoded_point(true);
    let address = public_key_to_address(public_key);
    let node_id = keccak256(&uncompressed.as_bytes()[1..]);

    let material = KeyMaterial {
        private_key: None,
        public_key_uncompressed: format!("0x{}", hex::encode(uncompressed.as_bytes())),
        public_key_compressed: format!("0x{}", hex::encode(compressed.as_bytes())),
        address: address.to_hex(),
        address_checksum: to_checksum_address(&address),
        node_id: format!("0x{}", hex::encode(node_id.as_bytes())),
        name: None,
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&material)?);
    } else {
        println!("Derived from private key:");
        println!("  Address:           {}", material.address);
        println!("  Address (EIP-55):  {}", material.address_checksum);
        println!("  Public Key (comp): {}", material.public_key_compressed);
        println!("  Node ID:           {}", material.node_id);
    }

    Ok(())
}

async fn decrypt_keystore(path: PathBuf, password: &str, json: bool) -> Result<(), CliError> {
    let content = std::fs::read_to_string(&path)?;
    let keystore: KeystoreV3 = serde_json::from_str(&content)?;

    let private_key = decrypt_keystore_file(&keystore, password)?;

    // Verify by deriving address
    let signing_key = SigningKey::from_slice(&private_key)
        .map_err(|e| CliError::Crypto(format!("Invalid decrypted key: {}", e)))?;
    let public_key = signing_key.verifying_key();
    let address = public_key_to_address(public_key);

    // Verify address matches
    let expected_address = format!("0x{}", keystore.address);
    if address.to_hex() != expected_address {
        return Err(CliError::Crypto(format!(
            "Address mismatch: expected {}, got {}",
            expected_address,
            address.to_hex()
        )));
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({
            "private_key": format!("0x{}", hex::encode(private_key)),
            "address": address.to_hex(),
            "verified": true,
        }))?);
    } else {
        println!("Decrypted keystore successfully!");
        println!("  Address:     {}", address.to_hex());
        println!("  Private Key: 0x{}", hex::encode(private_key));
        println!("\n⚠️  WARNING: Private key is displayed. Keep it secure!");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum_address() {
        // Known test case from EIP-55
        let addr = Address::from_hex("0xfb6916095ca1df60bb79ce92ce3ea74c37c5d359").unwrap();
        let checksum = to_checksum_address(&addr);
        assert_eq!(checksum, "0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359");
    }

    #[test]
    fn test_generate_key_material() {
        let (mut material, private_key) = generate_key_material_with_private_key(Some("test".to_string()));
        material.private_key = Some(format!("0x{}", hex::encode(private_key)));

        assert!(material.private_key.is_some());
        assert!(material.public_key_uncompressed.starts_with("0x04"));
        assert!(material.public_key_compressed.starts_with("0x02") ||
                material.public_key_compressed.starts_with("0x03"));
        assert!(material.address.starts_with("0x"));
        assert_eq!(material.address.len(), 42);
        assert!(material.node_id.starts_with("0x"));
    }

    #[test]
    fn test_keystore_roundtrip() {
        let (material, private_key) = generate_key_material_with_private_key(None);
        let address = Address::from_hex(&material.address).unwrap();
        let password = "test_password_123";

        let keystore = create_keystore(&private_key, password, &address).unwrap();
        let decrypted = decrypt_keystore_file(&keystore, password).unwrap();

        assert_eq!(private_key, decrypted);
    }

    #[test]
    fn test_keystore_wrong_password() {
        let (material, private_key) = generate_key_material_with_private_key(None);
        let address = Address::from_hex(&material.address).unwrap();
        let password = "correct_password";

        let keystore = create_keystore(&private_key, password, &address).unwrap();
        let result = decrypt_keystore_file(&keystore, "wrong_password");

        assert!(result.is_err());
    }

    #[test]
    fn test_known_address_derivation() {
        // Known test vector
        let key_hex = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
        let key_bytes = hex::decode(key_hex).unwrap();

        let private_key = SigningKey::from_slice(&key_bytes).unwrap();
        let public_key = private_key.verifying_key();
        let address = public_key_to_address(public_key);

        assert_eq!(address.to_hex(), "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266");
    }

    #[test]
    fn test_checksum_address_all_caps() {
        // All caps test case from EIP-55
        let addr = Address::from_hex("0x5aaeb6053f3e94c9b9a09f33669435e7ef1beaed").unwrap();
        let checksum = to_checksum_address(&addr);
        assert_eq!(checksum, "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed");
    }

    #[test]
    fn test_node_id_derivation() {
        let (material, _) = generate_key_material_with_private_key(None);

        // Node ID should be 32 bytes (64 hex chars + 0x)
        assert!(material.node_id.starts_with("0x"));
        assert_eq!(material.node_id.len(), 66);
    }
}
