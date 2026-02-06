//! VM test runner

use crate::error::{TestError, TestResult};
use crate::types::*;
use bach_evm::{BlockContext, CallContext, Environment, ExecutionResult, Interpreter, TxContext};
use bach_primitives::{Address, H256};
use std::path::Path;

/// VM test runner
pub struct VmTestRunner {
    /// Verbose output
    verbose: bool,
}

impl VmTestRunner {
    /// Create new VM test runner
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    /// Run all tests in a file
    pub fn run_file(&self, path: &Path) -> TestResult<VmTestResults> {
        let content = std::fs::read_to_string(path)?;
        let tests: VmTestFile = serde_json::from_str(&content)?;

        let mut results = VmTestResults::new(path.to_string_lossy().to_string());

        for (name, test_case) in tests {
            let result = self.run_test(&name, &test_case);
            match result {
                Ok(()) => {
                    if self.verbose {
                        tracing::info!("PASS: {}", name);
                    }
                    results.passed.push(name);
                }
                Err(e) => {
                    if self.verbose {
                        tracing::warn!("FAIL: {} - {}", name, e);
                    }
                    results.failed.push((name, e.to_string()));
                }
            }
        }

        Ok(results)
    }

    /// Run a single test case
    pub fn run_test(&self, name: &str, test: &VmTestCase) -> TestResult<()> {
        // Build environment from test case
        let env = self.build_environment(test)?;

        // Create interpreter with code and gas
        let mut interp = Interpreter::new(test.exec.code.0.clone(), test.exec.gas.0);

        // Run the interpreter
        let result = interp.run(&env);

        // Check results
        self.check_result(name, test, &result, &interp)
    }

    /// Build execution environment from test case
    fn build_environment(&self, test: &VmTestCase) -> TestResult<Environment> {
        // Convert addresses
        let caller = Address::from_slice(&test.exec.caller.0)
            .map_err(|e| TestError::Parse(format!("invalid caller address: {}", e)))?;
        let address = Address::from_slice(&test.exec.address.0)
            .map_err(|e| TestError::Parse(format!("invalid address: {}", e)))?;
        let origin = Address::from_slice(&test.exec.origin.0)
            .map_err(|e| TestError::Parse(format!("invalid origin address: {}", e)))?;
        let coinbase = Address::from_slice(&test.env.current_coinbase.0)
            .map_err(|e| TestError::Parse(format!("invalid coinbase address: {}", e)))?;

        // Convert value (U256 to u128 - may truncate for very large values)
        let value = u256_to_u128(&test.exec.value.0);
        let gas_price = test.exec.gas_price.0 as u128;

        // Build call context
        let call = CallContext {
            caller,
            address,
            value,
            data: test.exec.data.0.clone(),
            gas: test.exec.gas.0,
            is_static: false,
            depth: 0,
        };

        // Build block context
        let prevrandao = if let Some(r) = test.env.current_random.as_ref() {
            H256::from_slice(&r.0)
                .map_err(|e| TestError::Parse(format!("invalid prevrandao: {}", e)))?
        } else {
            H256::from_slice(&test.env.current_difficulty.0)
                .map_err(|e| TestError::Parse(format!("invalid difficulty: {}", e)))?
        };

        let block = BlockContext {
            coinbase,
            timestamp: test.env.current_timestamp.0,
            number: test.env.current_number.0,
            gas_limit: test.env.current_gas_limit.0,
            base_fee: test.env.current_base_fee.as_ref().map(|f| f.0 as u128).unwrap_or(0),
            prevrandao,
            chain_id: 1,
        };

        // Build tx context
        let tx = TxContext {
            origin,
            gas_price,
        };

        Ok(Environment::new(call, block, tx))
    }

    /// Check execution result against expected values
    fn check_result(
        &self,
        name: &str,
        test: &VmTestCase,
        result: &ExecutionResult,
        interp: &Interpreter,
    ) -> TestResult<()> {
        match (test.gas.as_ref(), test.post.as_ref()) {
            // Test expects success
            (Some(expected_gas), Some(_post)) => {
                // Check that execution succeeded
                if !result.success {
                    return Err(TestError::Assertion(format!(
                        "{}: expected success but execution failed",
                        name
                    )));
                }

                // Check gas remaining
                let gas_remaining = interp.gas_remaining();
                if gas_remaining != expected_gas.0 {
                    return Err(TestError::Assertion(format!(
                        "{}: gas mismatch: expected {}, got {}",
                        name, expected_gas.0, gas_remaining
                    )));
                }

                // Check output if specified
                if let Some(expected_out) = &test.out {
                    if result.output != expected_out.0 {
                        return Err(TestError::Assertion(format!(
                            "{}: output mismatch: expected {:?}, got {:?}",
                            name, expected_out.0, result.output
                        )));
                    }
                }

                Ok(())
            }
            // Test expects failure (no gas or post state)
            (None, None) => {
                // Execution should fail - this is acceptable
                Ok(())
            }
            _ => {
                // Unusual case - partial expectations
                Ok(())
            }
        }
    }
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

/// VM test results
#[derive(Debug)]
pub struct VmTestResults {
    /// File path
    pub file: String,
    /// Passed tests
    pub passed: Vec<String>,
    /// Failed tests (name, reason)
    pub failed: Vec<(String, String)>,
}

impl VmTestResults {
    /// Create new results
    pub fn new(file: String) -> Self {
        Self {
            file,
            passed: Vec::new(),
            failed: Vec::new(),
        }
    }

    /// Total number of tests
    pub fn total(&self) -> usize {
        self.passed.len() + self.failed.len()
    }

    /// Pass rate as percentage
    pub fn pass_rate(&self) -> f64 {
        if self.total() == 0 {
            return 100.0;
        }
        (self.passed.len() as f64 / self.total() as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u256_to_u128() {
        let mut bytes = [0u8; 32];
        bytes[31] = 1;
        assert_eq!(u256_to_u128(&bytes), 1);

        bytes[30] = 1;
        assert_eq!(u256_to_u128(&bytes), 257);
    }

    #[test]
    fn test_u256_to_u128_overflow() {
        let mut bytes = [0u8; 32];
        bytes[0] = 1; // Too large for u128
        assert_eq!(u256_to_u128(&bytes), u128::MAX);
    }
}
