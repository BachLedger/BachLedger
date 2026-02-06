//! Transaction commands

use bach_primitives::Address;
use bach_sdk::{TxBuilder, Wallet};
use clap::Subcommand;

use crate::{config::Config, output::Output, CliError};

/// Transaction subcommands
#[derive(Debug, Subcommand)]
pub enum TxCommand {
    /// Send ETH to an address
    Send {
        /// Recipient address
        #[arg(short, long)]
        to: String,
        /// Amount to send (in ETH)
        #[arg(short, long)]
        amount: String,
        /// Private key (hex) - for demo purposes
        #[arg(short, long)]
        key: String,
        /// Gas price in gwei
        #[arg(long, default_value = "1")]
        gas_price: u64,
        /// Gas limit
        #[arg(long, default_value = "21000")]
        gas_limit: u64,
        /// Nonce (auto-detect if not specified)
        #[arg(long)]
        nonce: Option<u64>,
    },
    /// Deploy a contract
    Deploy {
        /// Contract bytecode (hex)
        #[arg(short, long)]
        bytecode: String,
        /// Private key (hex)
        #[arg(short, long)]
        key: String,
        /// Gas limit
        #[arg(long, default_value = "1000000")]
        gas_limit: u64,
        /// Gas price in gwei
        #[arg(long, default_value = "1")]
        gas_price: u64,
    },
    /// Call a contract function
    Call {
        /// Contract address
        #[arg(short, long)]
        contract: String,
        /// Function selector or data (hex)
        #[arg(short, long)]
        data: String,
        /// Value to send (in ETH)
        #[arg(short, long, default_value = "0")]
        value: String,
        /// Private key (hex)
        #[arg(short, long)]
        key: String,
        /// Gas limit
        #[arg(long, default_value = "100000")]
        gas_limit: u64,
        /// Gas price in gwei
        #[arg(long, default_value = "1")]
        gas_price: u64,
    },
}

impl TxCommand {
    pub async fn execute(self, config: &Config, json: bool) -> Result<(), CliError> {
        match self {
            TxCommand::Send {
                to,
                amount,
                key,
                gas_price,
                gas_limit,
                nonce,
            } => send_tx(config, &to, &amount, &key, gas_price, gas_limit, nonce, json).await,
            TxCommand::Deploy {
                bytecode,
                key,
                gas_limit,
                gas_price,
            } => deploy_contract(config, &bytecode, &key, gas_limit, gas_price, json).await,
            TxCommand::Call {
                contract,
                data,
                value,
                key,
                gas_limit,
                gas_price,
            } => call_contract(config, &contract, &data, &value, &key, gas_limit, gas_price, json).await,
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn send_tx(
    config: &Config,
    to: &str,
    amount: &str,
    key: &str,
    gas_price_gwei: u64,
    gas_limit: u64,
    nonce: Option<u64>,
    json: bool,
) -> Result<(), CliError> {
    let to_addr = Address::from_hex(to).map_err(|e| CliError::InvalidAddress(e.to_string()))?;
    let wallet = Wallet::from_private_key_hex(key).map_err(|e| CliError::InvalidKey(e.to_string()))?;

    // Parse amount as ETH
    let value_wei = parse_eth_to_wei(amount)?;
    let gas_price = gas_price_gwei as u128 * 1_000_000_000; // Convert gwei to wei

    // Get nonce from network or use provided
    let client = bach_sdk::BachClient::new_mock();
    let nonce = match nonce {
        Some(n) => n,
        None => client
            .get_nonce(wallet.address(), bach_sdk::types::BlockId::Latest)
            .await
            .map_err(|e| CliError::Sdk(e.to_string()))?,
    };

    // Build and sign transaction
    let signed_tx = TxBuilder::new(config.chain_id)
        .nonce(nonce)
        .gas_limit(gas_limit)
        .gas_price(gas_price)
        .to(to_addr)
        .value(value_wei)
        .sign_legacy(&wallet)
        .map_err(|e| CliError::Sdk(e.to_string()))?;

    // Send transaction
    let pending = client
        .send_transaction(&signed_tx)
        .await
        .map_err(|e| CliError::Sdk(e.to_string()))?;

    Output::new(json)
        .field("tx_hash", &pending.hash().to_hex())
        .field("from", &wallet.address().to_hex())
        .field("to", &to_addr.to_hex())
        .field("value_wei", &format!("{}", value_wei))
        .field_u64("gas_limit", gas_limit)
        .field_u128("gas_price_wei", gas_price)
        .field_u64("nonce", nonce)
        .message(&format!("Transaction sent: {}", pending.hash().to_hex()))
        .print();

    Ok(())
}

async fn deploy_contract(
    config: &Config,
    bytecode: &str,
    key: &str,
    gas_limit: u64,
    gas_price_gwei: u64,
    json: bool,
) -> Result<(), CliError> {
    let wallet = Wallet::from_private_key_hex(key).map_err(|e| CliError::InvalidKey(e.to_string()))?;

    let bytecode_hex = bytecode.strip_prefix("0x").unwrap_or(bytecode);
    let bytecode_bytes = hex::decode(bytecode_hex).map_err(|e| CliError::InvalidHex(e.to_string()))?;

    let gas_price = gas_price_gwei as u128 * 1_000_000_000;

    let client = bach_sdk::BachClient::new_mock();
    let nonce = client
        .get_nonce(wallet.address(), bach_sdk::types::BlockId::Latest)
        .await
        .map_err(|e| CliError::Sdk(e.to_string()))?;

    let signed_tx = TxBuilder::new(config.chain_id)
        .nonce(nonce)
        .gas_limit(gas_limit)
        .gas_price(gas_price)
        .data(bytecode_bytes)
        .sign_legacy(&wallet)
        .map_err(|e| CliError::Sdk(e.to_string()))?;

    let pending = client
        .send_transaction(&signed_tx)
        .await
        .map_err(|e| CliError::Sdk(e.to_string()))?;

    Output::new(json)
        .field("tx_hash", &pending.hash().to_hex())
        .field("from", &wallet.address().to_hex())
        .field_u64("gas_limit", gas_limit)
        .message(&format!("Contract deployment tx: {}", pending.hash().to_hex()))
        .print();

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn call_contract(
    config: &Config,
    contract: &str,
    data: &str,
    value: &str,
    key: &str,
    gas_limit: u64,
    gas_price_gwei: u64,
    json: bool,
) -> Result<(), CliError> {
    let to_addr = Address::from_hex(contract).map_err(|e| CliError::InvalidAddress(e.to_string()))?;
    let wallet = Wallet::from_private_key_hex(key).map_err(|e| CliError::InvalidKey(e.to_string()))?;

    let data_hex = data.strip_prefix("0x").unwrap_or(data);
    let data_bytes = hex::decode(data_hex).map_err(|e| CliError::InvalidHex(e.to_string()))?;

    let value_wei = parse_eth_to_wei(value)?;
    let gas_price = gas_price_gwei as u128 * 1_000_000_000;

    let client = bach_sdk::BachClient::new_mock();
    let nonce = client
        .get_nonce(wallet.address(), bach_sdk::types::BlockId::Latest)
        .await
        .map_err(|e| CliError::Sdk(e.to_string()))?;

    let signed_tx = TxBuilder::new(config.chain_id)
        .nonce(nonce)
        .gas_limit(gas_limit)
        .gas_price(gas_price)
        .to(to_addr)
        .value(value_wei)
        .data(data_bytes)
        .sign_legacy(&wallet)
        .map_err(|e| CliError::Sdk(e.to_string()))?;

    let pending = client
        .send_transaction(&signed_tx)
        .await
        .map_err(|e| CliError::Sdk(e.to_string()))?;

    Output::new(json)
        .field("tx_hash", &pending.hash().to_hex())
        .field("from", &wallet.address().to_hex())
        .field("to", &to_addr.to_hex())
        .message(&format!("Contract call tx: {}", pending.hash().to_hex()))
        .print();

    Ok(())
}

fn parse_eth_to_wei(eth_str: &str) -> Result<u128, CliError> {
    // Simple ETH to wei conversion
    let parts: Vec<&str> = eth_str.split('.').collect();

    let whole = parts[0]
        .parse::<u128>()
        .map_err(|_| CliError::InvalidAmount("Invalid ETH amount".to_string()))?;

    let fractional = if parts.len() > 1 {
        let frac = parts[1];
        let padded = format!("{:0<18}", frac);
        let trimmed = &padded[..18];
        trimmed
            .parse::<u128>()
            .map_err(|_| CliError::InvalidAmount("Invalid decimal part".to_string()))?
    } else {
        0
    };

    Ok(whole * 1_000_000_000_000_000_000 + fractional)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_eth_to_wei() {
        assert_eq!(parse_eth_to_wei("1").unwrap(), 1_000_000_000_000_000_000);
        assert_eq!(parse_eth_to_wei("0.5").unwrap(), 500_000_000_000_000_000);
        assert_eq!(parse_eth_to_wei("0.1").unwrap(), 100_000_000_000_000_000);
        assert_eq!(parse_eth_to_wei("0").unwrap(), 0);
    }
}
