//! Compile-fail tests runner using trybuild
//!
//! These tests verify that certain unsafe patterns don't compile.
//! Run with: cargo test --test ui_tests
//!
//! NOTE: Disabled until trybuild is added as dev-dependency

#[test]
#[ignore = "Enable when trybuild is added as dev-dependency"]
fn ui_tests() {
    // Disabled - would require: let t = trybuild::TestCases::new();
    // t.compile_fail("tests/ui/*.rs");
}
