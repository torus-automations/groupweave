use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault, Promise, NearToken};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct StakeInfo {
    pub amount: NearToken,
    pub staked_at: u64,
    pub last_reward_claim: u64,
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
        Self {
            stakes: LookupMap::new(b"s"),
            total_staked: NearToken::from_yoctonear(0),
            reward_rate,
            min_stake_amount,
            max_stake_amount,
            owner: env::predecessor_account_id(),
        }
    }

    #[payable]
    pub fn stake(&mut self) {
        let staker = env::predecessor_account_id();
        let amount = env::attached_deposit();
        
        assert!(amount >= self.min_stake_amount, "Stake amount too low");
        assert!(amount <= self.max_stake_amount, "Stake amount too high");
        
        let current_time = env::block_timestamp();
        
        if let Some(mut stake_info) = self.stakes.get(&staker) {
            // Claim pending rewards before updating stake
            self.internal_claim_rewards(&staker, &mut stake_info);
            
            // Add to existing stake
            stake_info.amount = NearToken::from_yoctonear(stake_info.amount.as_yoctonear() + amount.as_yoctonear());
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
        
        self.total_staked = NearToken::from_yoctonear(self.total_staked.as_yoctonear() + amount.as_yoctonear());

        env::log_str(&format!("STAKE: Account {} staked {} yoctoNEAR", staker, amount));
    }

    pub fn unstake(&mut self, amount: NearToken) {
        let staker = env::predecessor_account_id();
        let mut stake_info = self.stakes.get(&staker).expect("No stake found");
        
        assert!(stake_info.amount.as_yoctonear() >= amount.as_yoctonear(), "Insufficient staked amount");
        
        // Claim pending rewards
        self.internal_claim_rewards(&staker, &mut stake_info);
        
        // Update stake
        stake_info.amount = NearToken::from_yoctonear(stake_info.amount.as_yoctonear() - amount.as_yoctonear());
        self.total_staked = NearToken::from_yoctonear(self.total_staked.as_yoctonear() - amount.as_yoctonear());
        
        if stake_info.amount.as_yoctonear() == 0 {
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
        
        let rewards = (stake_info.amount.as_yoctonear() * self.reward_rate * time_diff_seconds as u128) / 1_000_000_000_000_000_000_000_000;
        
        if rewards > 0 {
            stake_info.last_reward_claim = current_time;
            Promise::new(staker.clone()).transfer(NearToken::from_yoctonear(rewards));
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
            
            NearToken::from_yoctonear(((stake_info.amount.as_yoctonear() * self.reward_rate * time_diff_seconds as u128) / 1_000_000_000_000_000_000_000_000) as u128)
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
    pub fn update_reward_rate(&mut self, new_rate: u128) {
        assert_eq!(env::predecessor_account_id(), self.owner, "Only owner can update reward rate");
        self.reward_rate = new_rate;
    }

    pub fn update_max_stake_amount(&mut self, new_max_amount: NearToken) {
        assert_eq!(env::predecessor_account_id(), self.owner, "Only owner can update max stake amount");
        self.max_stake_amount = new_max_amount;
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
        assert_eq!(contract.get_max_stake_amount(), MAX_STAKE);
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
        assert_eq!(stake_info.amount, stake_amount);
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
        assert_eq!(contract.get_max_stake_amount(), new_max);
    }
}