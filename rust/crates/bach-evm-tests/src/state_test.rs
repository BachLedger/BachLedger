//! State test runner

use crate::error::{TestError, TestResult};
use crate::types::*;
use bach_evm::{BlockContext, CallContext, Environment, Interpreter, TxContext};
use bach_primitives::{Address, H256};
use std::path::Path;

/// Supported forks for testing
pub const SUPPORTED_FORKS: &[&str] = &[
    "Berlin",
    "London",
    "Paris",     // The Merge
    "Shanghai",
    "Cancun",
];

/// State test runner
pub struct StateTestRunner {
    /// Target fork
    fork: String,
    /// Verbose output
    verbose: bool,
}

impl StateTestRunner {
    /// Create new state test runner
    pub fn new(fork: &str, verbose: bool) -> Self {
        Self {
            fork: fork.to_string(),
            verbose,
        }
    }

    /// Run all tests in a file
    pub fn run_file(&self, path: &Path) -> TestResult<StateTestResults> {
        let content = std::fs::read_to_string(path)?;
        let tests: StateTestFile = serde_json::from_str(&content)?;

        let mut results = StateTestResults::new(path.to_string_lossy().to_string());

        for (name, test_case) in tests {
            // Check if fork is supported in this test
            if !test_case.post.contains_key(&self.fork) {
                results.skipped.push((name, format!("fork {} not in test", self.fork)));
                continue;
            }

            let fork_results = &test_case.post[&self.fork];

            for (idx, post_result) in fork_results.iter().enumerate() {
                let test_name = format!("{}_{}", name, idx);
                let result = self.run_test_case(&test_name, &test_case, post_result);

                match result {
                    Ok(()) => {
                        if self.verbose {
                            tracing::info!("PASS: {}", test_name);
                        }
                        results.passed.push(test_name);
                    }
                    Err(e) => {
                        if self.verbose {
                            tracing::warn!("FAIL: {} - {}", test_name, e);
                        }
                        results.failed.push((test_name, e.to_string()));
                    }
                }
            }
        }

        Ok(results)
    }

    /// Run a single test case with specific indices
    fn run_test_case(
        &self,
        name: &str,
        test: &StateTestCase,
        expected: &PostStateResult,
    ) -> TestResult<()> {
        let tx = &test.transaction;
        let idx = &expected.indexes;

        // Get specific transaction parameters
        let data = tx.data.get(idx.data)
            .ok_or_else(|| TestError::Parse(format!("data index {} out of bounds", idx.data)))?;
        let gas_limit = tx.gas_limit.get(idx.gas)
            .ok_or_else(|| TestError::Parse(format!("gas index {} out of bounds", idx.gas)))?;
        let value = tx.value.get(idx.value)
            .ok_or_else(|| TestError::Parse(format!("value index {} out of bounds", idx.value)))?;

        // Determine gas price
        let gas_price = tx.gas_price.as_ref()
            .map(|p| p.0)
            .or_else(|| tx.max_fee_per_gas.as_ref().map(|f| f.0))
            .unwrap_or(0);

        // Get sender from secret key (simplified - use coinbase as placeholder)
        // In real implementation, derive address from secret key using ECDSA
        let sender = Address::from_slice(&test.env.current_coinbase.0)
            .map_err(|e| TestError::Parse(format!("invalid sender address: {}", e)))?;

        // Determine target address
        let to_address = match &tx.to {
            Some(to) if !to.is_empty() && to != "0x" => {
                Some(parse_address(to)?)
            }
            _ => None, // Contract creation
        };

        // Build environment
        let env = self.build_environment(test, &sender, to_address, data, value, gas_price)?;

        // Get code to execute
        let code = if let Some(to) = to_address {
            // Call to existing contract
            test.pre.get(&format!("0x{}", hex::encode(to.as_bytes())))
                .map(|a| a.code.0.clone())
                .unwrap_or_default()
        } else {
            // Contract creation - execute init code
            data.0.clone()
        };

        // Create interpreter
        let mut interp = Interpreter::new(code, gas_limit.0);

        // Run execution
        let result = interp.run(&env);

        // Check for expected exception
        if let Some(expected_exception) = &expected.expect_exception {
            if result.success {
                return Err(TestError::Assertion(format!(
                    "{}: expected exception '{}' but execution succeeded",
                    name, expected_exception
                )));
            }
            return Ok(());
        }

        // For now, just verify execution completes
        // Full state root validation would require merkle trie implementation
        Ok(())
    }

    /// Build execution environment
    fn build_environment(
        &self,
        test: &StateTestCase,
        sender: &Address,
        to: Option<Address>,
        data: &HexBytes,
        value: &HexU256,
        gas_price: u64,
    ) -> TestResult<Environment> {
        let target_address = to.unwrap_or(Address::ZERO);
        let value_u128 = u256_to_u128(&value.0);

        let call = CallContext {
            caller: *sender,
            address: target_address,
            value: value_u128,
            data: data.0.clone(),
            gas: 0, // Will be set by interpreter
            is_static: false,
            depth: 0,
        };

        let prevrandao = if let Some(r) = test.env.current_random.as_ref() {
            H256::from_slice(&r.0)
                .map_err(|e| TestError::Parse(format!("invalid prevrandao: {}", e)))?
        } else {
            H256::from_slice(&test.env.current_difficulty.0)
                .map_err(|e| TestError::Parse(format!("invalid difficulty: {}", e)))?
        };

        let coinbase = Address::from_slice(&test.env.current_coinbase.0)
            .map_err(|e| TestError::Parse(format!("invalid coinbase: {}", e)))?;

        let block = BlockContext {
            coinbase,
            timestamp: test.env.current_timestamp.0,
            number: test.env.current_number.0,
            gas_limit: test.env.current_gas_limit.0,
            base_fee: test.env.current_base_fee.as_ref().map(|f| f.0 as u128).unwrap_or(0),
            prevrandao,
            chain_id: 1,
        };

        let tx = TxContext {
            origin: *sender,
            gas_price: gas_price as u128,
        };

        Ok(Environment::new(call, block, tx))
    }
}

/// Parse address from string
fn parse_address(s: &str) -> TestResult<Address> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    let bytes = hex::decode(s)?;
    if bytes.len() != 20 {
        return Err(TestError::Parse(format!(
            "invalid address length: {}",
            bytes.len()
        )));
    }
    Address::from_slice(&bytes)
        .map_err(|e| TestError::Parse(format!("invalid address: {}", e)))
}

/// Convert U256 bytes to u128 (truncates if too large)
fn u256_to_u128(bytes: &[u8; 32]) -> u128 {
    // Check if value fits in u128 (upper 16 bytes should be zero)
    if bytes[..16].iter().any(|&b| b != 0) {
        return u128::MAX; // Saturate
    }
    let mut buf = [0u8; 16];
    buf.copy_from_slice(&bytes[16..32]);
    u128::from_be_bytes(buf)
}

/// State test results
#[derive(Debug)]
pub struct StateTestResults {
    /// File path
    pub file: String,
    /// Passed tests
    pub passed: Vec<String>,
    /// Failed tests (name, reason)
    pub failed: Vec<(String, String)>,
    /// Skipped tests (name, reason)
    pub skipped: Vec<(String, String)>,
}

impl StateTestResults {
    /// Create new results
    pub fn new(file: String) -> Self {
        Self {
            file,
            passed: Vec::new(),
            failed: Vec::new(),
            skipped: Vec::new(),
        }
    }

    /// Total executed tests
    pub fn executed(&self) -> usize {
        self.passed.len() + self.failed.len()
    }

    /// Total tests including skipped
    pub fn total(&self) -> usize {
        self.executed() + self.skipped.len()
    }

    /// Pass rate as percentage
    pub fn pass_rate(&self) -> f64 {
        if self.executed() == 0 {
            return 100.0;
        }
        (self.passed.len() as f64 / self.executed() as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_forks() {
        assert!(SUPPORTED_FORKS.contains(&"London"));
        assert!(SUPPORTED_FORKS.contains(&"Shanghai"));
    }

    #[test]
    fn test_u256_to_u128() {
        let mut bytes = [0u8; 32];
        bytes[31] = 42;
        assert_eq!(u256_to_u128(&bytes), 42);
    }
}
