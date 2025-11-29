use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, AccountId};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct SimpleContract {
    owner: AccountId,
}

impl Default for SimpleContract {
    fn default() -> Self {
        env::panic_str("Not initialized")
    }
}

#[near_bindgen]
impl SimpleContract {
    #[init]
    pub fn new(owner: AccountId) -> Self {
        Self { owner }
    }

    pub fn get_owner(&self) -> AccountId {
        self.owner.clone()
    }
}
