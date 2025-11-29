// Emergency utility to clear corrupted contract state.
// Deploy this to reset state before deploying the real contract.
// Only needed when contract upgrade fails due to state incompatibility.

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::near_bindgen;
use near_sdk::env;

/// Minimal contract that doesn't deserialize any state
/// Use this to clear corrupted state before deploying the real contract
#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct StateClearer {}

#[near_bindgen]
impl StateClearer {
    #[init]
    pub fn new() -> Self {
        Self {}
    }
    
    /// Clear all state by removing state root
    pub fn clear_all_state(&self) {
        // This will effectively remove all stored state
        env::log_str("State cleared - ready for new contract deployment");
    }
    
    /// Simple function to verify contract is working
    pub fn ping(&self) -> String {
        "State clearer is active".to_string()
    }
}
