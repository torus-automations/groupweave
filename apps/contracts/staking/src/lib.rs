use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault, Promise, NearToken};
use schemars::JsonSchema;

#[derive(BorshDeserialize, BorshSerialize, Clone)]
pub struct StakeInfo {
    pub amount: NearToken,
    pub staked_at: u64,
    pub last_reward_claim: u64,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct StakeInfoView {
    pub amount: String,
    pub staked_at: u64,
    pub last_reward_claim: u64,
}

impl From<StakeInfo> for StakeInfoView {
    fn from(stake_info: StakeInfo) -> Self {
        Self {
            amount: stake_info.amount.to_string(),
            staked_at: stake_info.staked_at,
            last_reward_claim: stake_info.last_reward_claim,
        }
    }
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct StakingContract {
    stakes: LookupMap<AccountId, StakeInfo>,
    total_staked: NearToken,
    reward_rate: u128, // Rewards per second per NEAR staked
    min_stake_amount: NearToken,
    max_stake_amount: NearToken,
    owner: AccountId,
}

#[near_bindgen]
impl StakingContract {
    #[init]
    pub fn new(reward_rate: u128, min_stake_amount: NearToken, max_stake_amount: NearToken) -> Self {
        // Validate input parameters
        assert!(min_stake_amount <= max_stake_amount, "Minimum stake amount cannot exceed maximum");
        assert!(reward_rate > 0, "Reward rate must be positive");
        
        Self {
            stakes: LookupMap::new(b"s"),
            total_staked: NearToken::from_yoctonear(0),
            reward_rate,
            min_stake_amount,
            max_stake_amount,
            owner: env::predecessor_account_id(),
        }
    }

    // Helper function for safe token addition
    fn safe_add_tokens(a: NearToken, b: NearToken) -> Result<NearToken, &'static str> {
        a.as_yoctonear().checked_add(b.as_yoctonear())
            .map(NearToken::from_yoctonear)
            .ok_or("Token addition overflow")
    }

    // Helper function for safe token subtraction
    fn safe_sub_tokens(a: NearToken, b: NearToken) -> Result<NearToken, &'static str> {
        a.as_yoctonear().checked_sub(b.as_yoctonear())
            .map(NearToken::from_yoctonear)
            .ok_or("Token subtraction underflow")
    }

    // Helper function for safe reward calculation
    fn calculate_rewards_safe(stake_amount: NearToken, reward_rate: u128, time_seconds: u64) -> u128 {
        // Use checked arithmetic to prevent overflow
        // Divide by the scaling factor last to maintain precision
        stake_amount.as_yoctonear()
            .checked_mul(reward_rate)
            .and_then(|x| x.checked_mul(time_seconds as u128))
            .and_then(|x| x.checked_div(1_000_000_000_000_000_000_000_000))
            .unwrap_or(0) // Return 0 on overflow rather than panicking
    }

    #[payable]
    pub fn stake(&mut self) {
        let staker = env::predecessor_account_id();
        let amount = env::attached_deposit();
        
        assert!(amount >= self.min_stake_amount, "Stake amount too low");
        assert!(amount <= self.max_stake_amount, "Stake amount too high");
        
        // Validate that total stake (existing + new) doesn't exceed maximum
        let new_total_stake = if let Some(existing_stake) = self.stakes.get(&staker) {
            Self::safe_add_tokens(existing_stake.amount, amount)
                .expect("Stake addition overflow")
        } else {
            amount
        };
        
        assert!(new_total_stake <= self.max_stake_amount, "Total stake would exceed maximum allowed");
        
        let current_time = env::block_timestamp();
        
        if let Some(mut stake_info) = self.stakes.get(&staker) {
            // Claim pending rewards before updating stake
            self.internal_claim_rewards(&staker, &mut stake_info);
            
            // Add to existing stake using safe addition
            stake_info.amount = Self::safe_add_tokens(stake_info.amount, amount)
                .expect("Stake addition overflow");
            stake_info.last_reward_claim = current_time;
            self.stakes.insert(&staker, &stake_info);
        } else {
            // Create new stake
            let stake_info = StakeInfo {
                amount: amount,
                staked_at: current_time,
                last_reward_claim: current_time,
            };
            self.stakes.insert(&staker, &stake_info);
        }
        
        // Update total staked using safe addition
        self.total_staked = Self::safe_add_tokens(self.total_staked, amount)
            .expect("Total stake addition overflow");

        env::log_str(&format!("STAKE: Account {} staked {} NEAR", staker, amount));
    }

    pub fn unstake(&mut self, amount: NearToken) {
        let staker = env::predecessor_account_id();
        let mut stake_info = self.stakes.get(&staker).expect("No stake found");
        
        assert!(stake_info.amount >= amount, "Insufficient staked amount");
        assert!(amount > NearToken::from_yoctonear(0), "Unstake amount must be positive");
        
        // Claim pending rewards
        self.internal_claim_rewards(&staker, &mut stake_info);
        
        // Update stake using safe subtraction
        stake_info.amount = Self::safe_sub_tokens(stake_info.amount, amount)
            .expect("Stake subtraction underflow");
        self.total_staked = Self::safe_sub_tokens(self.total_staked, amount)
            .expect("Total stake subtraction underflow");
        
        if stake_info.amount == NearToken::from_yoctonear(0) {
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
        
        let rewards = Self::calculate_rewards_safe(stake_info.amount, self.reward_rate, time_diff_seconds);
        
        if rewards > 0 {
            let reward_amount = NearToken::from_yoctonear(rewards);
            
            // Check if contract has sufficient balance to pay rewards
            // Reserve 1 NEAR for contract operations
            let contract_balance = env::account_balance();
            let reserved_balance = NearToken::from_near(1);
            
            if contract_balance > Self::safe_add_tokens(reward_amount, reserved_balance).unwrap_or(contract_balance) {
                stake_info.last_reward_claim = current_time;
                Promise::new(staker.clone()).transfer(reward_amount);
                env::log_str(&format!("REWARD: Account {} claimed {} NEAR", staker, reward_amount));
            } else {
                env::log_str(&format!("REWARD_FAILED: Insufficient contract balance for {}", staker));
            }
        }
    }

    pub fn get_stake_info(&self, account: AccountId) -> Option<StakeInfoView> {
        self.stakes.get(&account).map(|stake_info| stake_info.into())
    }

    pub fn calculate_pending_rewards(&self, account: AccountId) -> String {
        if let Some(stake_info) = self.stakes.get(&account) {
            let current_time = env::block_timestamp();
            let time_diff = current_time - stake_info.last_reward_claim;
            let time_diff_seconds = time_diff / 1_000_000_000;
            
            let rewards = Self::calculate_rewards_safe(stake_info.amount, self.reward_rate, time_diff_seconds);
            NearToken::from_yoctonear(rewards).to_string()
        } else {
            NearToken::from_yoctonear(0).to_string()
        }
    }

    pub fn get_total_staked(&self) -> String {
        self.total_staked.to_string()
    }

    pub fn get_reward_rate(&self) -> u128 {
        self.reward_rate
    }

    pub fn get_max_stake_amount(&self) -> String {
        self.max_stake_amount.to_string()
    }

    // Owner functions
    pub fn update_reward_rate(&mut self, new_rate: u128) {
        assert_eq!(env::predecessor_account_id(), self.owner, "Only owner can update reward rate");
        self.reward_rate = new_rate;
    }

    pub fn update_max_stake_amount(&mut self, new_max_amount: NearToken) {
        assert_eq!(env::predecessor_account_id(), self.owner, "Only owner can update max stake amount");
        assert!(new_max_amount >= self.min_stake_amount, "Maximum stake amount cannot be less than minimum");
        self.max_stake_amount = new_max_amount;
        env::log_str(&format!("MAX_STAKE_UPDATED: New maximum stake amount is {} NEAR", new_max_amount));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;
    use near_sdk::NearToken;

    const REWARD_RATE: u128 = 10;
    const MIN_STAKE: NearToken = NearToken::from_near(1);
    const MAX_STAKE: NearToken = NearToken::from_near(100);

    fn get_context(predecessor_account_id: AccountId, attached_deposit: NearToken) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .predecessor_account_id(predecessor_account_id)
            .attached_deposit(attached_deposit);
        builder
    }

    #[test]
    fn test_new() {
        let context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let contract = StakingContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);
        assert_eq!(contract.get_reward_rate(), REWARD_RATE);
        assert_eq!(contract.min_stake_amount, MIN_STAKE);
        assert_eq!(contract.get_max_stake_amount(), MAX_STAKE.to_string());
    }

    #[test]
    fn test_stake_valid_amount() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = StakingContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        let stake_amount = NearToken::from_near(10);
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(stake_amount).build());
        contract.stake();

        let stake_info = contract.get_stake_info(accounts(1)).unwrap();
        assert_eq!(stake_info.amount, stake_amount.to_string());
    }

    #[test]
    #[should_panic(expected = "Stake amount too low")]
    fn test_stake_below_minimum() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = StakingContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        let stake_amount = NearToken::from_yoctonear(MIN_STAKE.as_yoctonear() - 1);
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(stake_amount).build());
        contract.stake();
    }

    #[test]
    #[should_panic(expected = "Stake amount too high")]
    fn test_stake_above_maximum() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = StakingContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        let stake_amount = NearToken::from_yoctonear(MAX_STAKE.as_yoctonear() + 1);
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(stake_amount).build());
        contract.stake();
    }

    #[test]
    fn test_update_max_stake_amount() {
        let context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = StakingContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        let new_max = NearToken::from_near(200);
        contract.update_max_stake_amount(new_max);
        assert_eq!(contract.get_max_stake_amount(), new_max.to_string());
    }
}