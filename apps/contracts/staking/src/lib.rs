use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, Balance, PanicOnDefault, Promise};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct StakeInfo {
    pub amount: Balance,
    pub staked_at: u64,
    pub last_reward_claim: u64,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct StakingContract {
    stakes: LookupMap<AccountId, StakeInfo>,
    total_staked: Balance,
    reward_rate: u128, // Rewards per second per NEAR staked
    min_stake_amount: Balance,
    owner: AccountId,
}

#[near_bindgen]
impl StakingContract {
    #[init]
    pub fn new(reward_rate: u128, min_stake_amount: Balance) -> Self {
        Self {
            stakes: LookupMap::new(b"s"),
            total_staked: 0,
            reward_rate,
            min_stake_amount,
            owner: env::predecessor_account_id(),
        }
    }

    #[payable]
    pub fn stake(&mut self) {
        let staker = env::predecessor_account_id();
        let amount = env::attached_deposit();
        
        assert!(amount >= self.min_stake_amount, "Stake amount too low");
        
        let current_time = env::block_timestamp();
        
        if let Some(mut stake_info) = self.stakes.get(&staker) {
            // Claim pending rewards before updating stake
            self.internal_claim_rewards(&staker, &mut stake_info);
            
            // Add to existing stake
            stake_info.amount += amount;
            stake_info.last_reward_claim = current_time;
            self.stakes.insert(&staker, &stake_info);
        } else {
            // Create new stake
            let stake_info = StakeInfo {
                amount,
                staked_at: current_time,
                last_reward_claim: current_time,
            };
            self.stakes.insert(&staker, &stake_info);
        }
        
        self.total_staked += amount;
    }

    pub fn unstake(&mut self, amount: Balance) {
        let staker = env::predecessor_account_id();
        let mut stake_info = self.stakes.get(&staker).expect("No stake found");
        
        assert!(stake_info.amount >= amount, "Insufficient staked amount");
        
        // Claim pending rewards
        self.internal_claim_rewards(&staker, &mut stake_info);
        
        // Update stake
        stake_info.amount -= amount;
        self.total_staked -= amount;
        
        if stake_info.amount == 0 {
            self.stakes.remove(&staker);
        } else {
            self.stakes.insert(&staker, &stake_info);
        }
        
        // Transfer unstaked amount back to user
        Promise::new(staker).transfer(amount);
    }

    pub fn claim_rewards(&mut self) {
        let staker = env::predecessor_account_id();
        let mut stake_info = self.stakes.get(&staker).expect("No stake found");
        
        self.internal_claim_rewards(&staker, &mut stake_info);
        self.stakes.insert(&staker, &stake_info);
    }

    fn internal_claim_rewards(&self, staker: &AccountId, stake_info: &mut StakeInfo) {
        let current_time = env::block_timestamp();
        let time_diff = current_time - stake_info.last_reward_claim;
        let time_diff_seconds = time_diff / 1_000_000_000;
        
        let rewards = (stake_info.amount as u128 * self.reward_rate * time_diff_seconds as u128) / 1_000_000_000_000_000_000_000_000;
        
        if rewards > 0 {
            stake_info.last_reward_claim = current_time;
            Promise::new(staker.clone()).transfer(rewards as Balance);
        }
    }

    pub fn get_stake_info(&self, account: AccountId) -> Option<StakeInfo> {
        self.stakes.get(&account)
    }

    pub fn calculate_pending_rewards(&self, account: AccountId) -> Balance {
        if let Some(stake_info) = self.stakes.get(&account) {
            let current_time = env::block_timestamp();
            let time_diff = current_time - stake_info.last_reward_claim;
            let time_diff_seconds = time_diff / 1_000_000_000;
            
            ((stake_info.amount as u128 * self.reward_rate * time_diff_seconds as u128) / 1_000_000_000_000_000_000_000_000) as Balance
        } else {
            0
        }
    }

    pub fn get_total_staked(&self) -> Balance {
        self.total_staked
    }

    pub fn get_reward_rate(&self) -> u128 {
        self.reward_rate
    }

    // Owner functions
    pub fn update_reward_rate(&mut self, new_rate: u128) {
        assert_eq!(env::predecessor_account_id(), self.owner, "Only owner can update reward rate");
        self.reward_rate = new_rate;
    }
}