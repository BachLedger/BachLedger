//! Compile-fail test: Wallet should NOT implement Clone
//!
//! This test verifies that private keys cannot be accidentally duplicated
//! by cloning a Wallet. If this compiles, it's a security issue.

fn main() {
    // TODO: Uncomment when bach_sdk is available
    // use bach_sdk::Wallet;
    //
    // let wallet = Wallet::new_random();
    // let _cloned = wallet.clone(); // This should fail to compile!
}
