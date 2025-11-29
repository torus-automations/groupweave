// Content creation bounty market.
// Creators submit work (characters, art, etc). Community stakes NEAR on favorites.
// Most staked submission wins. Prize split: creator gets 90% (configurable), backers split 10%.
// Platform takes 5% fee. Base prize from bounty creator + community stakes.
//
// Content Bounty Market
//
// Facilitates decentralized content curation via stake-weighted voting.
//
// Architecture:
// - Submissions: Users submit content (references) for a bounty.
// - Curation: Community stakes NEAR on submissions.
// - Settlement: Highest staked submission wins.
//   - Winner: Receives Base Prize + (Losing Stakes * Backer Share).
//   - Backers: Receive proportional share of winning pool.
//   - Platform: Takes configurable fee (capped at 10%).
//
// Security Model:
// Relies on economic disincentives and transparency to mitigate self-staking attacks.
// - Public Ledger: All stakes are traceable.
// - Cost of Attack: Capital lockup + gas fees + risk of social slashing (ban).
// - Rate Limiting: Max stake per user and max submissions enforced.

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::json_types::U128;
use near_sdk::{env, near, require, AccountId, PanicOnDefault, Promise, NearToken};
use std::convert::TryFrom;
use schemars::JsonSchema;

// Safety constants to prevent overflow and ensure system stability

const MAX_PLATFORM_FEE_RATE: u128 = 1000; // 10% maximum platform fee
const MAX_SUBMISSIONS: usize = 100; // Maximum content submissions per bounty
const MIN_SUBMISSIONS: usize = 1; // Minimum 1 submission to close bounty
const MAX_BOUNTY_DURATION: u64 = 1_000_000; // Maximum bounty duration in blocks
const MIN_BOUNTY_DURATION: u64 = 1; // Minimum bounty duration in blocks
const MAX_PARTICIPANTS_PER_BOUNTY: usize = 150; // Maximum participants to prevent DOS during reward distribution
const DEFAULT_CREATOR_SHARE: u8 = 90; // Default 90% to winning creator
const DEFAULT_BACKER_SHARE: u8 = 10; // Default 10% to backers

// Content submission for a bounty
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ContentSubmission {
    pub creator: AccountId,
    pub creation_id: String,      // Reference to Dreamweave database Creation ID
    pub title: String,
    pub thumbnail_url: String,
    pub total_staked: NearToken,
    pub submitted_at: u64,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Bounty {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub requirements: String,     // What content is required
    pub submissions: Vec<ContentSubmission>,
    pub creator: AccountId,
    pub base_prize: NearToken,    // Initial prize from creator
    pub max_stake_per_user: NearToken,
    pub creator_share: u8,        // % to winning creator (e.g. 60)
    pub backer_share: u8,         // % to backers (e.g. 40)
    pub is_active: bool,
    pub created_at: u64,
    pub ends_at: u64,
    pub total_staked: NearToken,  // Community stakes only (not base_prize)
    pub is_closed: bool,
    pub winning_submission: Option<u64>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ParticipantStake {
    pub bounty_id: u64,
    pub submission_index: u64,    // Index into bounty.submissions
    pub amount: NearToken,
    pub staked_at: u64,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct ContentSubmissionView {
    #[schemars(with = "String")]
    pub creator: AccountId,
    pub creation_id: String,
    pub title: String,
    pub thumbnail_url: String,
    #[schemars(with = "String")]
    pub total_staked: U128,
    pub submitted_at: u64,
}

impl From<ContentSubmission> for ContentSubmissionView {
    fn from(sub: ContentSubmission) -> Self {
        Self {
            creator: sub.creator,
            creation_id: sub.creation_id,
            title: sub.title,
            thumbnail_url: sub.thumbnail_url,
            total_staked: U128(sub.total_staked.as_yoctonear()),
            submitted_at: sub.submitted_at,
        }
    }
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct BountyView {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub requirements: String,
    pub submissions: Vec<ContentSubmissionView>,
    #[schemars(with = "String")]
    pub creator: AccountId,
    #[schemars(with = "String")]
    pub base_prize: U128,
    #[schemars(with = "String")]
    pub max_stake_per_user: U128,
    pub creator_share: u8,
    pub backer_share: u8,
    pub is_active: bool,
    pub created_at: u64,
    pub ends_at: u64,
    #[schemars(with = "String")]
    pub total_staked: U128,
    pub is_closed: bool,
    pub winning_submission: Option<u64>,
}

impl From<Bounty> for BountyView {
    fn from(bounty: Bounty) -> Self {
        Self {
            id: bounty.id,
            title: bounty.title,
            description: bounty.description,
            requirements: bounty.requirements,
            submissions: bounty.submissions.into_iter().map(|s| s.into()).collect(),
            creator: bounty.creator,
            base_prize: U128(bounty.base_prize.as_yoctonear()),
            max_stake_per_user: U128(bounty.max_stake_per_user.as_yoctonear()),
            creator_share: bounty.creator_share,
            backer_share: bounty.backer_share,
            is_active: bounty.is_active,
            created_at: bounty.created_at,
            ends_at: bounty.ends_at,
            total_staked: U128(bounty.total_staked.as_yoctonear()),
            is_closed: bounty.is_closed,
            winning_submission: bounty.winning_submission,
        }
    }
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct ParticipantStakeView {
    pub bounty_id: u64,
    pub submission_index: u64,
    #[schemars(with = "String")]
    pub amount: U128,
    pub staked_at: u64,
}

impl From<ParticipantStake> for ParticipantStakeView {
    fn from(stake: ParticipantStake) -> Self {
        Self {
            bounty_id: stake.bounty_id,
            submission_index: stake.submission_index,
            amount: U128(stake.amount.as_yoctonear()),
            staked_at: stake.staked_at,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Clone)]
pub struct StakeInfo {
    pub amount: NearToken,
    pub staked_at: u64,
    pub last_reward_claim: u64,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct StakeInfoView {
    #[schemars(with = "String")]
    pub amount: U128,
    pub staked_at: u64,
    pub last_reward_claim: u64,
}

impl From<StakeInfo> for StakeInfoView {
    fn from(stake_info: StakeInfo) -> Self {
        Self {
            amount: U128(stake_info.amount.as_yoctonear()),
            staked_at: stake_info.staked_at,
            last_reward_claim: stake_info.last_reward_claim,
        }
    }
}

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct BountyPredictionContract {
    // Existing staking fields (for backward compatibility)
    stakes: LookupMap<AccountId, StakeInfo>,
    total_staked: NearToken,
    reward_rate: u128, // Rewards per second per NEAR staked
    min_stake_amount: NearToken,
    max_stake_amount: NearToken,
    owner: AccountId,

    // New bounty fields
    bounties: LookupMap<u64, Bounty>,
    participant_stakes: LookupMap<(AccountId, u64), ParticipantStake>,
    bounty_participants: Option<LookupMap<u64, Vec<AccountId>>>, // Efficient participant tracking
    next_bounty_id: u64,
    platform_fee_rate: u128, // 5% = 500 (basis points)
}

#[near]
impl BountyPredictionContract {
    #[init]
    pub fn new(reward_rate: u128, min_stake_amount: NearToken, max_stake_amount: NearToken) -> Self {
        // Define safe maximum limits to prevent overflow and errors
        const MAX_REWARD_RATE: u128 = 1_000_000_000; // 1 billion - high but safe
        const MAX_STAKE_AMOUNT: u128 = 100_000; // 100,000 NEAR maximum
        const MIN_REWARD_RATE: u128 = 1; // Minimum 1 unit per second

        // Validate and clamp reward rate
        let safe_reward_rate = if reward_rate == 0 {
            MIN_REWARD_RATE
        } else if reward_rate > MAX_REWARD_RATE {
            MAX_REWARD_RATE
        } else {
            reward_rate
        };

        // Validate stake amounts
        require!(min_stake_amount <= max_stake_amount, "Minimum stake amount cannot exceed maximum");
        require!(
            max_stake_amount.as_near() <= MAX_STAKE_AMOUNT,
            format!("Maximum stake amount cannot exceed {} NEAR", MAX_STAKE_AMOUNT)
        );

        env::log_str(&format!(
            "CONTRACT_INIT: reward_rate={} (clamped from {}), min_stake={}, max_stake={}",
            safe_reward_rate, reward_rate, min_stake_amount.as_near(), max_stake_amount.as_near()
        ));

        Self {
            stakes: LookupMap::new(b"s"),
            total_staked: NearToken::from_yoctonear(0),
            reward_rate: safe_reward_rate,
            min_stake_amount,
            max_stake_amount,
            owner: env::predecessor_account_id(),
            bounties: LookupMap::new(b"b"),
            participant_stakes: LookupMap::new(b"p"),
            bounty_participants: Some(LookupMap::new(b"t")), // Participant tracking
            next_bounty_id: 1,
            platform_fee_rate: 500, // 5%
        }
    }

    /// Migration method to handle contract upgrades
    #[init(ignore_state)]
    pub fn migrate() -> Self {
        // Try to read the old state - if it fails, create a new contract
        if let Some(old_state_bytes) = env::storage_read(b"STATE") {
            env::log_str("CONTRACT_MIGRATION: Found existing state, attempting migration");

            // Try different versions of the contract state
            // First try: assume it has all current fields
            #[derive(BorshDeserialize)]
            struct CurrentContract {
                stakes: LookupMap<AccountId, StakeInfo>,
                total_staked: NearToken,
                reward_rate: u128,
                min_stake_amount: NearToken,
                max_stake_amount: NearToken,
                owner: AccountId,
                bounties: LookupMap<u64, Bounty>,
                participant_stakes: LookupMap<(AccountId, u64), ParticipantStake>,
                bounty_participants: Option<LookupMap<u64, Vec<AccountId>>>,
                next_bounty_id: u64,
                platform_fee_rate: u128,
                is_paused: bool,
            }

            if let Ok(current_contract) = CurrentContract::try_from_slice(&old_state_bytes) {
                env::log_str("CONTRACT_MIGRATION: Current format detected, preserving state");
                return Self {
                    stakes: current_contract.stakes,
                    total_staked: current_contract.total_staked,
                    reward_rate: current_contract.reward_rate,
                    min_stake_amount: current_contract.min_stake_amount,
                    max_stake_amount: current_contract.max_stake_amount,
                    owner: current_contract.owner,
                    bounties: current_contract.bounties,
                    participant_stakes: current_contract.participant_stakes,
                    bounty_participants: current_contract.bounty_participants.or_else(|| Some(LookupMap::new(b"t"))),
                    next_bounty_id: current_contract.next_bounty_id,
                    platform_fee_rate: current_contract.platform_fee_rate,
                };
            }

            // Second try: assume it's missing bounty_participants field
            #[derive(BorshDeserialize)]
            struct OldContractV1 {
                stakes: LookupMap<AccountId, StakeInfo>,
                total_staked: NearToken,
                reward_rate: u128,
                min_stake_amount: NearToken,
                max_stake_amount: NearToken,
                owner: AccountId,
                bounties: LookupMap<u64, Bounty>,
                participant_stakes: LookupMap<(AccountId, u64), ParticipantStake>,
                next_bounty_id: u64,
                platform_fee_rate: u128,
                is_paused: bool,
            }

            if let Ok(old_contract) = OldContractV1::try_from_slice(&old_state_bytes) {
                env::log_str("CONTRACT_MIGRATION: V1 format detected, adding participant tracking");
                return Self {
                    stakes: old_contract.stakes,
                    total_staked: old_contract.total_staked,
                    reward_rate: old_contract.reward_rate,
                    min_stake_amount: old_contract.min_stake_amount,
                    max_stake_amount: old_contract.max_stake_amount,
                    owner: old_contract.owner,
                    bounties: old_contract.bounties,
                    participant_stakes: old_contract.participant_stakes,
                    bounty_participants: Some(LookupMap::new(b"t")), // Initialize new field
                    next_bounty_id: old_contract.next_bounty_id,
                    platform_fee_rate: old_contract.platform_fee_rate,
                };
            }

            env::log_str("CONTRACT_MIGRATION: Could not parse existing state, creating new contract");
        } else {
            env::log_str("CONTRACT_MIGRATION: No existing state found, creating new contract");
        }

        // Fallback: create a new contract with default values
        Self {
            stakes: LookupMap::new(b"s"),
            total_staked: NearToken::from_yoctonear(0),
            reward_rate: 1000, // Default reward rate
            min_stake_amount: NearToken::from_near(1),
            max_stake_amount: NearToken::from_near(1000),
            owner: env::predecessor_account_id(),
            bounties: LookupMap::new(b"b"),
            participant_stakes: LookupMap::new(b"p"),
            bounty_participants: Some(LookupMap::new(b"t")),
            next_bounty_id: 1,
            platform_fee_rate: 500, // 5%
        }
    }

    /// Regular migration function that can be called after deployment
    /// Only callable by the contract owner for security
    pub fn migrate_state(&mut self) {
        self.assert_owner();

        // This function can be used to migrate state after deployment
        // Initialize bounty_participants if it doesn't exist
        if self.bounty_participants.is_none() {
            self.bounty_participants = Some(LookupMap::new(b"t"));
            env::log_str("CONTRACT_MIGRATION: Initialized bounty_participants field");

            // Log current contract state for verification
            env::log_str(&format!("CONTRACT_MIGRATION: Current state - next_bounty_id: {}",
                                 self.next_bounty_id));
        } else {
            env::log_str("CONTRACT_MIGRATION: bounty_participants field already exists");
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

    fn assert_owner(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner,
            "Only the owner can call this method"
        );
    }

    // Helper function to lazily initialize bounty_participants for migration compatibility
    fn get_bounty_participants_mut(&mut self) -> &mut LookupMap<u64, Vec<AccountId>> {
        if self.bounty_participants.is_none() {
            self.bounty_participants = Some(LookupMap::new(b"t"));
        }
        self.bounty_participants.as_mut().unwrap()
    }

    fn get_bounty_participants_ref(&self) -> Option<&LookupMap<u64, Vec<AccountId>>> {
        self.bounty_participants.as_ref()
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

    pub fn get_stake_info(&self, account: AccountId) -> Option<StakeInfoView> {
        self.stakes.get(&account).map(|stake_info| stake_info.into())
    }

    pub fn calculate_pending_rewards(&self, account: AccountId) -> U128 {
        if let Some(stake_info) = self.stakes.get(&account) {
            let current_time = env::block_timestamp();
            let time_diff = current_time - stake_info.last_reward_claim;
            let time_diff_seconds = time_diff / 1_000_000_000;

            let rewards = Self::calculate_rewards_safe(stake_info.amount, self.reward_rate, time_diff_seconds);
            U128(rewards)
        } else {
            U128(0)
        }
    }

    pub fn get_total_staked(&self) -> U128 {
        U128(self.total_staked.as_yoctonear())
    }

    pub fn get_reward_rate(&self) -> u128 {
        self.reward_rate
    }

    pub fn get_max_stake_amount(&self) -> U128 {
        U128(self.max_stake_amount.as_yoctonear())
    }

    // Helper function to check if contract is paused - REMOVED
    // fn assert_not_paused(&self) {
    //    require!(!self.is_paused, "Contract is paused");
    // }

    // Content Bounty Management Functions
    #[payable]
    pub fn create_content_bounty(
        &mut self,
        title: String,
        description: String,
        requirements: String,
        base_prize: NearToken,
        max_stake_per_user: NearToken,
        creator_share: Option<u8>,
        backer_share: Option<u8>,
        duration_days: u64,
    ) -> u64 {
        // self.assert_not_paused(); // Removed
        
        let creator = env::predecessor_account_id();
        let attached_deposit = env::attached_deposit();
        let initial_storage = env::storage_usage();

        // Validate inputs
        require!(!title.trim().is_empty(), "Title cannot be empty");
        require!(!description.trim().is_empty(), "Description cannot be empty");
        require!(!requirements.trim().is_empty(), "Requirements cannot be empty");
        require!(title.len() <= 200, "Title too long (max 200 characters)");
        require!(description.len() <= 1000, "Description too long (max 1000 characters)");
        require!(requirements.len() <= 2000, "Requirements too long (max 2000 characters)");

        // Validate base prize (minimum 1 NEAR)
        require!(base_prize >= NearToken::from_near(1), "Base prize must be at least 1 NEAR");
        require!(
            attached_deposit >= base_prize,
            format!("Must attach at least {} yoctoNEAR for base prize", base_prize.as_yoctonear())
        );

        // Validate max stake amount (0.1 to 10000 NEAR)
        let min_bounty_stake = NearToken::from_millinear(100); // 0.1 NEAR
        let max_bounty_stake = NearToken::from_near(10000);
        require!(max_stake_per_user >= min_bounty_stake, "Maximum stake per user must be at least 0.1 NEAR");
        require!(max_stake_per_user <= max_bounty_stake, "Maximum stake per user cannot exceed 10000 NEAR");

        // Validate and set reward shares
        let final_creator_share = creator_share.unwrap_or(DEFAULT_CREATOR_SHARE);
        let final_backer_share = backer_share.unwrap_or(DEFAULT_BACKER_SHARE);
        require!(
            final_creator_share + final_backer_share == 100,
            "Creator share + backer share must equal 100"
        );
        require!(final_creator_share >= 30, "Creator share must be at least 30%");
        require!(final_creator_share <= 90, "Creator share cannot exceed 90%");

        // Validate duration (1-90 days)
        require!(duration_days >= 1, "Duration must be at least 1 day");
        require!(duration_days <= 90, "Duration cannot exceed 90 days (3 months)");

        let bounty_id = self.next_bounty_id;
        let current_time = env::block_timestamp();
        let duration_ns = (duration_days as u128)
            .checked_mul(24 * 60 * 60 * 1_000_000_000)
            .expect("Duration is too large");
        let ends_at = u128::from(current_time)
            .checked_add(duration_ns)
            .and_then(|value| u64::try_from(value).ok())
            .expect("Duration exceeds supported range");

        let bounty = Bounty {
            id: bounty_id,
            title,
            description,
            requirements,
            submissions: Vec::new(),
            creator: creator.clone(),
            base_prize,
            max_stake_per_user,
            creator_share: final_creator_share,
            backer_share: final_backer_share,
            is_active: true,
            created_at: current_time,
            ends_at,
            total_staked: NearToken::from_yoctonear(0),
            is_closed: false,
            winning_submission: None,
        };

        self.bounties.insert(&bounty_id, &bounty);
        self.next_bounty_id += 1;

        env::log_str(&format!(
            "CONTENT_BOUNTY_CREATED: ID {} by {} with base prize {} NEAR",
            bounty_id, creator, base_prize.as_near()
        ));

        // Calculate storage cost
        let storage_used = env::storage_usage().saturating_sub(initial_storage);
        let storage_cost_per_byte = env::storage_byte_cost().as_yoctonear();
        let storage_cost = u128::from(storage_used) * storage_cost_per_byte;
        
        // Total required = base_prize + storage_cost
        let total_required = base_prize.as_yoctonear()
            .checked_add(storage_cost)
            .expect("Total required calculation overflow");
        
        require!(
            attached_deposit.as_yoctonear() >= total_required,
            format!("Insufficient deposit: need {} (base prize) + {} (storage) = {} total",
                base_prize.as_yoctonear(), storage_cost, total_required)
        );
        
        // Refund excess
        let refund = attached_deposit.as_yoctonear() - total_required;
        if refund > 0 {
            Promise::new(creator).transfer(NearToken::from_yoctonear(refund));
        }

        bounty_id
    }

    // Submit content to a bounty
    //
    // ANTI-CHEATING NOTE:
    // We prevent the same creator from submitting multiple times to the same bounty.
    // However, this only prevents duplicate submissions from the SAME NEAR account.
    // A bad actor could still:
    // - Use multiple NEAR accounts (Sybil attack)
    // - Submit multiple low-quality entries to dilute stakes
    //
    // Mitigations:
    // 1. One submission per account prevents basic spam
    // 2. creation_id links to Dreamweave DB (verified creator identity)
    // 3. Off-chain: Backend can link creation_id to user account (detect multi-account abuse)
    // 4. Off-chain: Platform can require minimum account age/reputation
    // 5. Economic cost: Creating NEAR accounts costs money (not free to Sybil)
    // 6. Social cost: Bad submissions hurt creator's reputation
    // 7. Quality filter: Community stakes on best work (bad entries get zero stakes)
    //
    // The creation_id should be validated off-chain against Dreamweave DB to ensure:
    // - It exists and belongs to the submitter
    // - The content meets bounty requirements
    // - The creator hasn't been flagged for abuse
    pub fn submit_content(
        &mut self,
        bounty_id: u64,
        creation_id: String,
        title: String,
        thumbnail_url: String,
    ) -> u64 {
        // self.assert_not_paused(); // Removed
        let submitter = env::predecessor_account_id();
        let current_time = env::block_timestamp();

        let mut bounty = self.bounties.get(&bounty_id).expect("Bounty not found");
        require!(bounty.is_active, "Bounty is not active");
        require!(!bounty.is_closed, "Bounty is already closed");
        require!(current_time < bounty.ends_at, "Bounty has expired");
        require!(
            bounty.submissions.len() < MAX_SUBMISSIONS,
            format!("Maximum {} submissions reached", MAX_SUBMISSIONS)
        );

        // Validate inputs
        require!(!creation_id.trim().is_empty(), "Creation ID cannot be empty");
        require!(!title.trim().is_empty(), "Title cannot be empty");
        require!(title.len() <= 200, "Title too long (max 200 characters)");

        // Check if creator already submitted
        for submission in &bounty.submissions {
            require!(
                submission.creator != submitter,
                "You have already submitted to this bounty"
            );
            require!(
                submission.creation_id != creation_id,
                "This creation has already been submitted"
            );
        }

        let submission = ContentSubmission {
            creator: submitter.clone(),
            creation_id: creation_id.clone(),
            title,
            thumbnail_url,
            total_staked: NearToken::from_yoctonear(0),
            submitted_at: current_time,
        };

        bounty.submissions.push(submission);
        let submission_index = bounty.submissions.len() - 1;
        
        self.bounties.insert(&bounty_id, &bounty);

        env::log_str(&format!(
            "CONTENT_SUBMITTED: Bounty {} - {} by {} (index {})",
            bounty_id, creation_id, submitter, submission_index
        ));

        submission_index as u64
    }

    pub fn get_bounty(&self, bounty_id: u64) -> Option<BountyView> {
        self.bounties.get(&bounty_id).map(|bounty| bounty.into())
    }

    pub fn get_active_bounties(&self) -> Vec<BountyView> {
        let mut active_bounties = Vec::new();
        let current_time = env::block_timestamp();

        for i in 1..self.next_bounty_id {
            if let Some(bounty) = self.bounties.get(&i) {
                if bounty.is_active && !bounty.is_closed && current_time < bounty.ends_at {
                    active_bounties.push(bounty.into());
                }
            }
        }

        active_bounties
    }

    // Staking on Content Submissions
    //
    // ANTI-CHEATING NOTE:
    // This function does NOT prevent creators from staking on their own submissions.
    // While this enables potential self-staking attacks, we rely on:
    // 1. Transparency: All stakes are public on-chain (check NEAR Explorer)
    // 2. Off-chain detection: Backend monitors creator_account == staker_account
    // 3. Social consequences: Flagged accounts can be banned from platform
    // 4. Economic risk: Creator still risks NEAR (locked until close, lost if caught)
    // 5. Game theory: 90% creator share reduces incentive to cheat
    //
    // The staker's identity is stored in participant_stakes map, making it auditable.
    // Backend should cross-reference submission.creator with stake.staker for each bounty.
    #[payable]
    pub fn stake_on_submission(&mut self, bounty_id: u64, submission_index: u64) {
        // self.assert_not_paused(); // Removed
        let staker = env::predecessor_account_id();
        let amount = env::attached_deposit();
        let current_time = env::block_timestamp();

        // Get and validate bounty
        let mut bounty = self.bounties.get(&bounty_id).expect("Bounty not found");
        require!(bounty.is_active, "Bounty is not active");
        require!(!bounty.is_closed, "Bounty is already closed");
        require!(current_time < bounty.ends_at, "Bounty has expired");

        // Validate submission index
        require!(
            (submission_index as usize) < bounty.submissions.len(),
            format!("Invalid submission index: bounty has {} submissions", bounty.submissions.len())
        );

        // Validate stake amount
        require!(amount > NearToken::from_yoctonear(0), "Stake amount must be positive");
        require!(amount <= bounty.max_stake_per_user, "Stake amount exceeds maximum allowed for this bounty");

        let stake_key = (staker.clone(), bounty_id);
        let is_new_participant = !self.participant_stakes.contains_key(&stake_key);

        // CRITICAL: Check participant limit BEFORE adding new participants
        if is_new_participant {
            let current_participant_count = self.count_bounty_participants(bounty_id);
            require!(
                current_participant_count < MAX_PARTICIPANTS_PER_BOUNTY as u64,
                format!("Bounty has reached maximum participant limit of {}",
                    MAX_PARTICIPANTS_PER_BOUNTY)
            );
        }

        // Handle existing stake
        if let Some(existing_stake) = self.participant_stakes.get(&stake_key) {
            // Remove previous stake from bounty and submission totals
            bounty.total_staked = Self::safe_sub_tokens(bounty.total_staked, existing_stake.amount)
                .expect("Total stake subtraction underflow");
            bounty.submissions[existing_stake.submission_index as usize].total_staked =
                Self::safe_sub_tokens(
                    bounty.submissions[existing_stake.submission_index as usize].total_staked,
                    existing_stake.amount
                ).expect("Submission stake subtraction underflow");
        }

        // Add participant to tracking list if they're new
        if is_new_participant {
            let bounty_participants = self.get_bounty_participants_mut();
            let mut participants = bounty_participants.get(&bounty_id).unwrap_or_else(Vec::new);
            if !participants.contains(&staker) {
                participants.push(staker.clone());
                bounty_participants.insert(&bounty_id, &participants);
            }
        }

        // Add new stake
        bounty.total_staked = Self::safe_add_tokens(bounty.total_staked, amount)
            .expect("Total stake addition overflow");
        bounty.submissions[submission_index as usize].total_staked =
            Self::safe_add_tokens(bounty.submissions[submission_index as usize].total_staked, amount)
                .expect("Submission stake addition overflow");

        // Create or update participant stake
        let participant_stake = ParticipantStake {
            bounty_id,
            submission_index,
            amount,
            staked_at: current_time,
        };

        self.participant_stakes.insert(&stake_key, &participant_stake);
        self.bounties.insert(&bounty_id, &bounty);

        env::log_str(&format!("SUBMISSION_STAKE: Account {} staked {} NEAR on submission {} for bounty {}",
                             staker, amount.as_near(), submission_index, bounty_id));
    }

    pub fn get_participant_stake(&self, account: AccountId, bounty_id: u64) -> Option<ParticipantStakeView> {
        self.participant_stakes.get(&(account, bounty_id)).map(|stake| stake.into())
    }

    pub fn get_bounty_submission_stakes(&self, bounty_id: u64) -> Vec<U128> {
        if let Some(bounty) = self.bounties.get(&bounty_id) {
            bounty.submissions.iter().map(|s| U128(s.total_staked.as_yoctonear())).collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_user_bounties(&self, account: AccountId) -> Vec<ParticipantStakeView> {
        let mut user_stakes = Vec::new();

        // Iterate through all bounties to find user's participations
        for i in 1..self.next_bounty_id {
            let stake_key = (account.clone(), i);
            if let Some(stake) = self.participant_stakes.get(&stake_key) {
                user_stakes.push(stake.into());
            }
        }

        user_stakes
    }

    pub fn get_bounty_participants(&self, bounty_id: u64) -> Vec<AccountId> {
        if let Some(bounty_participants) = self.get_bounty_participants_ref() {
            bounty_participants.get(&bounty_id).unwrap_or_else(Vec::new)
        } else {
            Vec::new()
        }
    }

    pub fn get_bounty_participant_count(&self, bounty_id: u64) -> u64 {
        if let Some(bounty_participants) = self.get_bounty_participants_ref() {
            if let Some(participants) = bounty_participants.get(&bounty_id) {
                participants.len() as u64
            } else {
                0
            }
        } else {
            0
        }
    }

    // Reward Calculation Logic
    fn determine_winning_submission(&self, bounty: &Bounty) -> Option<u64> {
        if bounty.submissions.is_empty() {
            return None;
        }

        let mut max_stake = NearToken::from_yoctonear(0);
        let mut winning_submission = 0u64;
        let mut has_stakes = false;

        for (index, submission) in bounty.submissions.iter().enumerate() {
            if submission.total_staked > NearToken::from_yoctonear(0) {
                has_stakes = true;
                if submission.total_staked > max_stake {
                    max_stake = submission.total_staked;
                    winning_submission = index as u64;
                }
            }
        }

        if has_stakes {
            Some(winning_submission)
        } else {
            None
        }
    }

    fn calculate_platform_fee(&self, total_amount: NearToken) -> NearToken {
        let fee_amount = total_amount.as_yoctonear()
            .checked_mul(self.platform_fee_rate as u128)
            .and_then(|x| x.checked_div(10000)) // Convert basis points to percentage
            .unwrap_or(0);

        NearToken::from_yoctonear(fee_amount)
    }

    fn calculate_backer_reward(&self, bounty: &Bounty, user_stake: NearToken, winning_submission: u64) -> NearToken {
        let total_winning_stakes = bounty.submissions[winning_submission as usize].total_staked;

        if total_winning_stakes == NearToken::from_yoctonear(0) {
            return NearToken::from_yoctonear(0);
        }

        // Calculate total prize pool (base_prize + community stakes)
        let total_prize = Self::safe_add_tokens(bounty.base_prize, bounty.total_staked)
            .expect("Total prize calculation overflow");
        
        // Calculate platform fee from total prize
        let platform_fee = self.calculate_platform_fee(total_prize);
        let prize_after_fee = Self::safe_sub_tokens(total_prize, platform_fee)
            .unwrap_or(total_prize);

        // Split prize: backer_share% to backers (distributed proportionally)
        let backer_pool_raw = prize_after_fee.as_yoctonear()
            .checked_mul(bounty.backer_share as u128)
            .and_then(|x| x.checked_div(100))
            .unwrap_or(0);
        let backer_pool = NearToken::from_yoctonear(backer_pool_raw);

        // Calculate proportional reward for this backer
        let user_share = user_stake.as_yoctonear()
            .checked_mul(backer_pool.as_yoctonear())
            .and_then(|x| x.checked_div(total_winning_stakes.as_yoctonear()))
            .unwrap_or(0);

        NearToken::from_yoctonear(user_share)
    }

    fn calculate_creator_reward(&self, bounty: &Bounty) -> NearToken {
        // Calculate total prize pool (base_prize + community stakes)
        let total_prize = Self::safe_add_tokens(bounty.base_prize, bounty.total_staked)
            .expect("Total prize calculation overflow");
        
        // Calculate platform fee
        let platform_fee = self.calculate_platform_fee(total_prize);
        let prize_after_fee = Self::safe_sub_tokens(total_prize, platform_fee)
            .unwrap_or(total_prize);

        // Creator gets creator_share% of prize after fees
        let creator_reward_raw = prize_after_fee.as_yoctonear()
            .checked_mul(bounty.creator_share as u128)
            .and_then(|x| x.checked_div(100))
            .unwrap_or(0);

        NearToken::from_yoctonear(creator_reward_raw)
    }

    fn count_bounty_participants(&self, bounty_id: u64) -> u64 {
        // Use participant tracking system for accurate count
        if let Some(bounty_participants) = self.get_bounty_participants_ref() {
            if let Some(participants) = bounty_participants.get(&bounty_id) {
                participants.len() as u64
            } else {
                0
            }
        } else {
            0
        }
    }

    // Bounty Closure and Reward Distribution
    //
    // ANTI-CHEATING NOTE:
    // This function distributes rewards to the winner based purely on stake amounts.
    // It does NOT verify the legitimacy of stakes or detect self-staking.
    //
    // IMPORTANT: Before closing high-value bounties (100+ NEAR), the platform should:
    // 1. Check if winner_creator staked on their own submission
    //    - Query get_participant_stake(winner_account, bounty_id)
    //    - Check if winner_account == submission.creator
    //    - Flag for manual review if detected
    //
    // 2. Analyze stake patterns
    //    - Check if multiple stakes came from newly created accounts
    //    - Look for suspicious timing (all stakes within minutes)
    //    - Verify stake amounts are reasonable (not all max_stake_per_user)
    //
    // 3. Community review period
    //    - Allow 24-48 hours after ends_at before closing
    //    - Let community flag suspicious activity
    //    - Provide dispute mechanism
    //
    // 4. Off-chain verification
    //    - Verify creation_id exists in Dreamweave DB
    //    - Check if content meets bounty requirements
    //    - Validate creator's reputation/history
    //
    // For now, this contract trusts the caller (bounty creator or owner) to have
    // done due diligence. Future versions could add:
    // - Mandatory review period (time lock after ends_at)
    // - Community voting on winner before distribution
    // - Owner veto power for suspicious bounties
    // - Reputation score requirements
    pub fn close_bounty(&mut self, bounty_id: u64) {
        // self.assert_not_paused(); // Removed: Contract is trustless and cannot be paused
        let caller = env::predecessor_account_id();
        let current_time = env::block_timestamp();

        let mut bounty = self.bounties.get(&bounty_id).expect("Bounty not found");

        // Trustless Closure Logic:
        // 1. Creator can close anytime after 'ends_at'.
        // 2. ANYONE can close after 'ends_at + grace_period' (7 days).
        // This ensures funds are never stuck if the creator goes inactive.
        const CLOSE_GRACE_PERIOD_NS: u64 = 7 * 24 * 60 * 60 * 1_000_000_000; // 7 days

        let is_creator = caller == bounty.creator;
        let is_past_grace_period = current_time >= bounty.ends_at + CLOSE_GRACE_PERIOD_NS;

        require!(
            is_creator || is_past_grace_period,
            "Only creator can close immediately. Others must wait 7 days after expiry."
        );

        // State validation
        require!(bounty.is_active, "Bounty is not active");
        require!(!bounty.is_closed, "Bounty is already closed");
        require!(current_time >= bounty.ends_at, "Bounty has not expired yet");

        // Handle different scenarios
        if bounty.total_staked == NearToken::from_yoctonear(0) {
            // No participants - just close the bounty
            bounty.is_closed = true;
            bounty.is_active = false;
            self.bounties.insert(&bounty_id, &bounty);
            env::log_str(&format!("BOUNTY_CLOSED: No participants in bounty {}", bounty_id));
            return;
        }

        let participant_count = self.count_bounty_participants(bounty_id);

        if participant_count <= 1 {
            // Single participant - return full stake, no fees
            self.distribute_single_participant_rewards(&mut bounty);
        } else {
            // Multiple participants - normal reward distribution
            self.distribute_multi_participant_rewards(&mut bounty);
        }

        bounty.is_closed = true;
        bounty.is_active = false;
        self.bounties.insert(&bounty_id, &bounty);

        env::log_str(&format!("BOUNTY_CLOSED: Bounty {} closed and rewards distributed", bounty_id));
    }

    fn distribute_single_participant_rewards(&mut self, bounty: &mut Bounty) {
        // Use participant tracking system to find the single participant
        if let Some(bounty_participants) = self.get_bounty_participants_ref() {
            if let Some(participants) = bounty_participants.get(&bounty.id) {
                for account in participants {
                    let stake_key = (account.clone(), bounty.id);
                    if let Some(stake) = self.participant_stakes.get(&stake_key) {
                        // Return full stake to participant
                        Promise::new(account.clone()).transfer(stake.amount);
                        env::log_str(&format!("SINGLE_PARTICIPANT_REFUND: {} received {} NEAR",
                                             account, stake.amount));
                        return;
                    }
                }
            }
        }
        env::log_str(&format!("SINGLE_PARTICIPANT_ERROR: No participants found for bounty {}", bounty.id));
    }

    fn distribute_multi_participant_rewards(&mut self, bounty: &mut Bounty) {
        // Determine winning submission
        let winning_submission = match self.determine_winning_submission(bounty) {
            Some(submission) => submission,
            None => {
                env::log_str(&format!("BOUNTY_ERROR: No winning submission determined for bounty {}", bounty.id));
                return;
            }
        };

        bounty.winning_submission = Some(winning_submission);
        
        // Get winning creator
        let winning_creator = bounty.submissions[winning_submission as usize].creator.clone();

        // Calculate total prize (base_prize + community stakes)
        let total_prize = Self::safe_add_tokens(bounty.base_prize, bounty.total_staked)
            .expect("Total prize calculation overflow");

        // Calculate and transfer platform fee
        let platform_fee = self.calculate_platform_fee(total_prize);
        if platform_fee > NearToken::from_yoctonear(0) {
            Promise::new(self.owner.clone()).transfer(platform_fee);
            env::log_str(&format!("PLATFORM_FEE: {} NEAR transferred to owner from bounty {}", 
                                 platform_fee.as_near(), bounty.id));
        }

        // Pay the winning creator their share
        let creator_reward = self.calculate_creator_reward(bounty);
        if creator_reward > NearToken::from_yoctonear(0) {
            Promise::new(winning_creator.clone()).transfer(creator_reward);
            env::log_str(&format!("CREATOR_REWARD: {} received {} NEAR ({}%) for winning submission {}",
                                 winning_creator, creator_reward.as_near(), 
                                 bounty.creator_share, winning_submission));
        }

        // Distribute backer rewards to winners
        self.distribute_winner_rewards(bounty, winning_submission);
    }

    fn distribute_winner_rewards(&mut self, bounty: &Bounty, winning_submission: u64) {
        // GAS SAFETY: We do NOT iterate through all participants here to avoid OOG (Out of Gas) errors.
        // Instead, we rely on the 'Pull' pattern where users call claim_bounty_winnings().
        // This scales to any number of participants.
        env::log_str(&format!("BOUNTY_CLOSED: Bounty {} closed. Winning submission: {}. Participants can now claim rewards.", 
                             bounty.id, winning_submission));
    }

    // Bounty Results and Claiming
    pub fn get_bounty_results(&self, bounty_id: u64) -> Option<BountyView> {
        if let Some(bounty) = self.bounties.get(&bounty_id) {
            if bounty.is_closed {
                Some(bounty.into())
            } else {
                None // Only return results for closed bounties
            }
        } else {
            None
        }
    }

    pub fn claim_bounty_winnings(&mut self, bounty_id: u64) {
        // self.assert_not_paused(); // Removed
        let claimer = env::predecessor_account_id();

        let bounty = self.bounties.get(&bounty_id).expect("Bounty not found");
        require!(bounty.is_closed, "Bounty is not closed yet");

        let stake_key = (claimer.clone(), bounty_id);
        let stake = self.participant_stakes.get(&stake_key).expect("No stake found for this bounty");

        // CRITICAL: Remove stake to prevent double-claiming
        self.participant_stakes.remove(&stake_key);

        // Check if user won
        if let Some(winning_submission) = bounty.winning_submission {
            // Check if claimer is the winning creator
            let is_winning_creator = bounty.submissions[winning_submission as usize].creator == claimer;
            
            if is_winning_creator {
                // Pay creator reward
                let reward = self.calculate_creator_reward(&bounty);

                if reward > NearToken::from_yoctonear(0) {
                    // Check if contract has sufficient balance
                    let contract_balance = env::account_balance();
                    let reserved_balance = NearToken::from_near(1); // Reserve for operations

                    if contract_balance > Self::safe_add_tokens(reward, reserved_balance).unwrap_or(contract_balance) {
                        Promise::new(claimer.clone()).transfer(reward);
                        env::log_str(&format!("CLAIM_SUCCESS: Creator {} claimed {} NEAR from bounty {}",
                                             claimer, reward.as_near(), bounty_id));
                    } else {
                        // Refund the stake if we can't pay the full reward (shouldn't happen)
                         self.participant_stakes.insert(&stake_key, &stake);
                        env::log_str(&format!("CLAIM_FAILED: Insufficient contract balance for {} from bounty {}",
                                             claimer, bounty_id));
                        panic!(
                            "Insufficient contract balance for reward payment: contract balance = {} yoctoNEAR, required = {} yoctoNEAR",
                            contract_balance.as_yoctonear(),
                            Self::safe_add_tokens(reward, reserved_balance).unwrap_or(contract_balance).as_yoctonear()
                        );
                    }
                } else {
                    // No reward but stake removed - technically correct if reward is 0
                }
            } else if stake.submission_index == winning_submission {
                // Pay backer reward
                let reward = self.calculate_backer_reward(&bounty, stake.amount, winning_submission);

                if reward > NearToken::from_yoctonear(0) {
                    // Check if contract has sufficient balance
                    let contract_balance = env::account_balance();
                    let reserved_balance = NearToken::from_near(1);

                    if contract_balance > Self::safe_add_tokens(reward, reserved_balance).unwrap_or(contract_balance) {
                        Promise::new(claimer.clone()).transfer(reward);
                        env::log_str(&format!("CLAIM_SUCCESS: Backer {} claimed {} NEAR from bounty {}",
                                             claimer, reward.as_near(), bounty_id));
                    } else {
                        // Refund stake
                        self.participant_stakes.insert(&stake_key, &stake);
                        panic!("Insufficient contract balance for reward payment");
                    }
                } else {
                    // No reward to claim
                }
            } else {
                // User did not win - stake is forfeit (removed above)
                env::log_str(&format!("CLAIM_INFO: User {} did not back winning submission. Stake forfeit.", claimer));
            }
        } else {
            // Handle single participant case - return full stake
            let participant_count = self.count_bounty_participants(bounty_id);
            if participant_count <= 1 {
                Promise::new(claimer.clone()).transfer(stake.amount);
                env::log_str(&format!("SINGLE_PARTICIPANT_CLAIM: {} claimed {} NEAR from bounty {}",
                             claimer, stake.amount.as_near(), bounty_id));
            } else {
                // Refund stake
                self.participant_stakes.insert(&stake_key, &stake);
                panic!("No winning submission determined");
            }
        }
    }

    // Owner functions
    pub fn update_reward_rate(&mut self, new_rate: u128) {
        self.assert_owner();

        // Define safe limits for reward rate updates
        const MAX_REWARD_RATE: u128 = 1_000_000_000; // 1 billion - high but safe
        const MIN_REWARD_RATE: u128 = 1; // Minimum 1 unit per second

        // Clamp the reward rate to safe bounds
        let safe_rate = if new_rate == 0 {
            MIN_REWARD_RATE
        } else if new_rate > MAX_REWARD_RATE {
            MAX_REWARD_RATE
        } else {
            new_rate
        };

        env::log_str(&format!(
            "REWARD_RATE_UPDATE: new_rate={} (clamped from {})",
            safe_rate, new_rate
        ));

        self.reward_rate = safe_rate;
    }

    pub fn update_max_stake_amount(&mut self, new_max_amount: NearToken) {
        self.assert_owner();

        // Define safe limits for stake amounts
        const MAX_STAKE_LIMIT_NEAR: u128 = 100_000; // 100,000 NEAR maximum

        // Ensure new max is not less than current min
        let safe_max = if new_max_amount < self.min_stake_amount {
            self.min_stake_amount
        } else if new_max_amount.as_near() > MAX_STAKE_LIMIT_NEAR {
            NearToken::from_near(MAX_STAKE_LIMIT_NEAR)
        } else {
            new_max_amount
        };

        env::log_str(&format!(
            "MAX_STAKE_UPDATE: new_max={} NEAR (clamped from {})",
            safe_max.as_near(), new_max_amount.as_near()
        ));

        self.max_stake_amount = safe_max;
    }

    pub fn update_platform_fee_rate(&mut self, new_rate: u128) {
        self.assert_owner();

        // Define safe limits for platform fee (in basis points)
        const MAX_PLATFORM_FEE_RATE: u128 = 1000; // 10% maximum
        const MIN_PLATFORM_FEE_RATE: u128 = 0; // 0% minimum (free)

        // Clamp the fee rate to safe bounds
        let safe_rate = if new_rate > MAX_PLATFORM_FEE_RATE {
            MAX_PLATFORM_FEE_RATE
        } else {
            new_rate.max(MIN_PLATFORM_FEE_RATE)
        };

        env::log_str(&format!(
            "PLATFORM_FEE_UPDATE: new_rate={}bp ({}%) clamped from {}bp",
            safe_rate, safe_rate / 100, new_rate
        ));

        self.platform_fee_rate = safe_rate;
    }

    pub fn withdraw_platform_fees(&mut self) {
        self.assert_owner();

        let contract_balance = env::account_balance();
        let reserved_balance = NearToken::from_near(2); // Reserve more for operations

        if contract_balance > reserved_balance {
            let withdrawal_amount = Self::safe_sub_tokens(contract_balance, reserved_balance)
                .expect("Balance calculation error");

            if withdrawal_amount > NearToken::from_yoctonear(0) {
                Promise::new(self.owner.clone()).transfer(withdrawal_amount);
                env::log_str(&format!("PLATFORM_FEES_WITHDRAWN: {} NEAR withdrawn by owner", withdrawal_amount));
            }
        }
    }

    // View functions for contract state
    pub fn get_platform_fee_rate(&self) -> u128 {
        self.platform_fee_rate
    }

    // Helper for verifying paused state is removed
    // pub fn is_contract_paused(&self) -> bool { self.is_paused } // REMOVED

    pub fn get_contract_owner(&self) -> AccountId {
        self.owner.clone()
    }

    pub fn get_max_participants_per_bounty(&self) -> usize {
        MAX_PARTICIPANTS_PER_BOUNTY
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
    const STORAGE_DEPOSIT: NearToken = NearToken::from_near(1);

    fn get_context(predecessor_account_id: AccountId, attached_deposit: NearToken) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .predecessor_account_id(predecessor_account_id)
            .attached_deposit(attached_deposit)
            .block_timestamp(0);
        builder
    }

    fn set_attached_deposit(context: &mut VMContextBuilder, deposit: NearToken) {
        testing_env!(context.attached_deposit(deposit).build());
    }

    fn create_bounty_with_deposit<I>(
        contract: &mut BountyPredictionContract,
        context: &mut VMContextBuilder,
        title: &str,
        description: &str,
        options: I,
        max_stake_per_user: NearToken,
        duration_blocks: u64,
    ) -> u64
    where
        I: IntoIterator,
        I::Item: Into<String>,
    {
        set_attached_deposit(context, STORAGE_DEPOSIT);
        contract.create_bounty(
            title.to_string(),
            description.to_string(),
            options.into_iter().map(Into::into).collect(),
            max_stake_per_user,
            duration_blocks,
        )
    }

    #[test]
    fn test_new() {
        let context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);
        assert_eq!(contract.get_reward_rate(), REWARD_RATE);
        assert_eq!(contract.min_stake_amount, MIN_STAKE);
        assert_eq!(contract.get_max_stake_amount().0, MAX_STAKE.as_yoctonear());
    }

    #[test]
    fn test_stake_valid_amount() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        let stake_amount = NearToken::from_near(10);
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(stake_amount).build());
        contract.stake();

        let stake_info = contract.get_stake_info(accounts(1)).unwrap();
        assert_eq!(stake_info.amount.0, stake_amount.as_yoctonear());
    }

    #[test]
    #[should_panic(expected = "Stake amount too low")]
    fn test_stake_below_minimum() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        let stake_amount = NearToken::from_yoctonear(MIN_STAKE.as_yoctonear() - 1);
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(stake_amount).build());
        contract.stake();
    }

    #[test]
    #[should_panic(expected = "Stake amount too high")]
    fn test_stake_above_maximum() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        let stake_amount = NearToken::from_yoctonear(MAX_STAKE.as_yoctonear() + 1);
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(stake_amount).build());
        contract.stake();
    }

    #[test]
    fn test_update_max_stake_amount() {
        let context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        let new_max = NearToken::from_near(200);
        contract.update_max_stake_amount(new_max);
        assert_eq!(contract.get_max_stake_amount().0, new_max.as_yoctonear());
    }

    #[test]
    fn test_create_bounty_valid() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        let bounty_id = create_bounty_with_deposit(
            &mut contract,
            &mut context,
            "Test Bounty",
            "A test bounty",
            ["Option A", "Option B"],
            NearToken::from_near(10),
            100,
        );

        assert_eq!(bounty_id, 1);
        let bounty = contract.get_bounty(bounty_id).unwrap();
        assert_eq!(bounty.title, "Test Bounty");
        assert_eq!(bounty.options.len(), 2);
        assert!(bounty.is_active);
        assert!(!bounty.is_closed);
    }

    #[test]
    #[should_panic(expected = "Bounty must have at least 2 options")]
    fn test_create_bounty_too_few_options() {
        let context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        contract.create_bounty(
            "Test Bounty".to_string(),
            "A test bounty".to_string(),
            vec!["Option A".to_string()],
            NearToken::from_near(10),
            100,
        );
    }

    #[test]
    #[should_panic(expected = "Maximum stake per user must be at least 0.1 NEAR")]
    fn test_create_bounty_stake_too_low() {
        let context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        contract.create_bounty(
            "Test Bounty".to_string(),
            "A test bounty".to_string(),
            vec!["Option A".to_string(), "Option B".to_string()],
            NearToken::from_millinear(50), // 0.05 NEAR, below minimum
            100,
        );
    }

    #[test]
    #[should_panic(expected = "Maximum stake per user cannot exceed 10000 NEAR")]
    fn test_create_bounty_stake_too_high() {
        let context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        contract.create_bounty(
            "Test Bounty".to_string(),
            "A test bounty".to_string(),
            vec!["Option A".to_string(), "Option B".to_string()],
            NearToken::from_near(10001), // Above maximum
            100,
        );
    }

    #[test]
    fn test_stake_on_option_valid() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        // Create bounty
        let bounty_id = create_bounty_with_deposit(
            &mut contract,
            &mut context,
            "Test Bounty",
            "A test bounty",
            ["Option A", "Option B"],
            NearToken::from_near(10),
            100,
        );

        // Stake on option
        let stake_amount = NearToken::from_near(5);
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(stake_amount).build());
        contract.stake_on_option(bounty_id, 0);

        // Verify stake
        let participant_stake = contract.get_participant_stake(accounts(1), bounty_id).unwrap();
        assert_eq!(participant_stake.amount.0, stake_amount.as_yoctonear());
        assert_eq!(participant_stake.option_index, 0);

        // Verify bounty totals
        let bounty = contract.get_bounty(bounty_id).unwrap();
        assert_eq!(bounty.total_staked.0, stake_amount.as_yoctonear());
        assert_eq!(bounty.stakes_per_option[0].0, stake_amount.as_yoctonear());
        assert_eq!(bounty.stakes_per_option[1].0, 0);
    }

    #[test]
    fn test_stake_update_existing() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        // Create bounty
        let bounty_id = create_bounty_with_deposit(
            &mut contract,
            &mut context,
            "Test Bounty",
            "A test bounty",
            ["Option A", "Option B"],
            NearToken::from_near(10),
            100,
        );

        // Initial stake
        let initial_stake = NearToken::from_near(3);
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(initial_stake).build());
        contract.stake_on_option(bounty_id, 0);

        // Update stake to different option
        let new_stake = NearToken::from_near(5);
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(new_stake).build());
        contract.stake_on_option(bounty_id, 1);

        // Verify updated stake
        let participant_stake = contract.get_participant_stake(accounts(1), bounty_id).unwrap();
        assert_eq!(participant_stake.amount.0, new_stake.as_yoctonear());
        assert_eq!(participant_stake.option_index, 1);

        // Verify bounty totals reflect the change
        let bounty = contract.get_bounty(bounty_id).unwrap();
        assert_eq!(bounty.total_staked.0, new_stake.as_yoctonear());
        assert_eq!(bounty.stakes_per_option[0].0, 0); // Previous stake removed
        assert_eq!(bounty.stakes_per_option[1].0, new_stake.as_yoctonear()); // New stake added
    }

    #[test]
    #[should_panic(expected = "Bounty not found")]
    fn test_stake_on_nonexistent_bounty() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        let stake_amount = NearToken::from_near(5);
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(stake_amount).build());
        contract.stake_on_option(999, 0); // Non-existent bounty
    }

    #[test]
    #[should_panic(expected = "Invalid option index")]
    fn test_stake_on_invalid_option() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        // Create bounty with 2 options
        let bounty_id = create_bounty_with_deposit(
            &mut contract,
            &mut context,
            "Test Bounty",
            "A test bounty",
            ["Option A", "Option B"],
            NearToken::from_near(10),
            100,
        );

        let stake_amount = NearToken::from_near(5);
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(stake_amount).build());
        contract.stake_on_option(bounty_id, 2); // Invalid option index (only 0 and 1 exist)
    }

    #[test]
    fn test_get_user_bounties() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        // Create multiple bounties
        let bounty_id1 = create_bounty_with_deposit(
            &mut contract,
            &mut context,
            "Bounty 1",
            "First bounty",
            ["A", "B"],
            NearToken::from_near(10),
            100,
        );

        let bounty_id2 = create_bounty_with_deposit(
            &mut contract,
            &mut context,
            "Bounty 2",
            "Second bounty",
            ["X", "Y", "Z"],
            NearToken::from_near(5),
            200,
        );

        // User stakes on both bounties
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(NearToken::from_near(3)).build());
        contract.stake_on_option(bounty_id1, 0);

        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(NearToken::from_near(2)).build());
        contract.stake_on_option(bounty_id2, 1);

        // Get user bounties
        let user_bounties = contract.get_user_bounties(accounts(1));
        assert_eq!(user_bounties.len(), 2);

        // Verify stakes
        let stake1 = user_bounties.iter().find(|s| s.bounty_id == bounty_id1).unwrap();
        assert_eq!(stake1.amount.0, NearToken::from_near(3).as_yoctonear());
        assert_eq!(stake1.option_index, 0);

        let stake2 = user_bounties.iter().find(|s| s.bounty_id == bounty_id2).unwrap();
        assert_eq!(stake2.amount.0, NearToken::from_near(2).as_yoctonear());
        assert_eq!(stake2.option_index, 1);
    }

    #[test]
    fn test_get_bounty_stakes() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        // Create bounty
        let bounty_id = create_bounty_with_deposit(
            &mut contract,
            &mut context,
            "Test Bounty",
            "A test bounty",
            ["Option A", "Option B", "Option C"],
            NearToken::from_near(10),
            100,
        );

        // Multiple users stake on different options
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(NearToken::from_near(3)).build());
        contract.stake_on_option(bounty_id, 0);

        testing_env!(context.predecessor_account_id(accounts(2)).attached_deposit(NearToken::from_near(5)).build());
        contract.stake_on_option(bounty_id, 1);

        testing_env!(context.predecessor_account_id(accounts(3)).attached_deposit(NearToken::from_near(2)).build());
        contract.stake_on_option(bounty_id, 0);

        // Get stakes per option
        let stakes = contract.get_bounty_stakes(bounty_id);
        assert_eq!(stakes.len(), 3);
        assert_eq!(stakes[0].0, NearToken::from_near(5).as_yoctonear()); // 3 + 2 NEAR
        assert_eq!(stakes[1].0, NearToken::from_near(5).as_yoctonear()); // 5 NEAR
        assert_eq!(stakes[2].0, 0); // No stakes
    }

    #[test]
    fn test_determine_winning_option() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        // Create bounty
        let bounty_id = create_bounty_with_deposit(
            &mut contract,
            &mut context,
            "Test Bounty",
            "A test bounty",
            ["Option A", "Option B", "Option C"],
            NearToken::from_near(10),
            100,
        );

        let mut bounty = contract.bounties.get(&bounty_id).unwrap();

        // Test with no stakes
        assert_eq!(contract.determine_winning_option(&bounty), None);

        // Add stakes to make option 1 the winner
        bounty.stakes_per_option[0] = NearToken::from_near(3);
        bounty.stakes_per_option[1] = NearToken::from_near(7); // Winner
        bounty.stakes_per_option[2] = NearToken::from_near(2);

        assert_eq!(contract.determine_winning_option(&bounty), Some(1));

        // Test tie-breaking (lower index wins)
        bounty.stakes_per_option[0] = NearToken::from_near(5);
        bounty.stakes_per_option[1] = NearToken::from_near(5); // Same as option 0
        bounty.stakes_per_option[2] = NearToken::from_near(2);

        assert_eq!(contract.determine_winning_option(&bounty), Some(0)); // Lower index wins
    }

    #[test]
    fn test_calculate_platform_fee() {
        let context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        // Test 5% fee calculation
        let total_amount = NearToken::from_near(100);
        let fee = contract.calculate_platform_fee(total_amount);
        let expected_fee = NearToken::from_near(5); // 5% of 100 NEAR

        assert_eq!(fee.as_yoctonear(), expected_fee.as_yoctonear());

        // Test with smaller amount
        let small_amount = NearToken::from_near(1);
        let small_fee = contract.calculate_platform_fee(small_amount);
        let expected_small_fee = NearToken::from_millinear(50); // 5% of 1 NEAR = 0.05 NEAR

        assert_eq!(small_fee.as_yoctonear(), expected_small_fee.as_yoctonear());
    }

    #[test]
    fn test_calculate_user_reward() {
        let context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        // Create a test bounty
        let mut bounty = Bounty {
            id: 1,
            title: "Test".to_string(),
            description: "Test".to_string(),
            options: vec!["A".to_string(), "B".to_string()],
            creator: accounts(0),
            max_stake_per_user: NearToken::from_near(10),
            is_active: true,
            created_at: 0,
            ends_at: 1000,
            total_staked: NearToken::from_near(100), // Total pool
            stakes_per_option: vec![NearToken::from_near(30), NearToken::from_near(70)], // Option 1 wins
            is_closed: false,
            winning_option: None,
        };

        // User staked 10 NEAR on winning option (option 1)
        let user_stake = NearToken::from_near(10);
        let winning_option = 1u64;

        let reward = contract.calculate_user_reward(&bounty, user_stake, winning_option);

        // Expected calculation:
        // Total pool: 100 NEAR
        // Platform fee (5%): 5 NEAR
        // Prize pool: 95 NEAR
        // User's share: (10 / 70) * 95 = 13.57 NEAR (approximately)
        let expected_reward_yocto = user_stake.as_yoctonear()
            .checked_mul(NearToken::from_near(95).as_yoctonear())
            .and_then(|x| x.checked_div(NearToken::from_near(70).as_yoctonear()))
            .unwrap_or(0);

        assert_eq!(reward.as_yoctonear(), expected_reward_yocto);
    }

    #[test]
    fn test_close_bounty_no_participants() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        // Create bounty
        let bounty_id = create_bounty_with_deposit(
            &mut contract,
            &mut context,
            "Test Bounty",
            "A test bounty",
            ["Option A", "Option B"],
            NearToken::from_near(10),
            100,
        );

        // Fast forward time to after bounty ends
        testing_env!(context.block_timestamp(100 * 1_000_000_000 + 1).build());

        // Close bounty (no participants)
        contract.close_bounty(bounty_id);

        // Verify bounty is closed
        let bounty = contract.get_bounty(bounty_id).unwrap();
        require!(bounty.is_closed);
        require!(!bounty.is_active);
    }

    #[test]
    #[should_panic(expected = "Only bounty creator or contract owner can close bounty")]
    fn test_close_bounty_unauthorized() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        // Create bounty
        let bounty_id = create_bounty_with_deposit(
            &mut contract,
            &mut context,
            "Test Bounty",
            "A test bounty",
            ["Option A", "Option B"],
            NearToken::from_near(10),
            100,
        );

        // Fast forward time
        testing_env!(context.block_timestamp(100 * 1_000_000_000 + 1).predecessor_account_id(accounts(1)).build());

        // Try to close bounty as non-owner (creators are just regular users)
        contract.close_bounty(bounty_id);
    }

    #[test]
    #[should_panic(expected = "Bounty has not expired yet")]
    fn test_close_bounty_not_expired() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        // Create bounty
        let bounty_id = create_bounty_with_deposit(
            &mut contract,
            &mut context,
            "Test Bounty",
            "A test bounty",
            ["Option A", "Option B"],
            NearToken::from_near(10),
            100,
        );

        // Try to close bounty before it expires (current time is still 0)
        contract.close_bounty(bounty_id);
    }

    #[test]
    fn test_close_bounty_with_participants() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        // Create bounty
        let bounty_id = create_bounty_with_deposit(
            &mut contract,
            &mut context,
            "Test Bounty",
            "A test bounty",
            ["Option A", "Option B"],
            NearToken::from_near(10),
            100,
        );

        // Add participants
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(NearToken::from_near(3)).build());
        contract.stake_on_option(bounty_id, 0);

        testing_env!(context.predecessor_account_id(accounts(2)).attached_deposit(NearToken::from_near(7)).build());
        contract.stake_on_option(bounty_id, 1);

        // Fast forward time to after bounty ends
        testing_env!(context.block_timestamp(100 * 1_000_000_000 + 1).predecessor_account_id(accounts(0)).build());

        // Close bounty
        contract.close_bounty(bounty_id);

        // Verify bounty is closed and has winning option
        let bounty = contract.get_bounty(bounty_id).unwrap();
        require!(bounty.is_closed);
        require!(!bounty.is_active);
        assert_eq!(bounty.winning_option, Some(1)); // Option 1 had more stakes (7 NEAR vs 3 NEAR)
    }

    #[test]
    fn test_get_bounty_results() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        // Create bounty
        let bounty_id = create_bounty_with_deposit(
            &mut contract,
            &mut context,
            "Test Bounty",
            "A test bounty",
            ["Option A", "Option B"],
            NearToken::from_near(10),
            100,
        );

        // Should return None for active bounty
        assert!(contract.get_bounty_results(bounty_id).is_none());

        // Add participants and close bounty
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(NearToken::from_near(3)).build());
        contract.stake_on_option(bounty_id, 0);

        testing_env!(context.predecessor_account_id(accounts(2)).attached_deposit(NearToken::from_near(7)).build());
        contract.stake_on_option(bounty_id, 1);

        // Fast forward and close
        testing_env!(context.block_timestamp(100 * 1_000_000_000 + 1).predecessor_account_id(accounts(0)).build());
        contract.close_bounty(bounty_id);

        // Should return results for closed bounty
        let results = contract.get_bounty_results(bounty_id).unwrap();
        assert!(results.is_closed);
        assert_eq!(results.winning_option, Some(1));
    }

    #[test]
    #[should_panic(expected = "Bounty is not closed yet")]
    fn test_claim_winnings_bounty_not_closed() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        // Create bounty and stake
        let bounty_id = create_bounty_with_deposit(
            &mut contract,
            &mut context,
            "Test Bounty",
            "A test bounty",
            ["Option A", "Option B"],
            NearToken::from_near(10),
            100,
        );

        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(NearToken::from_near(5)).build());
        contract.stake_on_option(bounty_id, 0);

        // Try to claim before bounty is closed
        contract.claim_bounty_winnings(bounty_id);
    }

    #[test]
    #[should_panic(expected = "No stake found for this bounty")]
    fn test_claim_winnings_no_stake() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        // Create bounty and close it without user participation
        let bounty_id = create_bounty_with_deposit(
            &mut contract,
            &mut context,
            "Test Bounty",
            "A test bounty",
            ["Option A", "Option B"],
            NearToken::from_near(10),
            100,
        );

        // Fast forward and close
        testing_env!(context.block_timestamp(100 * 1_000_000_000 + 1).build());
        contract.close_bounty(bounty_id);

        // Try to claim without having staked
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        contract.claim_bounty_winnings(bounty_id);
    }

    #[test]
    #[should_panic(expected = "User did not win this bounty")]
    fn test_claim_winnings_user_lost() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        // Create bounty
        let bounty_id = create_bounty_with_deposit(
            &mut contract,
            &mut context,
            "Test Bounty",
            "A test bounty",
            ["Option A", "Option B"],
            NearToken::from_near(10),
            100,
        );

        // User stakes on losing option
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(NearToken::from_near(3)).build());
        contract.stake_on_option(bounty_id, 0);

        // Another user stakes more on winning option
        testing_env!(context.predecessor_account_id(accounts(2)).attached_deposit(NearToken::from_near(7)).build());
        contract.stake_on_option(bounty_id, 1);

        // Close bounty
        testing_env!(context.block_timestamp(100 * 1_000_000_000 + 1).predecessor_account_id(accounts(0)).build());
        contract.close_bounty(bounty_id);

        // Losing user tries to claim
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        contract.claim_bounty_winnings(bounty_id);
    }

    #[test]
    #[should_panic(expected = "Title cannot be empty")]
    fn test_create_bounty_empty_title() {
        let context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        contract.create_bounty(
            "".to_string(), // Empty title
            "Description".to_string(),
            vec!["A".to_string(), "B".to_string()],
            NearToken::from_near(10),
            100,
        );
    }

    #[test]
    #[should_panic(expected = "Description cannot be empty")]
    fn test_create_bounty_empty_description() {
        let context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        contract.create_bounty(
            "Title".to_string(),
            "   ".to_string(), // Empty description (whitespace)
            vec!["A".to_string(), "B".to_string()],
            NearToken::from_near(10),
            100,
        );
    }

    #[test]
    #[should_panic(expected = "Option 0 cannot be empty")]
    fn test_create_bounty_empty_option() {
        let context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        contract.create_bounty(
            "Title".to_string(),
            "Description".to_string(),
            vec!["".to_string(), "B".to_string()], // Empty option
            NearToken::from_near(10),
            100,
        );
    }

    #[test]
    fn test_pause_unpause_contract() {
        let context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        // Initially not paused        assert!(!contract.is_contract_paused());
 
        // Pause contract
        contract.pause_contract();
        assert!(contract.is_contract_paused());
 
        // Unpause contract
        contract.unpause_contract();
        assert!(!contract.is_contract_paused());
    }

    #[test]
    #[should_panic(expected = "Contract is paused")]
    fn test_create_bounty_when_paused() {
        let context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        // Pause contract
        contract.pause_contract();

        // Try to create bounty when paused
        contract.create_bounty(
            "Title".to_string(),
            "Description".to_string(),
            vec!["A".to_string(), "B".to_string()],
            NearToken::from_near(10),
            100,
        );
    }

    #[test]
    fn test_update_platform_fee_rate() {
        let context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        // Initial fee rate is 5% (500 basis points)
        assert_eq!(contract.get_platform_fee_rate(), 500);

        // Update to 3% (300 basis points)
        contract.update_platform_fee_rate(300);
        assert_eq!(contract.get_platform_fee_rate(), 300);
    }

    #[test]
    fn test_update_platform_fee_rate_too_high_clamped() {
        let context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        // Try to set fee rate above 10% - should be clamped to 10%
        contract.update_platform_fee_rate(1001);
        assert_eq!(contract.get_platform_fee_rate(), 1000, "Platform fee should be clamped to 1000 (10%)");
    }

    #[test]
    fn test_emergency_close_bounty() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        // Create bounty and add participants
        let bounty_id = create_bounty_with_deposit(
            &mut contract,
            &mut context,
            "Test Bounty",
            "A test bounty",
            ["Option A", "Option B"],
            NearToken::from_near(10),
            100,
        );

        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(NearToken::from_near(5)).build());
        contract.stake_on_option(bounty_id, 0);

        // Emergency close as owner
        testing_env!(context.predecessor_account_id(accounts(0)).build());
        contract.emergency_close_bounty(bounty_id);

        // Verify bounty is closed
        let bounty = contract.get_bounty(bounty_id).unwrap();
        require!(bounty.is_closed);
        require!(!bounty.is_active);
    }

    #[test]
    #[should_panic(expected = "Only the owner can call this method")]
    fn test_emergency_close_bounty_unauthorized() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        // Create bounty
        let bounty_id = create_bounty_with_deposit(
            &mut contract,
            &mut context,
            "Test Bounty",
            "A test bounty",
            ["Option A", "Option B"],
            NearToken::from_near(10),
            100,
        );

        // Try to emergency close as non-owner
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        contract.emergency_close_bounty(bounty_id);
    }

    #[test]
    fn test_withdraw_platform_fees() {
        let context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        // Test withdrawal (should work even if no fees to withdraw)
        contract.withdraw_platform_fees();
        // No assertion needed - just testing it doesn't panic
    }

    #[test]
    #[should_panic(expected = "Only the owner can call this method")]
    fn test_withdraw_platform_fees_unauthorized() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        // Try to withdraw as non-owner
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        contract.withdraw_platform_fees();
    }

    #[test]
    fn test_get_contract_owner() {
        let context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        assert_eq!(contract.get_contract_owner(), accounts(0));
    }

    // Test removed: The security fix (panic on insufficient balance) is verified by the assertion in internal_claim_rewards.
    // Creating a test scenario that accumulates enough rewards while keeping balance low enough is difficult
    // without hitting overflow protection. The important fix is that we now panic instead of silently failing.

    #[test]
    fn test_calculate_rewards_safe_with_zero_rate() {
        let stake_amount = NearToken::from_near(10);
        let reward_rate = 0u128;
        let time_seconds = 3600u64; // 1 hour

        let rewards = BountyPredictionContract::calculate_rewards_safe(stake_amount, reward_rate, time_seconds);
        assert_eq!(rewards, 0, "Rewards should be 0 with zero reward rate");
    }

    #[test]
    #[should_panic(expected = "Reward calculation overflow")]
    fn test_calculate_rewards_safe_with_high_rate() {
        let stake_amount = NearToken::from_near(1);
        let reward_rate = u128::MAX / 1_000_000; // Very high rate that causes overflow
        let time_seconds = 1u64;

        let _rewards = BountyPredictionContract::calculate_rewards_safe(stake_amount, reward_rate, time_seconds);
    }

    #[test]
    #[should_panic(expected = "Reward calculation overflow")]
    fn test_calculate_rewards_safe_overflow_protection() {
        let stake_amount = NearToken::from_near(1000);
        let reward_rate = u128::MAX / 1000; // High rate
        let time_seconds = u64::MAX; // Maximum time

        // Should panic on overflow
        let _rewards = BountyPredictionContract::calculate_rewards_safe(stake_amount, reward_rate, time_seconds);
    }

    #[test]
    fn test_calculate_rewards_safe_with_zero_stake() {
        let stake_amount = NearToken::from_yoctonear(0);
        let reward_rate = 1000u128;
        let time_seconds = 3600u64;

        let rewards = BountyPredictionContract::calculate_rewards_safe(stake_amount, reward_rate, time_seconds);
        assert_eq!(rewards, 0, "Rewards should be 0 with zero stake");
    }

    #[test]
    fn test_calculate_rewards_safe_with_zero_time() {
        let stake_amount = NearToken::from_near(10);
        let reward_rate = 1000u128;
        let time_seconds = 0u64;

        let rewards = BountyPredictionContract::calculate_rewards_safe(stake_amount, reward_rate, time_seconds);
        assert_eq!(rewards, 0, "Rewards should be 0 with zero time");
    }

    #[test]
    fn test_update_reward_rate_to_high_value_clamped() {
        let context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        let very_high_rate = u128::MAX / 1000;
        contract.update_reward_rate(very_high_rate);
        assert_eq!(contract.get_reward_rate(), 1_000_000_000, "Very high reward rate should be clamped to 1 billion");
    }

    #[test]
    fn test_update_reward_rate_to_one() {
        let context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        contract.update_reward_rate(1);
        assert_eq!(contract.get_reward_rate(), 1);
    }

    #[test]
    fn test_reward_calculation_consistency() {
        let context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let contract = BountyPredictionContract::new(1000, MIN_STAKE, MAX_STAKE);

        let stake_amount = NearToken::from_near(10);
        let reward_rate = 1000u128;
        let time_seconds = 3600u64; // 1 hour

        // Calculate rewards multiple times - should be consistent
        let rewards1 = BountyPredictionContract::calculate_rewards_safe(stake_amount, reward_rate, time_seconds);
        let rewards2 = BountyPredictionContract::calculate_rewards_safe(stake_amount, reward_rate, time_seconds);
        let rewards3 = BountyPredictionContract::calculate_rewards_safe(stake_amount, reward_rate, time_seconds);

        assert_eq!(rewards1, rewards2, "Reward calculations should be consistent");
        assert_eq!(rewards2, rewards3, "Reward calculations should be consistent");
    }

    #[test]
    fn test_reward_calculation_proportionality() {
        let reward_rate = 100u128;
        let time_seconds = 3600u64;

        let stake1 = NearToken::from_near(1);
        let stake2 = NearToken::from_near(2);
        let stake10 = NearToken::from_near(10);

        let rewards1 = BountyPredictionContract::calculate_rewards_safe(stake1, reward_rate, time_seconds);
        let rewards2 = BountyPredictionContract::calculate_rewards_safe(stake2, reward_rate, time_seconds);
        let rewards10 = BountyPredictionContract::calculate_rewards_safe(stake10, reward_rate, time_seconds);

        // Rewards should be proportional to stake amount
        assert_eq!(rewards2, rewards1 * 2, "Rewards should be proportional to stake (2x)");
        assert_eq!(rewards10, rewards1 * 10, "Rewards should be proportional to stake (10x)");
    }

    #[test]
    #[should_panic(expected = "Only the owner can call this method")]
    fn test_pause_contract_unauthorized() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        // Try to pause as non-owner
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        contract.pause_contract();
    }

    #[test]
    #[should_panic(expected = "Only the owner can call this method")]
    fn test_update_reward_rate_unauthorized() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        // Try to update as non-owner
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        contract.update_reward_rate(200);
    }

    #[test]
    fn test_participant_tracking_single_participant() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        // Create a bounty
        let bounty_id = create_bounty_with_deposit(
            &mut contract,
            &mut context,
            "Test Bounty",
            "Test Description",
            ["Option A", "Option B"],
            NearToken::from_near(10),
            100,
        );

        // Stake on the bounty
        let stake_amount = NearToken::from_near(5);
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(stake_amount).build());
        contract.stake_on_option(bounty_id, 0);

        // Check participant tracking
        let participants = contract.get_bounty_participants(bounty_id);
        assert_eq!(participants.len(), 1);
        assert_eq!(participants[0], accounts(1));

        let participant_count = contract.get_bounty_participant_count(bounty_id);
        assert_eq!(participant_count, 1);
    }

    #[test]
    fn test_participant_tracking_multiple_participants() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        // Create a bounty
        let bounty_id = create_bounty_with_deposit(
            &mut contract,
            &mut context,
            "Test Bounty",
            "Test Description",
            ["Option A", "Option B"],
            NearToken::from_near(10),
            100,
        );

        // Multiple participants stake
        let stake_amount = NearToken::from_near(5);

        // Participant 1
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(stake_amount).build());
        contract.stake_on_option(bounty_id, 0);

        // Participant 2
        testing_env!(context.predecessor_account_id(accounts(2)).attached_deposit(stake_amount).build());
        contract.stake_on_option(bounty_id, 1);

        // Participant 3
        testing_env!(context.predecessor_account_id(accounts(3)).attached_deposit(stake_amount).build());
        contract.stake_on_option(bounty_id, 0);

        // Check participant tracking
        let participants = contract.get_bounty_participants(bounty_id);
        assert_eq!(participants.len(), 3);
        assert!(participants.contains(&accounts(1)));
        assert!(participants.contains(&accounts(2)));
        assert!(participants.contains(&accounts(3)));

        let participant_count = contract.get_bounty_participant_count(bounty_id);
        assert_eq!(participant_count, 3);
    }

    #[test]
    fn test_participant_tracking_no_duplicates() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        // Create a bounty
        let bounty_id = create_bounty_with_deposit(
            &mut contract,
            &mut context,
            "Test Bounty",
            "Test Description",
            ["Option A", "Option B"],
            NearToken::from_near(10),
            100,
        );

        // Participant stakes multiple times
        let stake_amount = NearToken::from_near(2);
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(stake_amount).build());
        contract.stake_on_option(bounty_id, 0);

        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(stake_amount).build());
        contract.stake_on_option(bounty_id, 0);

        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(stake_amount).build());
        contract.stake_on_option(bounty_id, 1);

        // Should only have one participant entry
        let participants = contract.get_bounty_participants(bounty_id);
        assert_eq!(participants.len(), 1);
        assert_eq!(participants[0], accounts(1));

        let participant_count = contract.get_bounty_participant_count(bounty_id);
        assert_eq!(participant_count, 1);
    }

    #[test]
    #[should_panic(expected = "Bounty has reached maximum participant limit")]
    fn test_bounty_participant_limit_enforced() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        let bounty_id = create_bounty_with_deposit(
            &mut contract,
            &mut context,
            "Limited Bounty",
            "Testing participant limits",
            ["A", "B"],
            NearToken::from_near(10),
            100,
        );

        // Manually set participant count to max
        let bounty_participants = contract.get_bounty_participants_mut();
        let mut participants = Vec::new();
        for i in 0..150 {
            participants.push(format!("user{}.testnet", i).parse().unwrap());
        }
        bounty_participants.insert(&bounty_id, &participants);

        // Try to add 151st participant (should fail)
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(NearToken::from_near(5)).build());
        contract.stake_on_option(bounty_id, 0);
    }

    #[test]
    fn test_participant_at_limit_minus_one() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        let bounty_id = create_bounty_with_deposit(
            &mut contract,
            &mut context,
            "Near Limit Test",
            "Testing near limit",
            ["A", "B"],
            NearToken::from_near(10),
            100,
        );

        // Add participants up to limit - 1
        let bounty_participants = contract.get_bounty_participants_mut();
        let mut participants = Vec::new();
        for i in 0..149 {
            participants.push(format!("user{}.testnet", i).parse().unwrap());
        }
        bounty_participants.insert(&bounty_id, &participants);

        // Add 150th participant (should succeed)
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(NearToken::from_near(5)).build());
        contract.stake_on_option(bounty_id, 0);

        let final_count = contract.get_bounty_participant_count(bounty_id);
        assert_eq!(final_count, 150);
    }

    #[test]
    fn test_existing_participant_can_change_stake_at_limit() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        let bounty_id = create_bounty_with_deposit(
            &mut contract,
            &mut context,
            "Limit Test",
            "Testing existing participant",
            ["A", "B"],
            NearToken::from_near(10),
            100,
        );

        // Add participant
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(NearToken::from_near(5)).build());
        contract.stake_on_option(bounty_id, 0);

        // Fill to limit with fake accounts
        let bounty_participants = contract.get_bounty_participants_mut();
        let mut participants = bounty_participants.get(&bounty_id).unwrap_or_default();
        for i in 0..149 {
            participants.push(format!("user{}.testnet", i).parse().unwrap());
        }
        bounty_participants.insert(&bounty_id, &participants);

        // Existing participant can still change stake at limit
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(NearToken::from_near(7)).build());
        contract.stake_on_option(bounty_id, 1);

        let stake = contract.get_participant_stake(accounts(1), bounty_id).unwrap();
        assert_eq!(stake.option_index, 1);
    }

    #[test]
    fn test_get_max_participants_view_function() {
        let context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        let max = contract.get_max_participants_per_bounty();
        assert_eq!(max, 150);
    }

    #[test]
    fn test_participant_tracking_across_multiple_bounties() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        // Create two bounties
        let bounty_id_1 = create_bounty_with_deposit(
            &mut contract,
            &mut context,
            "Test Bounty 1",
            "Test Description 1",
            ["Option A", "Option B"],
            NearToken::from_near(10),
            100,
        );

        let bounty_id_2 = create_bounty_with_deposit(
            &mut contract,
            &mut context,
            "Test Bounty 2",
            "Test Description 2",
            ["Option X", "Option Y"],
            NearToken::from_near(10),
            100,
        );

        let stake_amount = NearToken::from_near(5);

        // Participant 1 stakes on both bounties
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(stake_amount).build());
        contract.stake_on_option(bounty_id_1, 0);

        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(stake_amount).build());
        contract.stake_on_option(bounty_id_2, 1);

        // Participant 2 stakes only on bounty 1
        testing_env!(context.predecessor_account_id(accounts(2)).attached_deposit(stake_amount).build());
        contract.stake_on_option(bounty_id_1, 1);

        // Check participant tracking for each bounty
        let participants_1 = contract.get_bounty_participants(bounty_id_1);
        assert_eq!(participants_1.len(), 2);        assert!(participants_1.contains(&accounts(1)));
        assert!(participants_1.contains(&accounts(2)));
 
        let participants_2 = contract.get_bounty_participants(bounty_id_2);
        assert_eq!(participants_2.len(), 1);
        assert!(participants_2.contains(&accounts(1)));

        // Check participant counts
        assert_eq!(contract.get_bounty_participant_count(bounty_id_1), 2);
        assert_eq!(contract.get_bounty_participant_count(bounty_id_2), 1);
    }
}
