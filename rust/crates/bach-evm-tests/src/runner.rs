//! Test runner and statistics

use crate::error::TestResult;
use crate::state_test::{StateTestResults, StateTestRunner};
use crate::vm_test::{VmTestResults, VmTestRunner};
use std::path::Path;
use std::time::{Duration, Instant};

/// Aggregated test statistics
#[derive(Debug, Default)]
pub struct TestStats {
    /// Total tests executed
    pub total: usize,
    /// Tests passed
    pub passed: usize,
    /// Tests failed
    pub failed: usize,
    /// Tests skipped
    pub skipped: usize,
    /// Total execution time
    pub duration: Duration,
    /// Failed test names with reasons
    pub failures: Vec<(String, String)>,
}

impl TestStats {
    /// Create empty stats
    pub fn new() -> Self {
        Self::default()
    }

    /// Add VM test results
    pub fn add_vm_results(&mut self, results: &VmTestResults) {
        self.total += results.total();
        self.passed += results.passed.len();
        self.failed += results.failed.len();
        for (name, reason) in &results.failed {
            self.failures.push((name.clone(), reason.clone()));
        }
    }

    /// Add state test results
    pub fn add_state_results(&mut self, results: &StateTestResults) {
        self.total += results.total();
        self.passed += results.passed.len();
        self.failed += results.failed.len();
        self.skipped += results.skipped.len();
        for (name, reason) in &results.failed {
            self.failures.push((name.clone(), reason.clone()));
        }
    }

    /// Pass rate as percentage
    pub fn pass_rate(&self) -> f64 {
        let executed = self.passed + self.failed;
        if executed == 0 {
            return 100.0;
        }
        (self.passed as f64 / executed as f64) * 100.0
    }

    /// Print summary
    pub fn print_summary(&self) {
        println!("\n========================================");
        println!("Test Summary");
        println!("========================================");
        println!("Total:   {}", self.total);
        println!("Passed:  {}", self.passed);
        println!("Failed:  {}", self.failed);
        println!("Skipped: {}", self.skipped);
        println!("Pass Rate: {:.2}%", self.pass_rate());
        println!("Duration: {:.2}s", self.duration.as_secs_f64());

        if !self.failures.is_empty() {
            println!("\nFailed tests:");
            for (name, reason) in &self.failures {
                println!("  - {}: {}", name, reason);
            }
        }
    }
}

/// Main test runner
pub struct TestRunner {
    /// VM test runner
    vm_runner: VmTestRunner,
    /// State test runner
    state_runner: StateTestRunner,
    /// Verbose output
    verbose: bool,
}

impl TestRunner {
    /// Create new test runner
    pub fn new(fork: &str, verbose: bool) -> Self {
        Self {
            vm_runner: VmTestRunner::new(verbose),
            state_runner: StateTestRunner::new(fork, verbose),
            verbose,
        }
    }

    /// Run VM tests from directory
    pub fn run_vm_tests(&self, dir: &Path) -> TestResult<TestStats> {
        let mut stats = TestStats::new();
        let start = Instant::now();

        if self.verbose {
            println!("Running VM tests from: {:?}", dir);
        }

        self.run_vm_tests_recursive(dir, &mut stats)?;

        stats.duration = start.elapsed();
        Ok(stats)
    }

    /// Run VM tests recursively
    fn run_vm_tests_recursive(&self, dir: &Path, stats: &mut TestStats) -> TestResult<()> {
        if !dir.exists() {
            if self.verbose {
                println!("Directory not found: {:?}", dir);
            }
            return Ok(());
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                self.run_vm_tests_recursive(&path, stats)?;
            } else if path.extension().is_some_and(|e| e == "json") {
                if let Ok(results) = self.vm_runner.run_file(&path) {
                    stats.add_vm_results(&results);
                    if self.verbose && !results.failed.is_empty() {
                        println!("File: {:?} - {} passed, {} failed",
                            path, results.passed.len(), results.failed.len());
                    }
                }
            }
        }

        Ok(())
    }

    /// Run state tests from directory
    pub fn run_state_tests(&self, dir: &Path) -> TestResult<TestStats> {
        let mut stats = TestStats::new();
        let start = Instant::now();

        if self.verbose {
            println!("Running state tests from: {:?}", dir);
        }

        self.run_state_tests_recursive(dir, &mut stats)?;

        stats.duration = start.elapsed();
        Ok(stats)
    }

    /// Run state tests recursively
    fn run_state_tests_recursive(&self, dir: &Path, stats: &mut TestStats) -> TestResult<()> {
        if !dir.exists() {
            if self.verbose {
                println!("Directory not found: {:?}", dir);
            }
            return Ok(());
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                self.run_state_tests_recursive(&path, stats)?;
            } else if path.extension().is_some_and(|e| e == "json") {
                if let Ok(results) = self.state_runner.run_file(&path) {
                    stats.add_state_results(&results);
                    if self.verbose && !results.failed.is_empty() {
                        println!("File: {:?} - {} passed, {} failed, {} skipped",
                            path, results.passed.len(), results.failed.len(), results.skipped.len());
                    }
                }
            }
        }

        Ok(())
    }

    /// Run all tests (VM + State)
    pub fn run_all(&self, tests_dir: &Path) -> TestResult<TestStats> {
        let mut combined = TestStats::new();
        let start = Instant::now();

        // Run VM tests
        let vm_dir = tests_dir.join("VMTests");
        if vm_dir.exists() {
            let vm_stats = self.run_vm_tests(&vm_dir)?;
            combined.passed += vm_stats.passed;
            combined.failed += vm_stats.failed;
            combined.total += vm_stats.total;
            combined.failures.extend(vm_stats.failures);
        }

        // Run state tests
        let state_dir = tests_dir.join("GeneralStateTests");
        if state_dir.exists() {
            let state_stats = self.run_state_tests(&state_dir)?;
            combined.passed += state_stats.passed;
            combined.failed += state_stats.failed;
            combined.skipped += state_stats.skipped;
            combined.total += state_stats.total;
            combined.failures.extend(state_stats.failures);
        }

        combined.duration = start.elapsed();
        Ok(combined)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_pass_rate() {
        let mut stats = TestStats::new();
        stats.passed = 90;
        stats.failed = 10;
        assert!((stats.pass_rate() - 90.0).abs() < 0.01);
    }

    #[test]
    fn test_stats_empty() {
        let stats = TestStats::new();
        assert_eq!(stats.pass_rate(), 100.0);
    }

    #[test]
    fn test_runner_creation() {
        let runner = TestRunner::new("London", false);
        assert!(!runner.verbose);
    }
}
