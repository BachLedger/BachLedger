//! Query commands

use bach_primitives::H256;
use clap::Subcommand;

use crate::{config::Config, output::Output, CliError};

/// Query subcommands
#[derive(Debug, Subcommand)]
pub enum QueryCommand {
    /// Query block information
    Block {
        /// Block number or "latest"
        #[arg(default_value = "latest")]
        block: String,
    },
    /// Query transaction by hash
    Tx {
        /// Transaction hash
        hash: String,
    },
    /// Query transaction receipt
    Receipt {
        /// Transaction hash
        hash: String,
    },
    /// Query chain ID
    ChainId,
    /// Query current gas price
    GasPrice,
    /// Query current block number
    BlockNumber,
}

impl QueryCommand {
    pub async fn execute(self, config: &Config, json: bool) -> Result<(), CliError> {
        match self {
            QueryCommand::Block { block } => query_block(config, &block, json).await,
            QueryCommand::Tx { hash } => query_tx(config, &hash, json).await,
            QueryCommand::Receipt { hash } => query_receipt(config, &hash, json).await,
            QueryCommand::ChainId => query_chain_id(config, json).await,
            QueryCommand::GasPrice => query_gas_price(config, json).await,
            QueryCommand::BlockNumber => query_block_number(config, json).await,
        }
    }
}

async fn query_block(_config: &Config, block: &str, json: bool) -> Result<(), CliError> {
    let client = bach_sdk::BachClient::new_mock();

    let block_id = if block == "latest" {
        bach_sdk::types::BlockId::Latest
    } else if block == "pending" {
        bach_sdk::types::BlockId::Pending
    } else if block == "earliest" {
        bach_sdk::types::BlockId::Earliest
    } else {
        let num = block
            .parse::<u64>()
            .map_err(|_| CliError::InvalidInput("Invalid block number".to_string()))?;
        bach_sdk::types::BlockId::Number(num)
    };

    let _block_data = client
        .get_block(block_id)
        .await
        .map_err(|e| CliError::Sdk(e.to_string()))?;

    // Mock response since block parsing isn't fully implemented
    Output::new(json)
        .field("block", block)
        .field("status", "mock")
        .message(&format!("Block query: {} (mock mode)", block))
        .print();

    Ok(())
}

async fn query_tx(_config: &Config, hash: &str, json: bool) -> Result<(), CliError> {
    let hash = H256::from_hex(hash).map_err(|e| CliError::InvalidHex(e.to_string()))?;

    let client = bach_sdk::BachClient::new_mock();
    let _tx = client
        .get_transaction(&hash)
        .await
        .map_err(|e| CliError::Sdk(e.to_string()))?;

    // Mock response
    Output::new(json)
        .field("hash", &hash.to_hex())
        .field("status", "mock")
        .message(&format!("Transaction query: {} (mock mode)", hash.to_hex()))
        .print();

    Ok(())
}

async fn query_receipt(_config: &Config, hash: &str, json: bool) -> Result<(), CliError> {
    let hash = H256::from_hex(hash).map_err(|e| CliError::InvalidHex(e.to_string()))?;

    let client = bach_sdk::BachClient::new_mock();
    let _receipt = client
        .get_receipt(&hash)
        .await
        .map_err(|e| CliError::Sdk(e.to_string()))?;

    // Mock response
    Output::new(json)
        .field("hash", &hash.to_hex())
        .field("status", "mock")
        .message(&format!("Receipt query: {} (mock mode)", hash.to_hex()))
        .print();

    Ok(())
}

async fn query_chain_id(_config: &Config, json: bool) -> Result<(), CliError> {
    let client = bach_sdk::BachClient::new_mock();
    let chain_id = client
        .chain_id()
        .await
        .map_err(|e| CliError::Sdk(e.to_string()))?;

    Output::new(json)
        .field_u64("chain_id", chain_id)
        .message(&format!("Chain ID: {}", chain_id))
        .print();

    Ok(())
}

async fn query_gas_price(_config: &Config, json: bool) -> Result<(), CliError> {
    let client = bach_sdk::BachClient::new_mock();
    let gas_price = client
        .gas_price()
        .await
        .map_err(|e| CliError::Sdk(e.to_string()))?;

    let gwei = gas_price / 1_000_000_000;

    Output::new(json)
        .field_u128("gas_price_wei", gas_price)
        .field_u128("gas_price_gwei", gwei)
        .message(&format!("Gas Price: {} gwei ({} wei)", gwei, gas_price))
        .print();

    Ok(())
}

async fn query_block_number(_config: &Config, json: bool) -> Result<(), CliError> {
    let client = bach_sdk::BachClient::new_mock();
    let block_number = client
        .block_number()
        .await
        .map_err(|e| CliError::Sdk(e.to_string()))?;

    Output::new(json)
        .field_u64("block_number", block_number)
        .message(&format!("Block Number: {}", block_number))
        .print();

    Ok(())
}
