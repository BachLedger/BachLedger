//! Compile-fail test: Wallet should NOT implement Copy
//!
//! This test verifies that private keys cannot be implicitly copied.
//! If this compiles, it's a security issue.

fn main() {
    // TODO: Uncomment when bach_sdk is available
    // use bach_sdk::Wallet;
    //
    // let wallet = Wallet::new_random();
    // let _copied = wallet; // Move
    // let _use_original = wallet.address(); // This should fail - wallet was moved!
}
