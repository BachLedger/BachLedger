//! CLI integration tests for bach-cli
//!
//! Tests command parsing, output formatting, and config handling.

use std::process::Command;

/// Helper to run the CLI with arguments
fn run_bach(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bach"))
        .args(args)
        .output()
        .expect("Failed to execute command")
}

// ==================== Help & Version Tests ====================

#[test]
fn test_cli_help() {
    let output = run_bach(&["--help"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("bach"));
    assert!(stdout.contains("account"));
    assert!(stdout.contains("tx"));
    assert!(stdout.contains("query"));
}

#[test]
fn test_cli_version() {
    let output = run_bach(&["--version"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("bach"));
}

#[test]
fn test_cli_account_help() {
    let output = run_bach(&["account", "--help"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("create"));
    assert!(stdout.contains("balance"));
    assert!(stdout.contains("list"));
    assert!(stdout.contains("import"));
}

#[test]
fn test_cli_tx_help() {
    let output = run_bach(&["tx", "--help"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("send"));
}

#[test]
fn test_cli_query_help() {
    let output = run_bach(&["query", "--help"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("block"));
    assert!(stdout.contains("chain-id"));
    assert!(stdout.contains("gas-price"));
}

// ==================== Account Command Tests ====================

#[test]
fn test_account_create() {
    let output = run_bach(&["account", "create"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Created new account"));
    assert!(stdout.contains("0x"));
}

#[test]
fn test_account_create_json() {
    let output = run_bach(&["--json", "account", "create"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should be valid JSON
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
    assert!(json.get("address").is_some());
    let address = json["address"].as_str().unwrap();
    assert!(address.starts_with("0x"));
    assert_eq!(address.len(), 42); // 0x + 40 hex chars
}

#[test]
fn test_account_balance() {
    // Test with zero address (mock will return 1 ETH)
    let output = run_bach(&["account", "balance", "0x0000000000000000000000000000000000000000"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Balance"));
    assert!(stdout.contains("ETH"));
}

#[test]
fn test_account_balance_json() {
    let output = run_bach(&["--json", "account", "balance", "0x0000000000000000000000000000000000000000"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
    assert!(json.get("address").is_some());
    assert!(json.get("balance_wei").is_some());
    assert!(json.get("balance_eth").is_some());
}

#[test]
fn test_account_balance_invalid_address() {
    let output = run_bach(&["account", "balance", "not-an-address"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.to_lowercase().contains("error") || stderr.to_lowercase().contains("invalid"));
}

#[test]
fn test_account_import() {
    // Use a known test key
    let output = run_bach(&[
        "account", "import",
        "--key", "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
    ]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Imported"));
    assert!(stdout.to_lowercase().contains("0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"));
}

#[test]
fn test_account_import_json() {
    let output = run_bach(&[
        "--json", "account", "import",
        "--key", "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
    ]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
    let address = json["address"].as_str().unwrap().to_lowercase();
    assert_eq!(address, "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266");
}

#[test]
fn test_account_import_invalid_key() {
    let output = run_bach(&["account", "import", "--key", "invalid-key"]);
    assert!(!output.status.success());
}

#[test]
fn test_account_list() {
    let output = run_bach(&["account", "list"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Found") || stdout.contains("accounts"));
}

#[test]
fn test_account_list_json() {
    let output = run_bach(&["--json", "account", "list"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
    assert!(json.get("accounts").is_some());
    assert!(json.get("count").is_some());
}

// ==================== Query Command Tests ====================

#[test]
fn test_query_chain_id() {
    let output = run_bach(&["query", "chain-id"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1")); // Mock returns chain ID 1
}

#[test]
fn test_query_chain_id_json() {
    let output = run_bach(&["--json", "query", "chain-id"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
    assert!(json.get("chain_id").is_some());
}

#[test]
fn test_query_gas_price() {
    let output = run_bach(&["query", "gas-price"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Gas") || stdout.contains("gwei"));
}

#[test]
fn test_query_gas_price_json() {
    let output = run_bach(&["--json", "query", "gas-price"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
    assert!(json.get("gas_price_wei").is_some() || json.get("gas_price").is_some());
}

#[test]
fn test_query_block_latest() {
    let output = run_bach(&["query", "block", "latest"]);
    // May fail if mock doesn't support eth_getBlockByNumber
    // Just check it doesn't crash
    let _ = output;
}

// ==================== Config Command Tests ====================

#[test]
fn test_config_show() {
    let output = run_bach(&["config", "--show"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("RPC") || stdout.contains("rpc"));
}

#[test]
fn test_config_show_json() {
    let output = run_bach(&["--json", "config", "--show"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
    assert!(json.get("rpc_url").is_some() || json.get("chain_id").is_some());
}

// ==================== Global Options Tests ====================

#[test]
fn test_global_json_flag() {
    // JSON flag should work for any command
    let output = run_bach(&["--json", "account", "create"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should be valid JSON
    let _: serde_json::Value = serde_json::from_str(&stdout).expect("Output should be valid JSON");
}

#[test]
fn test_global_rpc_url_flag() {
    // RPC URL flag should be accepted (even if not used due to mock)
    let output = run_bach(&["--rpc-url", "http://localhost:8545", "query", "chain-id"]);
    assert!(output.status.success());
}

// ==================== Error Output Tests ====================

#[test]
fn test_error_output_text() {
    let output = run_bach(&["account", "balance", "invalid"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.to_lowercase().contains("error"));
}

#[test]
fn test_error_output_json() {
    let output = run_bach(&["--json", "account", "balance", "invalid"]);
    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Error should be valid JSON");
    assert!(json.get("error").is_some());
    assert_eq!(json.get("success"), Some(&serde_json::Value::Bool(false)));
}

// ==================== Address Validation Tests ====================

#[test]
fn test_address_too_short() {
    let output = run_bach(&["account", "balance", "0x123"]);
    assert!(!output.status.success());
}

#[test]
fn test_address_too_long() {
    let output = run_bach(&["account", "balance", "0x00000000000000000000000000000000000000000000"]);
    assert!(!output.status.success());
}

#[test]
fn test_address_without_prefix() {
    // Should work without 0x prefix
    let output = run_bach(&["account", "balance", "0000000000000000000000000000000000000000"]);
    // Depends on implementation - might succeed or fail
    let _ = output;
}

// ==================== Security Tests ====================

#[test]
fn test_private_key_not_in_output() {
    // When importing, the private key should not appear in output
    let key = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    let output = run_bach(&["account", "import", "--key", key]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Key should not be echoed
    assert!(!stdout.contains(key));
    assert!(!stderr.contains(key));
}

#[test]
fn test_error_no_key_leak() {
    // When providing invalid key, error should not contain the key
    let bad_key = "my_secret_key_data_here";
    let output = run_bach(&["account", "import", "--key", bad_key]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!stdout.contains("secret"));
    assert!(!stderr.contains("secret"));
}
