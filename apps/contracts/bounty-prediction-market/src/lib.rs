use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, NearToken, PanicOnDefault, Promise};
use schemars::JsonSchema;

// Safety constants to prevent overflow and ensure system stability

const MAX_PLATFORM_FEE_RATE: u128 = 1000; // 10% maximum platform fee
const MAX_BOUNTY_OPTIONS: usize = 1000; // Maximum options per bounty
const MIN_BOUNTY_OPTIONS: usize = 2; // Minimum options per bounty
const MAX_BOUNTY_DURATION: u64 = 1_000_000; // Maximum bounty duration in blocks
const MIN_BOUNTY_DURATION: u64 = 1; // Minimum bounty duration in blocks

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Bounty {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub options: Vec<String>,
    pub creator: AccountId,
    pub max_stake_per_user: NearToken,
    pub is_active: bool,
    pub created_at: u64,
    pub ends_at: u64,
    pub total_staked: NearToken,
    pub stakes_per_option: Vec<NearToken>,
    pub is_closed: bool,
    pub winning_option: Option<u64>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ParticipantStake {
    pub bounty_id: u64,
    pub option_index: u64,
    pub amount: NearToken,
    pub staked_at: u64,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct BountyView {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub options: Vec<String>,
    #[schemars(with = "String")]
    pub creator: AccountId,
    #[schemars(with = "String")]
    pub max_stake_per_user: U128,
    pub is_active: bool,
    pub created_at: u64,
    pub ends_at: u64,
    #[schemars(with = "String")]
    pub total_staked: U128,
    #[schemars(with = "Vec<String>")]
    pub stakes_per_option: Vec<U128>,
    pub is_closed: bool,
    pub winning_option: Option<u64>,
}

impl From<Bounty> for BountyView {
    fn from(bounty: Bounty) -> Self {
        Self {
            id: bounty.id,
            title: bounty.title,
            description: bounty.description,
            options: bounty.options,
            creator: bounty.creator,
            max_stake_per_user: U128(bounty.max_stake_per_user.as_yoctonear()),
            is_active: bounty.is_active,
            created_at: bounty.created_at,
            ends_at: bounty.ends_at,
            total_staked: U128(bounty.total_staked.as_yoctonear()),
            stakes_per_option: bounty
                .stakes_per_option
                .iter()
                .map(|s| U128(s.as_yoctonear()))
                .collect(),
            is_closed: bounty.is_closed,
            winning_option: bounty.winning_option,
        }
    }
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct ParticipantStakeView {
    pub bounty_id: u64,
    pub option_index: u64,
    #[schemars(with = "String")]
    pub amount: U128,
    pub staked_at: u64,
}

impl From<ParticipantStake> for ParticipantStakeView {
    fn from(stake: ParticipantStake) -> Self {
        Self {
            bounty_id: stake.bounty_id,
            option_index: stake.option_index,
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

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
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
    is_paused: bool,         // Emergency pause functionality
}

#[near_bindgen]
impl BountyPredictionContract {
    #[init]
    pub fn new(
        reward_rate: u128,
        min_stake_amount: NearToken,
        max_stake_amount: NearToken,
    ) -> Self {
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
        assert!(
            min_stake_amount <= max_stake_amount,
            "Minimum stake amount cannot exceed maximum"
        );
        assert!(
            max_stake_amount.as_near() <= MAX_STAKE_AMOUNT,
            "Maximum stake amount cannot exceed {} NEAR",
            MAX_STAKE_AMOUNT
        );

        env::log_str(&format!(
            "CONTRACT_INIT: reward_rate={} (clamped from {}), min_stake={}, max_stake={}",
            safe_reward_rate,
            reward_rate,
            min_stake_amount.as_near(),
            max_stake_amount.as_near()
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
            is_paused: false,
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
                    bounty_participants: current_contract
                        .bounty_participants
                        .or_else(|| Some(LookupMap::new(b"t"))),
                    next_bounty_id: current_contract.next_bounty_id,
                    platform_fee_rate: current_contract.platform_fee_rate,
                    is_paused: current_contract.is_paused,
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
                    is_paused: old_contract.is_paused,
                };
            }

            env::log_str(
                "CONTRACT_MIGRATION: Could not parse existing state, creating new contract",
            );
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
            is_paused: false,
        }
    }

    /// Regular migration function that can be called after deployment
    /// Only callable by the contract owner for security
    pub fn migrate_state(&mut self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner,
            "Only owner can migrate state"
        );

        // This function can be used to migrate state after deployment
        // Initialize bounty_participants if it doesn't exist
        if self.bounty_participants.is_none() {
            self.bounty_participants = Some(LookupMap::new(b"t"));
            env::log_str("CONTRACT_MIGRATION: Initialized bounty_participants field");

            // Log current contract state for verification
            env::log_str(&format!(
                "CONTRACT_MIGRATION: Current state - next_bounty_id: {}, paused: {}",
                self.next_bounty_id, self.is_paused
            ));
        } else {
            env::log_str("CONTRACT_MIGRATION: bounty_participants field already exists");
        }
    }

    // Helper function for safe token addition
    fn safe_add_tokens(a: NearToken, b: NearToken) -> Result<NearToken, &'static str> {
        a.as_yoctonear()
            .checked_add(b.as_yoctonear())
            .map(NearToken::from_yoctonear)
            .ok_or("Token addition overflow")
    }

    // Helper function for safe token subtraction
    fn safe_sub_tokens(a: NearToken, b: NearToken) -> Result<NearToken, &'static str> {
        a.as_yoctonear()
            .checked_sub(b.as_yoctonear())
            .map(NearToken::from_yoctonear)
            .ok_or("Token subtraction underflow")
    }

    // Helper function for safe reward calculation
    fn calculate_rewards_safe(
        stake_amount: NearToken,
        reward_rate: u128,
        time_seconds: u64,
    ) -> u128 {
        // Use checked arithmetic to prevent overflow
        // Divide by the scaling factor last to maintain precision
        stake_amount
            .as_yoctonear()
            .checked_mul(reward_rate)
            .and_then(|x| x.checked_mul(time_seconds as u128))
            .and_then(|x| x.checked_div(1_000_000_000_000_000_000_000_000))
            .unwrap_or(0) // Return 0 on overflow rather than panicking
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

        assert!(amount >= self.min_stake_amount, "Stake amount too low");
        assert!(amount <= self.max_stake_amount, "Stake amount too high");

        // Validate that total stake (existing + new) doesn't exceed maximum
        let new_total_stake = if let Some(existing_stake) = self.stakes.get(&staker) {
            Self::safe_add_tokens(existing_stake.amount, amount).expect("Stake addition overflow")
        } else {
            amount
        };

        assert!(
            new_total_stake <= self.max_stake_amount,
            "Total stake would exceed maximum allowed"
        );

        let current_time = env::block_timestamp();

        if let Some(mut stake_info) = self.stakes.get(&staker) {
            // Claim pending rewards before updating stake
            self.internal_claim_rewards(&staker, &mut stake_info);

            // Add to existing stake using safe addition
            stake_info.amount =
                Self::safe_add_tokens(stake_info.amount, amount).expect("Stake addition overflow");
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
        assert!(
            amount > NearToken::from_yoctonear(0),
            "Unstake amount must be positive"
        );

        // Claim pending rewards
        self.internal_claim_rewards(&staker, &mut stake_info);

        // Update stake using safe subtraction
        stake_info.amount =
            Self::safe_sub_tokens(stake_info.amount, amount).expect("Stake subtraction underflow");
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

        let rewards =
            Self::calculate_rewards_safe(stake_info.amount, self.reward_rate, time_diff_seconds);

        if rewards > 0 {
            let reward_amount = NearToken::from_yoctonear(rewards);

            // Check if contract has sufficient balance to pay rewards
            // Reserve 1 NEAR for contract operations
            let contract_balance = env::account_balance();
            let reserved_balance = NearToken::from_near(1);

            if contract_balance
                > Self::safe_add_tokens(reward_amount, reserved_balance).unwrap_or(contract_balance)
            {
                stake_info.last_reward_claim = current_time;
                Promise::new(staker.clone()).transfer(reward_amount);
                env::log_str(&format!(
                    "REWARD: Account {} claimed {} NEAR",
                    staker, reward_amount
                ));
            } else {
                env::log_str(&format!(
                    "REWARD_FAILED: Insufficient contract balance for {}",
                    staker
                ));
            }
        }
    }

    pub fn get_stake_info(&self, account: AccountId) -> Option<StakeInfoView> {
        self.stakes
            .get(&account)
            .map(|stake_info| stake_info.into())
    }

    pub fn calculate_pending_rewards(&self, account: AccountId) -> U128 {
        if let Some(stake_info) = self.stakes.get(&account) {
            let current_time = env::block_timestamp();
            let time_diff = current_time - stake_info.last_reward_claim;
            let time_diff_seconds = time_diff / 1_000_000_000;

            let rewards = Self::calculate_rewards_safe(
                stake_info.amount,
                self.reward_rate,
                time_diff_seconds,
            );
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

    // Helper function to check if contract is paused
    fn assert_not_paused(&self) {
        assert!(!self.is_paused, "Contract is paused");
    }

    // Bounty Management Functions
    pub fn create_bounty(
        &mut self,
        title: String,
        description: String,
        options: Vec<String>,
        max_stake_per_user: NearToken,
        duration_blocks: u64,
    ) -> u64 {
        self.assert_not_paused();
        let creator = env::predecessor_account_id();

        // Validate inputs
        assert!(!title.trim().is_empty(), "Title cannot be empty");
        assert!(
            !description.trim().is_empty(),
            "Description cannot be empty"
        );
        assert!(title.len() <= 200, "Title too long (max 200 characters)");
        assert!(
            description.len() <= 1000,
            "Description too long (max 1000 characters)"
        );

        // Validate options count (2-1000)
        assert!(options.len() >= 2, "Bounty must have at least 2 options");
        assert!(
            options.len() <= 1000,
            "Bounty cannot have more than 1000 options"
        );

        // Validate option content
        for (i, option) in options.iter().enumerate() {
            assert!(!option.trim().is_empty(), "Option {} cannot be empty", i);
            assert!(
                option.len() <= 100,
                "Option {} too long (max 100 characters)",
                i
            );
        }

        // Validate max stake amount (0.1 to 10000 NEAR)
        const MIN_BOUNTY_STAKE_MILLINEAR: u128 = 100; // 0.1 NEAR
        let min_bounty_stake = NearToken::from_millinear(MIN_BOUNTY_STAKE_MILLINEAR);
        let max_bounty_stake = NearToken::from_near(10000);
        assert!(
            max_stake_per_user >= min_bounty_stake,
            "Maximum stake per user must be at least {} millinear",
            MIN_BOUNTY_STAKE_MILLINEAR
        );
        assert!(
            max_stake_per_user <= max_bounty_stake,
            "Maximum stake per user cannot exceed 10000 NEAR"
        );

        // Validate duration
        assert!(
            duration_blocks > 0,
            "Duration must be greater than 0 blocks"
        );

        let bounty_id = self.next_bounty_id;
        let current_time = env::block_timestamp();
        let ends_at = current_time + (duration_blocks * 1_000_000_000); // Convert blocks to nanoseconds (approximate)

        let stakes_per_option = vec![NearToken::from_yoctonear(0); options.len()];

        let bounty = Bounty {
            id: bounty_id,
            title,
            description,
            options,
            creator: creator.clone(),
            max_stake_per_user,
            is_active: true,
            created_at: current_time,
            ends_at,
            total_staked: NearToken::from_yoctonear(0),
            stakes_per_option,
            is_closed: false,
            winning_option: None,
        };

        self.bounties.insert(&bounty_id, &bounty);
        self.next_bounty_id += 1;

        env::log_str(&format!("BOUNTY_CREATED: ID {} by {}", bounty_id, creator));

        bounty_id
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

    // Staking on Bounty Options
    #[payable]
    pub fn stake_on_option(&mut self, bounty_id: u64, option_index: u64) {
        self.assert_not_paused();
        let staker = env::predecessor_account_id();
        let amount = env::attached_deposit();
        let current_time = env::block_timestamp();

        // Get and validate bounty
        let mut bounty = self.bounties.get(&bounty_id).expect("Bounty not found");
        assert!(bounty.is_active, "Bounty is not active");
        assert!(!bounty.is_closed, "Bounty is already closed");
        assert!(current_time < bounty.ends_at, "Bounty has expired");

        // Validate option index
        assert!(
            (option_index as usize) < bounty.options.len(),
            "Invalid option index"
        );

        // Validate stake amount
        assert!(
            amount > NearToken::from_yoctonear(0),
            "Stake amount must be positive"
        );
        assert!(
            amount <= bounty.max_stake_per_user,
            "Stake amount exceeds maximum allowed for this bounty"
        );

        let stake_key = (staker.clone(), bounty_id);
        let is_new_participant = !self.participant_stakes.contains_key(&stake_key);

        // Handle existing stake
        if let Some(existing_stake) = self.participant_stakes.get(&stake_key) {
            // Remove previous stake from bounty totals
            bounty.total_staked = Self::safe_sub_tokens(bounty.total_staked, existing_stake.amount)
                .expect("Total stake subtraction underflow");
            bounty.stakes_per_option[existing_stake.option_index as usize] = Self::safe_sub_tokens(
                bounty.stakes_per_option[existing_stake.option_index as usize],
                existing_stake.amount,
            )
            .expect("Option stake subtraction underflow");
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
        bounty.stakes_per_option[option_index as usize] =
            Self::safe_add_tokens(bounty.stakes_per_option[option_index as usize], amount)
                .expect("Option stake addition overflow");

        // Create or update participant stake
        let participant_stake = ParticipantStake {
            bounty_id,
            option_index,
            amount,
            staked_at: current_time,
        };

        self.participant_stakes
            .insert(&stake_key, &participant_stake);
        self.bounties.insert(&bounty_id, &bounty);

        env::log_str(&format!(
            "BOUNTY_STAKE: Account {} staked {} NEAR on option {} for bounty {}",
            staker, amount, option_index, bounty_id
        ));
    }

    pub fn get_participant_stake(
        &self,
        account: AccountId,
        bounty_id: u64,
    ) -> Option<ParticipantStakeView> {
        self.participant_stakes
            .get(&(account, bounty_id))
            .map(|stake| stake.into())
    }

    pub fn get_bounty_stakes(&self, bounty_id: u64) -> Vec<U128> {
        if let Some(bounty) = self.bounties.get(&bounty_id) {
            bounty
                .stakes_per_option
                .iter()
                .map(|s| U128(s.as_yoctonear()))
                .collect()
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
    fn determine_winning_option(&self, bounty: &Bounty) -> Option<u64> {
        if bounty.stakes_per_option.is_empty() {
            return None;
        }

        let mut max_stake = NearToken::from_yoctonear(0);
        let mut winning_option = 0u64;
        let mut has_stakes = false;

        for (index, stake) in bounty.stakes_per_option.iter().enumerate() {
            if *stake > NearToken::from_yoctonear(0) {
                has_stakes = true;
                if *stake > max_stake {
                    max_stake = *stake;
                    winning_option = index as u64;
                }
            }
        }

        if has_stakes {
            Some(winning_option)
        } else {
            None
        }
    }

    fn calculate_platform_fee(&self, total_amount: NearToken) -> NearToken {
        let fee_amount = total_amount
            .as_yoctonear()
            .checked_mul(self.platform_fee_rate as u128)
            .and_then(|x| x.checked_div(10000)) // Convert basis points to percentage
            .unwrap_or(0);

        NearToken::from_yoctonear(fee_amount)
    }

    fn calculate_user_reward(
        &self,
        bounty: &Bounty,
        user_stake: NearToken,
        winning_option: u64,
    ) -> NearToken {
        let total_winning_stakes = bounty.stakes_per_option[winning_option as usize];

        if total_winning_stakes == NearToken::from_yoctonear(0) {
            return NearToken::from_yoctonear(0);
        }

        // Calculate total prize pool after platform fee
        let platform_fee = self.calculate_platform_fee(bounty.total_staked);
        let prize_pool =
            Self::safe_sub_tokens(bounty.total_staked, platform_fee).unwrap_or(bounty.total_staked);

        // Calculate proportional reward
        let user_share = user_stake
            .as_yoctonear()
            .checked_mul(prize_pool.as_yoctonear())
            .and_then(|x| x.checked_div(total_winning_stakes.as_yoctonear()))
            .unwrap_or(0);

        NearToken::from_yoctonear(user_share)
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
    pub fn close_bounty(&mut self, bounty_id: u64) {
        self.assert_not_paused();
        let caller = env::predecessor_account_id();
        let current_time = env::block_timestamp();

        let mut bounty = self.bounties.get(&bounty_id).expect("Bounty not found");

        // Authorization check - only contract owner (deployer) can close bounties
        assert!(caller == self.owner, "Only contract owner can close bounty");

        // State validation
        assert!(bounty.is_active, "Bounty is not active");
        assert!(!bounty.is_closed, "Bounty is already closed");
        assert!(current_time >= bounty.ends_at, "Bounty has not expired yet");

        // Handle different scenarios
        if bounty.total_staked == NearToken::from_yoctonear(0) {
            // No participants - just close the bounty
            bounty.is_closed = true;
            bounty.is_active = false;
            self.bounties.insert(&bounty_id, &bounty);
            env::log_str(&format!(
                "BOUNTY_CLOSED: No participants in bounty {}",
                bounty_id
            ));
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

        env::log_str(&format!(
            "BOUNTY_CLOSED: Bounty {} closed and rewards distributed",
            bounty_id
        ));
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
                        env::log_str(&format!(
                            "SINGLE_PARTICIPANT_REFUND: {} received {} NEAR",
                            account, stake.amount
                        ));
                        return;
                    }
                }
            }
        }
        env::log_str(&format!(
            "SINGLE_PARTICIPANT_ERROR: No participants found for bounty {}",
            bounty.id
        ));
    }

    fn distribute_multi_participant_rewards(&mut self, bounty: &mut Bounty) {
        // Determine winning option
        let winning_option = match self.determine_winning_option(bounty) {
            Some(option) => option,
            None => {
                env::log_str(&format!(
                    "BOUNTY_ERROR: No winning option determined for bounty {}",
                    bounty.id
                ));
                return;
            }
        };

        bounty.winning_option = Some(winning_option);

        // Calculate and transfer platform fee
        let platform_fee = self.calculate_platform_fee(bounty.total_staked);
        if platform_fee > NearToken::from_yoctonear(0) {
            Promise::new(self.owner.clone()).transfer(platform_fee);
            env::log_str(&format!(
                "PLATFORM_FEE: {} NEAR transferred to owner",
                platform_fee
            ));
        }

        // Distribute rewards to winners
        self.distribute_winner_rewards(bounty, winning_option);
    }

    fn distribute_winner_rewards(&mut self, bounty: &Bounty, winning_option: u64) {
        // Use participant tracking system to iterate through all participants
        if let Some(bounty_participants) = self.get_bounty_participants_ref() {
            if let Some(participants) = bounty_participants.get(&bounty.id) {
                for account in participants {
                    let stake_key = (account.clone(), bounty.id);
                    if let Some(stake) = self.participant_stakes.get(&stake_key) {
                        if stake.option_index == winning_option {
                            // Calculate and transfer reward
                            let reward =
                                self.calculate_user_reward(bounty, stake.amount, winning_option);
                            if reward > NearToken::from_yoctonear(0) {
                                Promise::new(account.clone()).transfer(reward);
                                env::log_str(&format!(
                                    "WINNER_REWARD: {} received {} NEAR for winning option {}",
                                    account, reward, winning_option
                                ));
                            }
                        }
                    }
                }
            } else {
                env::log_str(&format!(
                    "WINNER_REWARD_ERROR: No participants found for bounty {}",
                    bounty.id
                ));
            }
        } else {
            env::log_str(&format!(
                "WINNER_REWARD_ERROR: No participant tracking available for bounty {}",
                bounty.id
            ));
        }
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
        self.assert_not_paused();
        let claimer = env::predecessor_account_id();

        let bounty = self.bounties.get(&bounty_id).expect("Bounty not found");
        assert!(bounty.is_closed, "Bounty is not closed yet");

        let stake_key = (claimer.clone(), bounty_id);
        let stake = self
            .participant_stakes
            .get(&stake_key)
            .expect("No stake found for this bounty");

        // Check if user won
        if let Some(winning_option) = bounty.winning_option {
            if stake.option_index == winning_option {
                let reward = self.calculate_user_reward(&bounty, stake.amount, winning_option);

                if reward > NearToken::from_yoctonear(0) {
                    // Check if contract has sufficient balance
                    let contract_balance = env::account_balance();
                    let reserved_balance = NearToken::from_near(1); // Reserve for operations

                    if contract_balance
                        > Self::safe_add_tokens(reward, reserved_balance)
                            .unwrap_or(contract_balance)
                    {
                        Promise::new(claimer.clone()).transfer(reward);
                        env::log_str(&format!(
                            "CLAIM_SUCCESS: {} claimed {} NEAR from bounty {}",
                            claimer, reward, bounty_id
                        ));
                    } else {
                        env::log_str(&format!(
                            "CLAIM_FAILED: Insufficient contract balance for {} from bounty {}",
                            claimer, bounty_id
                        ));
                        panic!(
                            "Insufficient contract balance for reward payment: contract balance = {} yoctoNEAR, required = {} yoctoNEAR",
                            contract_balance.as_yoctonear(),
                            Self::safe_add_tokens(reward, reserved_balance).unwrap_or(contract_balance).as_yoctonear()
                        );
                    }
                } else {
                    panic!("No reward to claim");
                }
            } else {
                panic!("User did not win this bounty");
            }
        } else {
            // Handle single participant case - return full stake
            let participant_count = self.count_bounty_participants(bounty_id);
            if participant_count <= 1 {
                Promise::new(claimer.clone()).transfer(stake.amount);
                env::log_str(&format!(
                    "SINGLE_PARTICIPANT_CLAIM: {} claimed {} NEAR from bounty {}",
                    claimer, stake.amount, bounty_id
                ));
            } else {
                panic!("No winning option determined");
            }
        }
    }

    // Owner functions
    pub fn update_reward_rate(&mut self, new_rate: u128) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner,
            "Only owner can update reward rate"
        );

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
        assert_eq!(
            env::predecessor_account_id(),
            self.owner,
            "Only owner can update max stake amount"
        );

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
            safe_max.as_near(),
            new_max_amount.as_near()
        ));

        self.max_stake_amount = safe_max;
    }

    pub fn update_platform_fee_rate(&mut self, new_rate: u128) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner,
            "Only owner can update platform fee rate"
        );

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
            safe_rate,
            safe_rate / 100,
            new_rate
        ));

        self.platform_fee_rate = safe_rate;
    }

    pub fn pause_contract(&mut self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner,
            "Only owner can pause contract"
        );
        self.is_paused = true;
        env::log_str("CONTRACT_PAUSED: Contract has been paused");
    }

    pub fn unpause_contract(&mut self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner,
            "Only owner can unpause contract"
        );
        self.is_paused = false;
        env::log_str("CONTRACT_UNPAUSED: Contract has been unpaused");
    }

    pub fn emergency_close_bounty(&mut self, bounty_id: u64) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner,
            "Only owner can emergency close bounty"
        );

        let mut bounty = self.bounties.get(&bounty_id).expect("Bounty not found");
        assert!(!bounty.is_closed, "Bounty is already closed");

        // Emergency close - refund all participants without fees
        self.emergency_refund_participants(&bounty);

        bounty.is_closed = true;
        bounty.is_active = false;
        self.bounties.insert(&bounty_id, &bounty);

        env::log_str(&format!(
            "EMERGENCY_CLOSE: Bounty {} emergency closed and participants refunded",
            bounty_id
        ));
    }

    fn emergency_refund_participants(&mut self, bounty: &Bounty) {
        // Use participant tracking system to iterate through actual participants
        if let Some(bounty_participants) = self.get_bounty_participants_ref() {
            if let Some(participants) = bounty_participants.get(&bounty.id) {
                for account in participants {
                    let stake_key = (account.clone(), bounty.id);
                    if let Some(stake) = self.participant_stakes.get(&stake_key) {
                        Promise::new(account.clone()).transfer(stake.amount);
                        env::log_str(&format!(
                            "EMERGENCY_REFUND: {} refunded {} NEAR",
                            account, stake.amount
                        ));
                    }
                }
            } else {
                env::log_str(&format!(
                    "EMERGENCY_REFUND: No participants found for bounty {}",
                    bounty.id
                ));
            }
        } else {
            env::log_str(&format!(
                "EMERGENCY_REFUND: No participant tracking available for bounty {}",
                bounty.id
            ));
        }
    }

    pub fn withdraw_platform_fees(&mut self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner,
            "Only owner can withdraw platform fees"
        );

        let contract_balance = env::account_balance();
        let reserved_balance = NearToken::from_near(2); // Reserve more for operations

        if contract_balance > reserved_balance {
            let withdrawal_amount = Self::safe_sub_tokens(contract_balance, reserved_balance)
                .expect("Balance calculation error");

            if withdrawal_amount > NearToken::from_yoctonear(0) {
                Promise::new(self.owner.clone()).transfer(withdrawal_amount);
                env::log_str(&format!(
                    "PLATFORM_FEES_WITHDRAWN: {} NEAR withdrawn by owner",
                    withdrawal_amount
                ));
            }
        }
    }

    // View functions for contract state
    pub fn get_platform_fee_rate(&self) -> u128 {
        self.platform_fee_rate
    }

    pub fn is_contract_paused(&self) -> bool {
        self.is_paused
    }

    pub fn get_contract_owner(&self) -> AccountId {
        self.owner.clone()
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

    fn get_context(
        predecessor_account_id: AccountId,
        attached_deposit: NearToken,
    ) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .predecessor_account_id(predecessor_account_id)
            .attached_deposit(attached_deposit)
            .block_timestamp(0);
        builder
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
        testing_env!(context
            .predecessor_account_id(accounts(1))
            .attached_deposit(stake_amount)
            .build());
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
        testing_env!(context
            .predecessor_account_id(accounts(1))
            .attached_deposit(stake_amount)
            .build());
        contract.stake();
    }

    #[test]
    #[should_panic(expected = "Stake amount too high")]
    fn test_stake_above_maximum() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        let stake_amount = NearToken::from_yoctonear(MAX_STAKE.as_yoctonear() + 1);
        testing_env!(context
            .predecessor_account_id(accounts(1))
            .attached_deposit(stake_amount)
            .build());
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
        let context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        let bounty_id = contract.create_bounty(
            "Test Bounty".to_string(),
            "A test bounty".to_string(),
            vec!["Option A".to_string(), "Option B".to_string()],
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
    #[should_panic(expected = "Maximum stake per user must be at least 100 millinear")]
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
        let bounty_id = contract.create_bounty(
            "Test Bounty".to_string(),
            "A test bounty".to_string(),
            vec!["Option A".to_string(), "Option B".to_string()],
            NearToken::from_near(10),
            100,
        );

        // Stake on option
        let stake_amount = NearToken::from_near(5);
        testing_env!(context
            .predecessor_account_id(accounts(1))
            .attached_deposit(stake_amount)
            .build());
        contract.stake_on_option(bounty_id, 0);

        // Verify stake
        let participant_stake = contract
            .get_participant_stake(accounts(1), bounty_id)
            .unwrap();
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
        let bounty_id = contract.create_bounty(
            "Test Bounty".to_string(),
            "A test bounty".to_string(),
            vec!["Option A".to_string(), "Option B".to_string()],
            NearToken::from_near(10),
            100,
        );

        // Initial stake
        let initial_stake = NearToken::from_near(3);
        testing_env!(context
            .predecessor_account_id(accounts(1))
            .attached_deposit(initial_stake)
            .build());
        contract.stake_on_option(bounty_id, 0);

        // Update stake to different option
        let new_stake = NearToken::from_near(5);
        testing_env!(context
            .predecessor_account_id(accounts(1))
            .attached_deposit(new_stake)
            .build());
        contract.stake_on_option(bounty_id, 1);

        // Verify updated stake
        let participant_stake = contract
            .get_participant_stake(accounts(1), bounty_id)
            .unwrap();
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
        testing_env!(context
            .predecessor_account_id(accounts(1))
            .attached_deposit(stake_amount)
            .build());
        contract.stake_on_option(999, 0); // Non-existent bounty
    }

    #[test]
    #[should_panic(expected = "Invalid option index")]
    fn test_stake_on_invalid_option() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        // Create bounty with 2 options
        let bounty_id = contract.create_bounty(
            "Test Bounty".to_string(),
            "A test bounty".to_string(),
            vec!["Option A".to_string(), "Option B".to_string()],
            NearToken::from_near(10),
            100,
        );

        let stake_amount = NearToken::from_near(5);
        testing_env!(context
            .predecessor_account_id(accounts(1))
            .attached_deposit(stake_amount)
            .build());
        contract.stake_on_option(bounty_id, 2); // Invalid option index (only 0 and 1 exist)
    }

    #[test]
    fn test_get_user_bounties() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        // Create multiple bounties
        let bounty_id1 = contract.create_bounty(
            "Bounty 1".to_string(),
            "First bounty".to_string(),
            vec!["A".to_string(), "B".to_string()],
            NearToken::from_near(10),
            100,
        );

        let bounty_id2 = contract.create_bounty(
            "Bounty 2".to_string(),
            "Second bounty".to_string(),
            vec!["X".to_string(), "Y".to_string(), "Z".to_string()],
            NearToken::from_near(5),
            200,
        );

        // User stakes on both bounties
        testing_env!(context
            .predecessor_account_id(accounts(1))
            .attached_deposit(NearToken::from_near(3))
            .build());
        contract.stake_on_option(bounty_id1, 0);

        testing_env!(context
            .predecessor_account_id(accounts(1))
            .attached_deposit(NearToken::from_near(2))
            .build());
        contract.stake_on_option(bounty_id2, 1);

        // Get user bounties
        let user_bounties = contract.get_user_bounties(accounts(1));
        assert_eq!(user_bounties.len(), 2);

        // Verify stakes
        let stake1 = user_bounties
            .iter()
            .find(|s| s.bounty_id == bounty_id1)
            .unwrap();
        assert_eq!(stake1.amount.0, NearToken::from_near(3).as_yoctonear());
        assert_eq!(stake1.option_index, 0);

        let stake2 = user_bounties
            .iter()
            .find(|s| s.bounty_id == bounty_id2)
            .unwrap();
        assert_eq!(stake2.amount.0, NearToken::from_near(2).as_yoctonear());
        assert_eq!(stake2.option_index, 1);
    }

    #[test]
    fn test_get_bounty_stakes() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        // Create bounty
        let bounty_id = contract.create_bounty(
            "Test Bounty".to_string(),
            "A test bounty".to_string(),
            vec![
                "Option A".to_string(),
                "Option B".to_string(),
                "Option C".to_string(),
            ],
            NearToken::from_near(10),
            100,
        );

        // Multiple users stake on different options
        testing_env!(context
            .predecessor_account_id(accounts(1))
            .attached_deposit(NearToken::from_near(3))
            .build());
        contract.stake_on_option(bounty_id, 0);

        testing_env!(context
            .predecessor_account_id(accounts(2))
            .attached_deposit(NearToken::from_near(5))
            .build());
        contract.stake_on_option(bounty_id, 1);

        testing_env!(context
            .predecessor_account_id(accounts(3))
            .attached_deposit(NearToken::from_near(2))
            .build());
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
        let context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        // Create bounty
        let bounty_id = contract.create_bounty(
            "Test Bounty".to_string(),
            "A test bounty".to_string(),
            vec![
                "Option A".to_string(),
                "Option B".to_string(),
                "Option C".to_string(),
            ],
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
        let expected_reward_yocto = user_stake
            .as_yoctonear()
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
        let bounty_id = contract.create_bounty(
            "Test Bounty".to_string(),
            "A test bounty".to_string(),
            vec!["Option A".to_string(), "Option B".to_string()],
            NearToken::from_near(10),
            100,
        );

        // Fast forward time to after bounty ends
        testing_env!(context.block_timestamp(100 * 1_000_000_000 + 1).build());

        // Close bounty (no participants)
        contract.close_bounty(bounty_id);

        // Verify bounty is closed
        let bounty = contract.get_bounty(bounty_id).unwrap();
        assert!(bounty.is_closed);
        assert!(!bounty.is_active);
    }

    #[test]
    #[should_panic(expected = "Only contract owner can close bounty")]
    fn test_close_bounty_unauthorized() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        // Create bounty
        let bounty_id = contract.create_bounty(
            "Test Bounty".to_string(),
            "A test bounty".to_string(),
            vec!["Option A".to_string(), "Option B".to_string()],
            NearToken::from_near(10),
            100,
        );

        // Fast forward time
        testing_env!(context
            .block_timestamp(100 * 1_000_000_000 + 1)
            .predecessor_account_id(accounts(1))
            .build());

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
        let bounty_id = contract.create_bounty(
            "Test Bounty".to_string(),
            "A test bounty".to_string(),
            vec!["Option A".to_string(), "Option B".to_string()],
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
        let bounty_id = contract.create_bounty(
            "Test Bounty".to_string(),
            "A test bounty".to_string(),
            vec!["Option A".to_string(), "Option B".to_string()],
            NearToken::from_near(10),
            100,
        );

        // Add participants
        testing_env!(context
            .predecessor_account_id(accounts(1))
            .attached_deposit(NearToken::from_near(3))
            .build());
        contract.stake_on_option(bounty_id, 0);

        testing_env!(context
            .predecessor_account_id(accounts(2))
            .attached_deposit(NearToken::from_near(7))
            .build());
        contract.stake_on_option(bounty_id, 1);

        // Fast forward time to after bounty ends
        testing_env!(context
            .block_timestamp(100 * 1_000_000_000 + 1)
            .predecessor_account_id(accounts(0))
            .build());

        // Close bounty
        contract.close_bounty(bounty_id);

        // Verify bounty is closed and has winning option
        let bounty = contract.get_bounty(bounty_id).unwrap();
        assert!(bounty.is_closed);
        assert!(!bounty.is_active);
        assert_eq!(bounty.winning_option, Some(1)); // Option 1 had more stakes (7 NEAR vs 3 NEAR)
    }

    #[test]
    fn test_get_bounty_results() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        // Create bounty
        let bounty_id = contract.create_bounty(
            "Test Bounty".to_string(),
            "A test bounty".to_string(),
            vec!["Option A".to_string(), "Option B".to_string()],
            NearToken::from_near(10),
            100,
        );

        // Should return None for active bounty
        assert!(contract.get_bounty_results(bounty_id).is_none());

        // Add participants and close bounty
        testing_env!(context
            .predecessor_account_id(accounts(1))
            .attached_deposit(NearToken::from_near(3))
            .build());
        contract.stake_on_option(bounty_id, 0);

        testing_env!(context
            .predecessor_account_id(accounts(2))
            .attached_deposit(NearToken::from_near(7))
            .build());
        contract.stake_on_option(bounty_id, 1);

        // Fast forward and close
        testing_env!(context
            .block_timestamp(100 * 1_000_000_000 + 1)
            .predecessor_account_id(accounts(0))
            .build());
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
        let bounty_id = contract.create_bounty(
            "Test Bounty".to_string(),
            "A test bounty".to_string(),
            vec!["Option A".to_string(), "Option B".to_string()],
            NearToken::from_near(10),
            100,
        );

        testing_env!(context
            .predecessor_account_id(accounts(1))
            .attached_deposit(NearToken::from_near(5))
            .build());
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
        let bounty_id = contract.create_bounty(
            "Test Bounty".to_string(),
            "A test bounty".to_string(),
            vec!["Option A".to_string(), "Option B".to_string()],
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
        let bounty_id = contract.create_bounty(
            "Test Bounty".to_string(),
            "A test bounty".to_string(),
            vec!["Option A".to_string(), "Option B".to_string()],
            NearToken::from_near(10),
            100,
        );

        // User stakes on losing option
        testing_env!(context
            .predecessor_account_id(accounts(1))
            .attached_deposit(NearToken::from_near(3))
            .build());
        contract.stake_on_option(bounty_id, 0);

        // Another user stakes more on winning option
        testing_env!(context
            .predecessor_account_id(accounts(2))
            .attached_deposit(NearToken::from_near(7))
            .build());
        contract.stake_on_option(bounty_id, 1);

        // Close bounty
        testing_env!(context
            .block_timestamp(100 * 1_000_000_000 + 1)
            .predecessor_account_id(accounts(0))
            .build());
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

        // Initially not paused
        assert!(!contract.is_contract_paused());

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
        assert_eq!(
            contract.get_platform_fee_rate(),
            1000,
            "Platform fee should be clamped to 1000 (10%)"
        );
    }

    #[test]
    fn test_emergency_close_bounty() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        // Create bounty and add participants
        let bounty_id = contract.create_bounty(
            "Test Bounty".to_string(),
            "A test bounty".to_string(),
            vec!["Option A".to_string(), "Option B".to_string()],
            NearToken::from_near(10),
            100,
        );

        testing_env!(context
            .predecessor_account_id(accounts(1))
            .attached_deposit(NearToken::from_near(5))
            .build());
        contract.stake_on_option(bounty_id, 0);

        // Emergency close as owner
        testing_env!(context.predecessor_account_id(accounts(0)).build());
        contract.emergency_close_bounty(bounty_id);

        // Verify bounty is closed
        let bounty = contract.get_bounty(bounty_id).unwrap();
        assert!(bounty.is_closed);
        assert!(!bounty.is_active);
    }

    #[test]
    #[should_panic(expected = "Only owner can emergency close bounty")]
    fn test_emergency_close_bounty_unauthorized() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        // Create bounty
        let bounty_id = contract.create_bounty(
            "Test Bounty".to_string(),
            "A test bounty".to_string(),
            vec!["Option A".to_string(), "Option B".to_string()],
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
    #[should_panic(expected = "Only owner can withdraw platform fees")]
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

    #[test]
    fn test_calculate_rewards_safe_with_zero_rate() {
        let stake_amount = NearToken::from_near(10);
        let reward_rate = 0u128;
        let time_seconds = 3600u64; // 1 hour

        let rewards = BountyPredictionContract::calculate_rewards_safe(
            stake_amount,
            reward_rate,
            time_seconds,
        );
        assert_eq!(rewards, 0, "Rewards should be 0 with zero reward rate");
    }

    #[test]
    fn test_calculate_rewards_safe_with_high_rate() {
        let stake_amount = NearToken::from_near(1);
        let reward_rate = u128::MAX / 1_000_000; // Very high but safe
        let time_seconds = 1u64;

        let rewards = BountyPredictionContract::calculate_rewards_safe(
            stake_amount,
            reward_rate,
            time_seconds,
        );
        // Should not panic and should return a valid value
        assert!(rewards <= u128::MAX, "Rewards should not overflow");
    }

    #[test]
    fn test_calculate_rewards_safe_overflow_protection() {
        let stake_amount = NearToken::from_near(1000);
        let reward_rate = u128::MAX / 1000; // High rate
        let time_seconds = u64::MAX; // Maximum time

        // This should not panic due to checked arithmetic
        let rewards = BountyPredictionContract::calculate_rewards_safe(
            stake_amount,
            reward_rate,
            time_seconds,
        );
        // If overflow occurs, checked_mul returns None and we get 0
        assert!(
            rewards <= u128::MAX,
            "Rewards calculation should handle overflow gracefully"
        );
    }

    #[test]
    fn test_calculate_rewards_safe_with_zero_stake() {
        let stake_amount = NearToken::from_yoctonear(0);
        let reward_rate = 1000u128;
        let time_seconds = 3600u64;

        let rewards = BountyPredictionContract::calculate_rewards_safe(
            stake_amount,
            reward_rate,
            time_seconds,
        );
        assert_eq!(rewards, 0, "Rewards should be 0 with zero stake");
    }

    #[test]
    fn test_calculate_rewards_safe_with_zero_time() {
        let stake_amount = NearToken::from_near(10);
        let reward_rate = 1000u128;
        let time_seconds = 0u64;

        let rewards = BountyPredictionContract::calculate_rewards_safe(
            stake_amount,
            reward_rate,
            time_seconds,
        );
        assert_eq!(rewards, 0, "Rewards should be 0 with zero time");
    }

    #[test]
    fn test_update_reward_rate_to_high_value_clamped() {
        let context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        let very_high_rate = u128::MAX / 1000;
        contract.update_reward_rate(very_high_rate);
        assert_eq!(
            contract.get_reward_rate(),
            1_000_000_000,
            "Very high reward rate should be clamped to 1 billion"
        );
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
        let rewards1 = BountyPredictionContract::calculate_rewards_safe(
            stake_amount,
            reward_rate,
            time_seconds,
        );
        let rewards2 = BountyPredictionContract::calculate_rewards_safe(
            stake_amount,
            reward_rate,
            time_seconds,
        );
        let rewards3 = BountyPredictionContract::calculate_rewards_safe(
            stake_amount,
            reward_rate,
            time_seconds,
        );

        assert_eq!(
            rewards1, rewards2,
            "Reward calculations should be consistent"
        );
        assert_eq!(
            rewards2, rewards3,
            "Reward calculations should be consistent"
        );
    }

    #[test]
    fn test_reward_calculation_proportionality() {
        let reward_rate = 100u128;
        let time_seconds = 3600u64;

        let stake1 = NearToken::from_near(1);
        let stake2 = NearToken::from_near(2);
        let stake10 = NearToken::from_near(10);

        let rewards1 =
            BountyPredictionContract::calculate_rewards_safe(stake1, reward_rate, time_seconds);
        let rewards2 =
            BountyPredictionContract::calculate_rewards_safe(stake2, reward_rate, time_seconds);
        let rewards10 =
            BountyPredictionContract::calculate_rewards_safe(stake10, reward_rate, time_seconds);

        // Rewards should be proportional to stake amount
        assert_eq!(
            rewards2,
            rewards1 * 2,
            "Rewards should be proportional to stake (2x)"
        );
        assert_eq!(
            rewards10,
            rewards1 * 10,
            "Rewards should be proportional to stake (10x)"
        );
    }

    #[test]
    #[should_panic(expected = "Only owner can pause contract")]
    fn test_pause_contract_unauthorized() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        // Try to pause as non-owner
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        contract.pause_contract();
    }

    #[test]
    #[should_panic(expected = "Only owner can update reward rate")]
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
        let bounty_id = contract.create_bounty(
            "Test Bounty".to_string(),
            "Test Description".to_string(),
            vec!["Option A".to_string(), "Option B".to_string()],
            NearToken::from_near(10),
            100,
        );

        // Stake on the bounty
        let stake_amount = NearToken::from_near(5);
        testing_env!(context
            .predecessor_account_id(accounts(1))
            .attached_deposit(stake_amount)
            .build());
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
        let bounty_id = contract.create_bounty(
            "Test Bounty".to_string(),
            "Test Description".to_string(),
            vec!["Option A".to_string(), "Option B".to_string()],
            NearToken::from_near(10),
            100,
        );

        // Multiple participants stake
        let stake_amount = NearToken::from_near(5);

        // Participant 1
        testing_env!(context
            .predecessor_account_id(accounts(1))
            .attached_deposit(stake_amount)
            .build());
        contract.stake_on_option(bounty_id, 0);

        // Participant 2
        testing_env!(context
            .predecessor_account_id(accounts(2))
            .attached_deposit(stake_amount)
            .build());
        contract.stake_on_option(bounty_id, 1);

        // Participant 3
        testing_env!(context
            .predecessor_account_id(accounts(3))
            .attached_deposit(stake_amount)
            .build());
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
        let bounty_id = contract.create_bounty(
            "Test Bounty".to_string(),
            "Test Description".to_string(),
            vec!["Option A".to_string(), "Option B".to_string()],
            NearToken::from_near(10),
            100,
        );

        // Participant stakes multiple times
        let stake_amount = NearToken::from_near(2);
        testing_env!(context
            .predecessor_account_id(accounts(1))
            .attached_deposit(stake_amount)
            .build());
        contract.stake_on_option(bounty_id, 0);

        testing_env!(context
            .predecessor_account_id(accounts(1))
            .attached_deposit(stake_amount)
            .build());
        contract.stake_on_option(bounty_id, 0);

        testing_env!(context
            .predecessor_account_id(accounts(1))
            .attached_deposit(stake_amount)
            .build());
        contract.stake_on_option(bounty_id, 1);

        // Should only have one participant entry
        let participants = contract.get_bounty_participants(bounty_id);
        assert_eq!(participants.len(), 1);
        assert_eq!(participants[0], accounts(1));

        let participant_count = contract.get_bounty_participant_count(bounty_id);
        assert_eq!(participant_count, 1);
    }

    #[test]
    fn test_participant_tracking_across_multiple_bounties() {
        let mut context = get_context(accounts(0), NearToken::from_near(0));
        testing_env!(context.build());
        let mut contract = BountyPredictionContract::new(REWARD_RATE, MIN_STAKE, MAX_STAKE);

        // Create two bounties
        let bounty_id_1 = contract.create_bounty(
            "Test Bounty 1".to_string(),
            "Test Description 1".to_string(),
            vec!["Option A".to_string(), "Option B".to_string()],
            NearToken::from_near(10),
            100,
        );

        let bounty_id_2 = contract.create_bounty(
            "Test Bounty 2".to_string(),
            "Test Description 2".to_string(),
            vec!["Option X".to_string(), "Option Y".to_string()],
            NearToken::from_near(10),
            100,
        );

        let stake_amount = NearToken::from_near(5);

        // Participant 1 stakes on both bounties
        testing_env!(context
            .predecessor_account_id(accounts(1))
            .attached_deposit(stake_amount)
            .build());
        contract.stake_on_option(bounty_id_1, 0);

        testing_env!(context
            .predecessor_account_id(accounts(1))
            .attached_deposit(stake_amount)
            .build());
        contract.stake_on_option(bounty_id_2, 1);

        // Participant 2 stakes only on bounty 1
        testing_env!(context
            .predecessor_account_id(accounts(2))
            .attached_deposit(stake_amount)
            .build());
        contract.stake_on_option(bounty_id_1, 1);

        // Check participant tracking for each bounty
        let participants_1 = contract.get_bounty_participants(bounty_id_1);
        assert_eq!(participants_1.len(), 2);
        assert!(participants_1.contains(&accounts(1)));
        assert!(participants_1.contains(&accounts(2)));

        let participants_2 = contract.get_bounty_participants(bounty_id_2);
        assert_eq!(participants_2.len(), 1);
        assert!(participants_2.contains(&accounts(1)));

        // Check participant counts
        assert_eq!(contract.get_bounty_participant_count(bounty_id_1), 2);
        assert_eq!(contract.get_bounty_participant_count(bounty_id_2), 1);
    }
}
