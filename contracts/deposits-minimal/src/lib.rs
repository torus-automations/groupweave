use near_sdk::{env, near_bindgen, AccountId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct MinimalContract {
    owner: AccountId,
    counter: u64,
}

impl Default for MinimalContract {
    fn default() -> Self {
        Self {
            owner: "near".parse().unwrap(),
            counter: 0,
        }
    }
}

#[near_bindgen]
impl MinimalContract {
    #[init]
    pub fn new(owner: AccountId) -> Self {
        Self { owner, counter: 0 }
    }

    pub fn get_owner(&self) -> AccountId {
        self.owner.clone()
    }

    pub fn increment(&mut self) {
        self.counter += 1;
    }

    pub fn get_counter(&self) -> u64 {
        self.counter
    }
}
