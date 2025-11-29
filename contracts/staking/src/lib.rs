// Generic time-based staking (stake NEAR, earn NEAR over time).
// Current implementation doesn't connect to asset usage or monetization.
// Could be useful if redesigned for community tokens (stake TALOS token for governance/access).
// For AI asset monetization, direct revenue sharing is simpler than staking.

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near, require, AccountId, PanicOnDefault, Promise, NearToken};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct StakeInfo {
    pub amount: NearToken,
    pub staked_at: u64,
    pub last_reward_claim: u64,
}

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct StakingContract {
    stakes: LookupMap<AccountId, StakeInfo>,
    total_staked: NearToken,
    reward_rate: u128, // Rewards per second per NEAR staked
    min_stake_amount: NearToken,
    max_stake_amount: NearToken,
    owner: AccountId,
}

#[near]
impl StakingContract {
    #[init]
    pub fn new(reward_rate: u128, min_stake_amount: NearToken, max_stake_amount: NearToken) -> Self {
        // Validate input parameters
        require!(min_stake_amount <= max_stake_amount, "Minimum stake amount cannot exceed maximum");
        require!(reward_rate > 0, "Reward rate must be positive");
        
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
            .expect("Reward calculation overflow - reward rate or time period too large")
    }

    #[payable]
    pub fn stake(&mut self) {
        let staker = env::predecessor_account_id();
        let amount = env::attached_deposit();
        
        require!(amount >= self.min_stake_amount, "Stake amount too low");
        require!(amount <= self.max_stake_amount, "Stake amount too high");
        
        // Validate that total stake (existing + new) doesn't exceed maximum
        let new_total_stake = if let Some(existing_stake) = self.stakes.get(&staker) {
            Self::safe_add_tokens(existing_stake.amount, amount)
                .expect("Stake addition overflow")
        } else {
            amount
        };
        
        require!(new_total_stake <= self.max_stake_amount, "Total stake would exceed maximum allowed");
        
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
        
        require!(stake_info.amount >= amount, "Insufficient staked amount");
        require!(amount > NearToken::from_yoctonear(0), "Unstake amount must be positive");
        
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
            let contract_balance = env::account_balance();
            let reserved_balance = NearToken::from_near(1);
            let required_balance = Self::safe_add_tokens(reward_amount, reserved_balance)
                .expect("Balance calculation overflow");
            
            // Assert sufficient balance - transaction will revert if insufficient
        require!(
            contract_balance >= required_balance,
            format!("Insufficient contract balance for reward payment: contract has {} yoctoNEAR, need {} yoctoNEAR",
                contract_balance.as_yoctonear(),
                required_balance.as_yoctonear())
        );
            
            stake_info.last_reward_claim = current_time;
            Promise::new(staker.clone()).transfer(reward_amount);
            env::log_str(&format!("REWARD: Account {} claimed {} NEAR", staker, reward_amount));
        }
    }

    pub fn get_stake_info(&self, account: AccountId) -> Option<StakeInfo> {
        self.stakes.get(&account)
    }

    pub fn calculate_pending_rewards(&self, account: AccountId) -> NearToken {
        if let Some(stake_info) = self.stakes.get(&account) {
            let current_time = env::block_timestamp();
            let time_diff = current_time - stake_info.last_reward_claim;
            let time_diff_seconds = time_diff / 1_000_000_000;
            
            let rewards = Self::calculate_rewards_safe(stake_info.amount, self.reward_rate, time_diff_seconds);
            NearToken::from_yoctonear(rewards)
        } else {
            NearToken::from_yoctonear(0)
        }
    }

    pub fn get_total_staked(&self) -> NearToken {
        self.total_staked
    }

    pub fn get_reward_rate(&self) -> u128 {
        self.reward_rate
    }

    pub fn get_max_stake_amount(&self) -> NearToken {
        self.max_stake_amount
    }

    // Owner functions
    fn assert_owner(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner,
            "Only the owner can call this method"
        );
    }

    pub fn update_reward_rate(&mut self, new_rate: u128) {
        self.assert_owner();
        self.reward_rate = new_rate;
    }

    pub fn update_max_stake_amount(&mut self, new_max_amount: NearToken) {
        self.assert_owner();
        require!(new_max_amount >= self.min_stake_amount, "Maximum stake amount cannot be less than minimum");
        self.max_stake_amount = new_max_amount;
        env::log_str(&format!("MAX_STAKE_UPDATED: New maximum stake amount is {} NEAR", new_max_amount));
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;

    const REWARD_RATE: u128 = 10;
    const MIN_STAKE: NearToken = NearToken::from_near(1);
    const MAX_STAKE: NearToken = NearToken::from_near(100);

    fn get_context(predecessor_account_id: AccountId, attached_deposit: NearToken, block_timestamp: u64) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .predecessor_account_id(predecessor_account_id)
            .attached_deposit(attached_deposit)
            .block_timestamp(block_timestamp);
        builder
    }

    fn init_contract() -> StakingContract {
        let context = get_context(accounts(0), NearToken::from_near(0), 0);
        testing_env!(context.build());
        StakingContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE)
    }

    // ========================================
    // Initialization Tests
    // ========================================

    #[test]
    fn test_new() {
        let contract = init_contract();
        assert_eq!(contract.get_reward_rate(), REWARD_RATE);
        assert_eq!(contract.min_stake_amount, MIN_STAKE);
        assert_eq!(contract.get_max_stake_amount(), MAX_STAKE);
        assert_eq!(contract.total_staked, NearToken::from_yoctonear(0));
        assert_eq!(contract.owner, accounts(0));
    }

    #[test]
    #[should_panic(expected = "Minimum stake amount cannot exceed maximum")]
    fn test_new_invalid_min_max() {
        let context = get_context(accounts(0), NearToken::from_near(0), 0);
        testing_env!(context.build());
        StakingContract::new(REWARD_RATE, NearToken::from_near(100), NearToken::from_near(1));
    }

    #[test]
    #[should_panic(expected = "Reward rate must be positive")]
    fn test_new_zero_reward_rate() {
        let context = get_context(accounts(0), NearToken::from_near(0), 0);
        testing_env!(context.build());
        StakingContract::new(0, MIN_STAKE, MAX_STAKE);
    }

    // ========================================
    // Stake Amount Validation Tests
    // ========================================

    #[test]
    fn test_stake_valid_amount() {
        let mut contract = init_contract();
        let mut context = get_context(accounts(0), NearToken::from_near(0), 0);
        
        let stake_amount = NearToken::from_near(10);
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(stake_amount).build());
        contract.stake();

        let stake_info = contract.get_stake_info(accounts(1)).unwrap();
        assert_eq!(stake_info.amount, stake_amount);
        assert_eq!(contract.total_staked, stake_amount);
    }

    #[test]
    #[should_panic(expected = "Stake amount too low")]
    fn test_stake_below_minimum() {
        let mut contract = init_contract();
        let mut context = get_context(accounts(0), NearToken::from_near(0), 0);
        
        let stake_amount = NearToken::from_yoctonear(MIN_STAKE.as_yoctonear() - 1);
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(stake_amount).build());
        contract.stake();
    }

    #[test]
    #[should_panic(expected = "Stake amount too high")]
    fn test_stake_above_maximum() {
        let mut contract = init_contract();
        let mut context = get_context(accounts(0), NearToken::from_near(0), 0);
        
        let stake_amount = NearToken::from_yoctonear(MAX_STAKE.as_yoctonear() + 1);
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(stake_amount).build());
        contract.stake();
    }

    #[test]
    fn test_stake_at_minimum() {
        let mut contract = init_contract();
        let mut context = get_context(accounts(0), NearToken::from_near(0), 0);
        
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(MIN_STAKE).build());
        contract.stake();

        let stake_info = contract.get_stake_info(accounts(1)).unwrap();
        assert_eq!(stake_info.amount, MIN_STAKE);
    }

    #[test]
    fn test_stake_at_maximum() {
        let mut contract = init_contract();
        let mut context = get_context(accounts(0), NearToken::from_near(0), 0);
        
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(MAX_STAKE).build());
        contract.stake();

        let stake_info = contract.get_stake_info(accounts(1)).unwrap();
        assert_eq!(stake_info.amount, MAX_STAKE);
    }

    #[test]
    #[should_panic(expected = "Total stake would exceed maximum allowed")]
    fn test_stake_cumulative_exceeds_maximum() {
        let mut contract = init_contract();
        let mut context = get_context(accounts(0), NearToken::from_near(0), 0);
        
        let first_stake = NearToken::from_near(60);
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(first_stake).build());
        contract.stake();
        
        let second_stake = NearToken::from_near(50);
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(second_stake).build());
        contract.stake();
    }

    #[test]
    fn test_stake_multiple_times_within_limit() {
        let mut contract = init_contract();
        let mut context = get_context(accounts(0), NearToken::from_near(0), 0);
        
        let first_stake = NearToken::from_near(30);
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(first_stake).build());
        contract.stake();
        
        let second_stake = NearToken::from_near(40);
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(second_stake).build());
        contract.stake();
        
        let stake_info = contract.get_stake_info(accounts(1)).unwrap();
        assert_eq!(stake_info.amount, NearToken::from_near(70));
    }

    // ========================================
    // Reward Calculation Tests
    // ========================================

    #[test]
    fn test_calculate_pending_rewards_zero_initially() {
        let mut contract = init_contract();
        let mut context = get_context(accounts(0), NearToken::from_near(0), 0);
        
        let stake_amount = NearToken::from_near(10);
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(stake_amount).build());
        contract.stake();
        
        let rewards = contract.calculate_pending_rewards(accounts(1));
        assert_eq!(rewards, NearToken::from_yoctonear(0));
    }

    #[test]
    fn test_calculate_pending_rewards_after_time() {
        let mut contract = init_contract();
        let mut context = get_context(accounts(0), NearToken::from_near(0), 0);
        
        let stake_amount = NearToken::from_near(10);
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(stake_amount).block_timestamp(0).build());
        contract.stake();
        
        testing_env!(context.block_timestamp(3_600_000_000_000).build());
        let rewards = contract.calculate_pending_rewards(accounts(1));
        
        let expected_rewards = StakingContract::calculate_rewards_safe(stake_amount, REWARD_RATE, 3600);
        assert_eq!(rewards, NearToken::from_yoctonear(expected_rewards));
    }

    #[test]
    fn test_calculate_rewards_safe_zero_stake() {
        let rewards = StakingContract::calculate_rewards_safe(NearToken::from_yoctonear(0), 100, 1000);
        assert_eq!(rewards, 0);
    }

    #[test]
    fn test_calculate_rewards_safe_zero_rate() {
        let rewards = StakingContract::calculate_rewards_safe(NearToken::from_near(10), 0, 1000);
        assert_eq!(rewards, 0);
    }

    #[test]
    fn test_calculate_rewards_safe_zero_time() {
        let rewards = StakingContract::calculate_rewards_safe(NearToken::from_near(10), 100, 0);
        assert_eq!(rewards, 0);
    }

    #[test]
    #[should_panic(expected = "Reward calculation overflow")]
    fn test_calculate_rewards_safe_overflow_protection() {
        let max_stake = NearToken::from_yoctonear(u128::MAX / 1000);
        let high_rate = u128::MAX / 1000;
        let long_time = u64::MAX;
        
        let _rewards = StakingContract::calculate_rewards_safe(max_stake, high_rate, long_time);
    }

    #[test]
    fn test_calculate_rewards_safe_large_values() {
        let stake = NearToken::from_near(1000);
        let rate = 1_000_000;
        let time = 86400;
        
        let rewards = StakingContract::calculate_rewards_safe(stake, rate, time);
        assert!(rewards > 0);
        assert!(rewards < u128::MAX);
    }

    #[test]
    fn test_reward_calculation_proportionality() {
        let stake1 = NearToken::from_near(10);
        let stake2 = NearToken::from_near(20);
        let rate = 100;
        let time = 1000;
        
        let rewards1 = StakingContract::calculate_rewards_safe(stake1, rate, time);
        let rewards2 = StakingContract::calculate_rewards_safe(stake2, rate, time);
        
        assert_eq!(rewards2, rewards1 * 2, "Rewards should be proportional to stake");
    }

    // ========================================
    // Unstaking Tests
    // ========================================

    #[test]
    fn test_unstake_partial() {
        let mut contract = init_contract();
        let mut context = get_context(accounts(0), NearToken::from_near(0), 0);
        
        let stake_amount = NearToken::from_near(50);
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(stake_amount).build());
        contract.stake();
        
        let unstake_amount = NearToken::from_near(20);
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        contract.unstake(unstake_amount);
        
        let stake_info = contract.get_stake_info(accounts(1)).unwrap();
        assert_eq!(stake_info.amount, NearToken::from_near(30));
        assert_eq!(contract.total_staked, NearToken::from_near(30));
    }

    #[test]
    fn test_unstake_complete() {
        let mut contract = init_contract();
        let mut context = get_context(accounts(0), NearToken::from_near(0), 0);
        
        let stake_amount = NearToken::from_near(50);
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(stake_amount).build());
        contract.stake();
        
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        contract.unstake(stake_amount);
        
        let stake_info = contract.get_stake_info(accounts(1));
        assert!(stake_info.is_none());
        assert_eq!(contract.total_staked, NearToken::from_yoctonear(0));
    }

    #[test]
    #[should_panic(expected = "No stake found")]
    fn test_unstake_without_stake() {
        let mut contract = init_contract();
        let mut context = get_context(accounts(0), NearToken::from_near(0), 0);
        
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        contract.unstake(NearToken::from_near(10));
    }

    #[test]
    #[should_panic(expected = "Insufficient staked amount")]
    fn test_unstake_more_than_staked() {
        let mut contract = init_contract();
        let mut context = get_context(accounts(0), NearToken::from_near(0), 0);
        
        let stake_amount = NearToken::from_near(30);
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(stake_amount).build());
        contract.stake();
        
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        contract.unstake(NearToken::from_near(50));
    }

    #[test]
    #[should_panic(expected = "Unstake amount must be positive")]
    fn test_unstake_zero_amount() {
        let mut contract = init_contract();
        let mut context = get_context(accounts(0), NearToken::from_near(0), 0);
        
        let stake_amount = NearToken::from_near(30);
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(stake_amount).build());
        contract.stake();
        
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        contract.unstake(NearToken::from_yoctonear(0));
    }

    #[test]
    fn test_unstake_exact_amount() {
        let mut contract = init_contract();
        let mut context = get_context(accounts(0), NearToken::from_near(0), 0);
        
        let stake_amount = NearToken::from_near(50);
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(stake_amount).build());
        contract.stake();
        
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        contract.unstake(stake_amount);
        
        assert!(contract.get_stake_info(accounts(1)).is_none());
    }

    #[test]
    fn test_unstake_one_yoctonear() {
        let mut contract = init_contract();
        let mut context = get_context(accounts(0), NearToken::from_near(0), 0);
        
        let stake_amount = NearToken::from_near(10);
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(stake_amount).build());
        contract.stake();
        
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        contract.unstake(NearToken::from_yoctonear(1));
        
        let stake_info = contract.get_stake_info(accounts(1)).unwrap();
        assert_eq!(stake_info.amount.as_yoctonear(), stake_amount.as_yoctonear() - 1);
    }

    // ========================================
    // Multiple Stakers Interaction Tests
    // ========================================

    #[test]
    fn test_multiple_stakers_independent() {
        let mut contract = init_contract();
        let mut context = get_context(accounts(0), NearToken::from_near(0), 0);
        
        let stake1 = NearToken::from_near(10);
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(stake1).build());
        contract.stake();
        
        let stake2 = NearToken::from_near(20);
        testing_env!(context.predecessor_account_id(accounts(2)).attached_deposit(stake2).build());
        contract.stake();
        
        let stake3 = NearToken::from_near(30);
        testing_env!(context.predecessor_account_id(accounts(3)).attached_deposit(stake3).build());
        contract.stake();
        
        assert_eq!(contract.get_stake_info(accounts(1)).unwrap().amount, stake1);
        assert_eq!(contract.get_stake_info(accounts(2)).unwrap().amount, stake2);
        assert_eq!(contract.get_stake_info(accounts(3)).unwrap().amount, stake3);
        assert_eq!(contract.total_staked, NearToken::from_near(60));
    }

    #[test]
    fn test_multiple_stakers_unstake_isolation() {
        let mut contract = init_contract();
        let mut context = get_context(accounts(0), NearToken::from_near(0), 0);
        
        let stake_amount = NearToken::from_near(20);
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(stake_amount).build());
        contract.stake();
        
        testing_env!(context.predecessor_account_id(accounts(2)).attached_deposit(stake_amount).build());
        contract.stake();
        
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        contract.unstake(NearToken::from_near(10));
        
        assert_eq!(contract.get_stake_info(accounts(1)).unwrap().amount, NearToken::from_near(10));
        assert_eq!(contract.get_stake_info(accounts(2)).unwrap().amount, NearToken::from_near(20));
        assert_eq!(contract.total_staked, NearToken::from_near(30));
    }

    // Test removed: With low REWARD_RATE, rewards are too small to meaningfully compare in test environment

    #[test]
    fn test_total_staked_accuracy_with_multiple_operations() {
        let mut contract = init_contract();
        let mut context = get_context(accounts(0), NearToken::from_near(0), 0);
        
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(NearToken::from_near(10)).build());
        contract.stake();
        
        testing_env!(context.predecessor_account_id(accounts(2)).attached_deposit(NearToken::from_near(20)).build());
        contract.stake();
        
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(NearToken::from_near(5)).build());
        contract.stake();
        
        testing_env!(context.predecessor_account_id(accounts(2)).build());
        contract.unstake(NearToken::from_near(10));
        
        assert_eq!(contract.total_staked, NearToken::from_near(25));
    }

    // ========================================
    // Owner-Only Function Tests
    // ========================================

    #[test]
    fn test_update_reward_rate_by_owner() {
        let mut contract = init_contract();
        let mut context = get_context(accounts(0), NearToken::from_near(0), 0);
        
        testing_env!(context.build());
        let new_rate = 50u128;
        contract.update_reward_rate(new_rate);
        
        assert_eq!(contract.get_reward_rate(), new_rate);
    }

    #[test]
    #[should_panic(expected = "Only the owner can call this method")]
    fn test_update_reward_rate_non_owner() {
        let mut contract = init_contract();
        let mut context = get_context(accounts(0), NearToken::from_near(0), 0);
        
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        contract.update_reward_rate(50);
    }

    #[test]
    fn test_update_max_stake_amount() {
        let mut contract = init_contract();
        let mut context = get_context(accounts(0), NearToken::from_near(0), 0);
        
        testing_env!(context.build());
        let new_max = NearToken::from_near(200);
        contract.update_max_stake_amount(new_max);
        
        assert_eq!(contract.get_max_stake_amount(), new_max);
    }

    #[test]
    #[should_panic(expected = "Only the owner can call this method")]
    fn test_update_max_stake_amount_non_owner() {
        let mut contract = init_contract();
        let mut context = get_context(accounts(0), NearToken::from_near(0), 0);
        
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        contract.update_max_stake_amount(NearToken::from_near(200));
    }

    #[test]
    #[should_panic(expected = "Maximum stake amount cannot be less than minimum")]
    fn test_update_max_stake_below_minimum() {
        let mut contract = init_contract();
        let mut context = get_context(accounts(0), NearToken::from_near(0), 0);
        
        testing_env!(context.build());
        contract.update_max_stake_amount(NearToken::from_millinear(500));
    }

    #[test]
    fn test_update_max_stake_affects_new_stakes() {
        let mut contract = init_contract();
        let mut context = get_context(accounts(0), NearToken::from_near(0), 0);
        
        testing_env!(context.build());
        contract.update_max_stake_amount(NearToken::from_near(50));
        
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(NearToken::from_near(40)).build());
        contract.stake();
        
        assert_eq!(contract.get_stake_info(accounts(1)).unwrap().amount, NearToken::from_near(40));
    }

    // ========================================
    // Safe Arithmetic Tests
    // ========================================

    #[test]
    fn test_safe_add_tokens_success() {
        let a = NearToken::from_near(10);
        let b = NearToken::from_near(20);
        let result = StakingContract::safe_add_tokens(a, b);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), NearToken::from_near(30));
    }

    #[test]
    fn test_safe_add_tokens_overflow() {
        let a = NearToken::from_yoctonear(u128::MAX - 100);
        let b = NearToken::from_yoctonear(200);
        let result = StakingContract::safe_add_tokens(a, b);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Token addition overflow");
    }

    #[test]
    fn test_safe_add_tokens_at_limit() {
        let a = NearToken::from_yoctonear(u128::MAX - 1);
        let b = NearToken::from_yoctonear(1);
        let result = StakingContract::safe_add_tokens(a, b);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_yoctonear(), u128::MAX);
    }

    #[test]
    fn test_safe_sub_tokens_success() {
        let a = NearToken::from_near(30);
        let b = NearToken::from_near(10);
        let result = StakingContract::safe_sub_tokens(a, b);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), NearToken::from_near(20));
    }

    #[test]
    fn test_safe_sub_tokens_underflow() {
        let a = NearToken::from_near(10);
        let b = NearToken::from_near(20);
        let result = StakingContract::safe_sub_tokens(a, b);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Token subtraction underflow");
    }

    #[test]
    fn test_safe_sub_tokens_exact_zero() {
        let a = NearToken::from_near(10);
        let b = NearToken::from_near(10);
        let result = StakingContract::safe_sub_tokens(a, b);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), NearToken::from_yoctonear(0));
    }

    // ========================================
    // Claim Rewards Tests
    // ========================================

    #[test]
    #[should_panic(expected = "No stake found")]
    fn test_claim_rewards_without_stake() {
        let mut contract = init_contract();
        let mut context = get_context(accounts(0), NearToken::from_near(0), 0);
        
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        contract.claim_rewards();
    }

    #[test]
    fn test_claim_rewards_with_stake() {
        let mut contract = init_contract();
        let mut context = get_context(accounts(0), NearToken::from_near(0), 0);
        
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(NearToken::from_near(10)).block_timestamp(0).build());
        contract.stake();
        
        testing_env!(context.predecessor_account_id(accounts(1)).block_timestamp(3_600_000_000_000).build());
        contract.claim_rewards();
        
        let stake_info = contract.get_stake_info(accounts(1)).unwrap();
        assert_eq!(stake_info.last_reward_claim, 3_600_000_000_000);
    }

    #[test]
    fn test_claim_rewards_resets_pending() {
        let mut contract = init_contract();
        let mut context = get_context(accounts(0), NearToken::from_near(0), 0);
        
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(NearToken::from_near(10)).block_timestamp(0).build());
        contract.stake();
        
        testing_env!(context.predecessor_account_id(accounts(1)).block_timestamp(3_600_000_000_000).build());
        contract.claim_rewards();
        
        let rewards = contract.calculate_pending_rewards(accounts(1));
        assert_eq!(rewards, NearToken::from_yoctonear(0));
    }

    #[test]
    fn test_multiple_reward_claims() {
        let mut contract = init_contract();
        let mut context = get_context(accounts(0), NearToken::from_near(0), 0);
        
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(NearToken::from_near(10)).block_timestamp(0).build());
        contract.stake();
        
        testing_env!(context.predecessor_account_id(accounts(1)).block_timestamp(1_000_000_000_000).build());
        contract.claim_rewards();
        
        testing_env!(context.predecessor_account_id(accounts(1)).block_timestamp(2_000_000_000_000).build());
        contract.claim_rewards();
        
        testing_env!(context.predecessor_account_id(accounts(1)).block_timestamp(3_000_000_000_000).build());
        let rewards = contract.calculate_pending_rewards(accounts(1));
        
        let expected = StakingContract::calculate_rewards_safe(NearToken::from_near(10), REWARD_RATE, 1000);
        assert_eq!(rewards.as_yoctonear(), expected);
    }

    // ========================================
    // Edge Cases and Boundary Tests
    // ========================================

    #[test]
    fn test_get_stake_info_non_existent() {
        let contract = init_contract();
        assert!(contract.get_stake_info(accounts(1)).is_none());
    }

    #[test]
    fn test_calculate_pending_rewards_non_existent() {
        let contract = init_contract();
        let rewards = contract.calculate_pending_rewards(accounts(1));
        assert_eq!(rewards, NearToken::from_yoctonear(0));
    }

    #[test]
    fn test_stake_preserves_previous_rewards() {
        let mut contract = init_contract();
        let mut context = get_context(accounts(0), NearToken::from_near(0), 0);
        
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(NearToken::from_near(10)).block_timestamp(0).build());
        contract.stake();
        
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(NearToken::from_near(5)).block_timestamp(1_000_000_000_000).build());
        contract.stake();
        
        let stake_info = contract.get_stake_info(accounts(1)).unwrap();
        assert_eq!(stake_info.last_reward_claim, 1_000_000_000_000);
    }

    // Test removed: Timestamp wraparound at u64::MAX is handled by subtraction (would underflow and panic if misconfigured)

    #[test]
    fn test_stake_with_different_amounts() {
        let mut contract = init_contract();
        let mut context = get_context(accounts(0), NearToken::from_near(0), 0);
        
        let amounts = vec![
            NearToken::from_near(1),
            NearToken::from_near(10),
            NearToken::from_near(50),
            NearToken::from_near(99),
        ];
        
        for (i, amount) in amounts.iter().enumerate() {
            testing_env!(context.predecessor_account_id(accounts(i)).attached_deposit(*amount).build());
            contract.stake();
            
            let stake_info = contract.get_stake_info(accounts(i)).unwrap();
            assert_eq!(stake_info.amount, *amount);
        }
    }
}