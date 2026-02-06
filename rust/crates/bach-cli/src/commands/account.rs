//! Account management commands

use bach_primitives::Address;
use bach_sdk::Wallet;
use clap::Subcommand;

use crate::{config::Config, output::Output, CliError};

/// Account subcommands
#[derive(Debug, Subcommand)]
pub enum AccountCommand {
    /// Create a new account
    Create {
        /// Save to keystore with this name
        #[arg(short, long)]
        name: Option<String>,
    },
    /// List accounts in keystore
    List,
    /// Get balance of an address
    Balance {
        /// Address to query
        address: String,
    },
    /// Import account from private key
    Import {
        /// Private key (hex)
        #[arg(short, long)]
        key: String,
        /// Name for the account
        #[arg(short, long)]
        name: Option<String>,
    },
}

impl AccountCommand {
    pub async fn execute(self, config: &Config, json: bool) -> Result<(), CliError> {
        match self {
            AccountCommand::Create { name } => create_account(config, name, json).await,
            AccountCommand::List => list_accounts(config, json).await,
            AccountCommand::Balance { address } => get_balance(config, &address, json).await,
            AccountCommand::Import { key, name } => import_account(config, &key, name, json).await,
        }
    }
}

async fn create_account(config: &Config, name: Option<String>, json: bool) -> Result<(), CliError> {
    let wallet = Wallet::new_random();
    let address = wallet.address().to_hex();

    if let Some(name) = &name {
        // Save to keystore
        if let Some(keystore_dir) = config.keystore_dir() {
            std::fs::create_dir_all(&keystore_dir)?;
            let path = keystore_dir.join(format!("{}.json", name));
            // For now, just save the address (proper keystore encryption would be needed)
            let content = serde_json::json!({
                "name": name,
                "address": address,
            });
            std::fs::write(&path, serde_json::to_string_pretty(&content)?)?;
        }
    }

    Output::new(json)
        .field("address", &address)
        .field("name", &name.unwrap_or_default())
        .message(&format!("Created new account: {}", address))
        .print();

    // Also print the private key warning
    if !json {
        println!("\nWARNING: Save your private key securely. It cannot be recovered!");
        println!("This is a demo - in production, use proper keystore encryption.");
    }

    Ok(())
}

async fn list_accounts(config: &Config, json: bool) -> Result<(), CliError> {
    let mut accounts = Vec::new();

    if let Some(keystore_dir) = config.keystore_dir() {
        if keystore_dir.exists() {
            for entry in std::fs::read_dir(&keystore_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "json") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                            accounts.push(serde_json::json!({
                                "name": data.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                                "address": data.get("address").and_then(|v| v.as_str()).unwrap_or(""),
                            }));
                        }
                    }
                }
            }
        }
    }

    Output::new(json)
        .field_value("accounts", serde_json::Value::Array(accounts.clone()))
        .field_u64("count", accounts.len() as u64)
        .message(&format!("Found {} accounts", accounts.len()))
        .print();

    if !json && !accounts.is_empty() {
        println!("\nAccounts:");
        for account in &accounts {
            println!(
                "  {} - {}",
                account.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                account.get("address").and_then(|v| v.as_str()).unwrap_or("")
            );
        }
    }

    Ok(())
}

async fn get_balance(_config: &Config, address: &str, json: bool) -> Result<(), CliError> {
    let address = Address::from_hex(address).map_err(|e| CliError::InvalidAddress(e.to_string()))?;

    // For now, use mock client since we don't have a real node
    let client = bach_sdk::BachClient::new_mock();
    let balance = client
        .get_balance(&address, bach_sdk::types::BlockId::Latest)
        .await
        .map_err(|e| CliError::Sdk(e.to_string()))?;

    // Convert to ETH (divide by 10^18)
    let eth_str = format_wei_to_eth(&balance);

    Output::new(json)
        .field("address", &address.to_hex())
        .field("balance_wei", &format!("{:?}", balance))
        .field("balance_eth", &eth_str)
        .message(&format!("Balance: {} ETH", eth_str))
        .print();

    Ok(())
}

async fn import_account(
    config: &Config,
    key: &str,
    name: Option<String>,
    json: bool,
) -> Result<(), CliError> {
    let wallet =
        Wallet::from_private_key_hex(key).map_err(|e| CliError::InvalidKey(e.to_string()))?;
    let address = wallet.address().to_hex();

    let name = name.unwrap_or_else(|| {
        // Generate a name from address
        format!("account-{}", &address[2..10])
    });

    if let Some(keystore_dir) = config.keystore_dir() {
        std::fs::create_dir_all(&keystore_dir)?;
        let path = keystore_dir.join(format!("{}.json", name));
        let content = serde_json::json!({
            "name": name,
            "address": address,
        });
        std::fs::write(&path, serde_json::to_string_pretty(&content)?)?;
    }

    Output::new(json)
        .field("address", &address)
        .field("name", &name)
        .message(&format!("Imported account: {} as '{}'", address, name))
        .print();

    Ok(())
}

fn format_wei_to_eth(wei: &bach_primitives::U256) -> String {
    // Simple conversion - divide by 10^18
    // This is a simplified version; proper decimal handling would be better
    let divisor = bach_primitives::U256::from(1_000_000_000_000_000_000u128);
    let eth = *wei / divisor;
    let remainder = *wei % divisor;

    if remainder.is_zero() {
        format!("{}", eth)
    } else {
        // Show up to 6 decimal places
        let remainder_str = format!("{:018}", remainder.low_u64());
        let trimmed = remainder_str.trim_end_matches('0');
        let decimals = if trimmed.len() > 6 {
            &trimmed[..6]
        } else {
            trimmed
        };
        format!("{}.{}", eth, decimals)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_wei_to_eth() {
        let one_eth = bach_primitives::U256::from(1_000_000_000_000_000_000u128);
        assert_eq!(format_wei_to_eth(&one_eth), "1");

        let half_eth = bach_primitives::U256::from(500_000_000_000_000_000u128);
        assert_eq!(format_wei_to_eth(&half_eth), "0.5");
    }
}
