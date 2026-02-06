//! CLI configuration management

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// CLI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// RPC endpoint URL
    #[serde(default = "default_rpc_url")]
    pub rpc_url: String,
    /// Chain ID
    #[serde(default = "default_chain_id")]
    pub chain_id: u64,
    /// Default gas limit
    #[serde(default = "default_gas_limit")]
    pub gas_limit: u64,
    /// Keystore directory path
    #[serde(default)]
    pub keystore_dir: Option<PathBuf>,
}

fn default_rpc_url() -> String {
    "http://localhost:8545".to_string()
}

fn default_chain_id() -> u64 {
    1
}

fn default_gas_limit() -> u64 {
    21000
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rpc_url: default_rpc_url(),
            chain_id: default_chain_id(),
            gas_limit: default_gas_limit(),
            keystore_dir: None,
        }
    }
}

impl Config {
    /// Get the config directory path
    pub fn config_dir() -> Option<PathBuf> {
        dirs::home_dir().map(|h| h.join(".bachledger"))
    }

    /// Get the config file path
    pub fn config_path() -> Option<PathBuf> {
        Self::config_dir().map(|d| d.join("config.toml"))
    }

    /// Get the default keystore directory
    pub fn default_keystore_dir() -> Option<PathBuf> {
        Self::config_dir().map(|d| d.join("keystore"))
    }

    /// Load config from file or return default
    pub fn load() -> Self {
        Self::config_path()
            .and_then(|path| {
                if path.exists() {
                    std::fs::read_to_string(&path).ok()
                } else {
                    None
                }
            })
            .and_then(|content| toml::from_str(&content).ok())
            .unwrap_or_default()
    }

    /// Save config to file
    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = Self::config_path().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "Cannot determine config path")
        })?;

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
        })?;

        std::fs::write(path, content)
    }

    /// Get the keystore directory, using default if not configured
    pub fn keystore_dir(&self) -> Option<PathBuf> {
        self.keystore_dir.clone().or_else(Self::default_keystore_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.rpc_url, "http://localhost:8545");
        assert_eq!(config.chain_id, 1);
        assert_eq!(config.gas_limit, 21000);
    }

    #[test]
    fn test_config_serialize() {
        let config = Config::default();
        let toml = toml::to_string(&config).unwrap();
        assert!(toml.contains("rpc_url"));
        assert!(toml.contains("chain_id"));
    }

    #[test]
    fn test_config_deserialize() {
        let toml = r#"
            rpc_url = "http://example.com:8545"
            chain_id = 5
            gas_limit = 100000
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.rpc_url, "http://example.com:8545");
        assert_eq!(config.chain_id, 5);
        assert_eq!(config.gas_limit, 100000);
    }
}
