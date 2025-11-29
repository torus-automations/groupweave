// On-chain voting for content curation decisions.
// Communities create polls for governance and content approval.
// Users vote on options, can change votes before deadline.
// Vote counts stored immutably on blockchain for transparency.
//
// Security-hardened version with production limits (2025-11-02)

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near, require, AccountId, NearToken, PanicOnDefault, Promise};
use std::convert::TryFrom;

// Production Safety Limits
const MAX_VOTERS_PER_POLL: u64 = 100_000_000; // 100 million max voters
const MAX_OPTIONS_PER_POLL: usize = 100; // 100 max options
const MAX_DURATION_MINUTES: u64 = 525_600; // 1 year max duration
const MIN_REWARD_YOCTO: u128 = 100_000_000_000_000_000_000_000; // 0.1 NEAR min reward
const MAX_REWARD_YOCTO: u128 = 100_000_000_000_000_000_000_000_000_000_000; // 100M NEAR max (100,000,000 * 10^24)

// Text Length Limits
const TITLE_MAX: usize = 120;
const CRITERIA_MIN: usize = 8;
const CRITERIA_MAX: usize = 200;
const DETAILS_MAX: usize = 600;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct OptionEntry {
    pub label: String,
    pub recipient: AccountId,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct OptionInput {
    pub label: String,
    pub recipient: AccountId,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Poll {
    pub id: u64,
    pub title: String,
    // Short criteria shown prominently to voters (hard limit enforced)
    pub criteria: String,
    // Optional short details/description; allows a bit more room than criteria
    pub description: Option<String>,
    pub options: Vec<OptionEntry>,
    pub votes: Vec<u64>,
    pub creator: AccountId,
    pub is_active: bool,
    pub is_open: bool,
    pub created_at: u64,
    pub ends_at: Option<u64>,
    // Reward escrowed in yoctoNEAR for winners (split among winning options' recipients)
    pub reward_yocto: u128,
    pub payout_done: bool,
    // Total unique voters (for security limit enforcement)
    pub total_voters: u64,
    // Whether the poll creator is allowed to vote
    pub allow_creator_vote: bool,
}

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct VotingContract {
    polls: LookupMap<u64, Poll>,
    user_votes: LookupMap<(AccountId, u64), u64>, // (user, poll_id) -> option_index
    whitelist: LookupMap<(u64, AccountId), bool>, // (poll_id, account) -> allowed
    owner: AccountId,
    platform_fee_bps: u16, // 0-10000 basis points
    next_poll_id: u64,
}

#[near]
impl VotingContract {
    #[init]
    pub fn new() -> Self {
        Self {
            polls: LookupMap::new(b"p"),
            user_votes: LookupMap::new(b"v"),
            whitelist: LookupMap::new(b"w"),
            owner: env::predecessor_account_id(),
            platform_fee_bps: 0,
            next_poll_id: 1,
        }
    }

    pub fn set_platform_fee_bps(&mut self, bps: u16) {
        require!(env::predecessor_account_id() == self.owner, "Only owner can set fee");
        require!(bps <= 2000, "Fee too high"); // cap at 20%
        self.platform_fee_bps = bps;
    }

    pub fn get_platform_fee_bps(&self) -> u16 {
        self.platform_fee_bps
    }

    pub fn add_to_whitelist(&mut self, poll_id: u64, account: AccountId) {
        let poll = self.polls.get(&poll_id).expect("Poll not found");
        require!(env::predecessor_account_id() == poll.creator, "Only creator can whitelist");
        let initial = env::storage_usage();
        self.whitelist.insert(&(poll_id, account), &true);
        // Refund any excess deposit; charge storage if needed
        let storage_used = env::storage_usage().saturating_sub(initial);
        let cost = u128::from(storage_used) * env::storage_byte_cost().as_yoctonear();
        let attached = env::attached_deposit().as_yoctonear();
        require!(attached >= cost, "Insufficient deposit for whitelist entry");
        let refund = attached - cost;
        if refund > 0 {
            Promise::new(env::predecessor_account_id()).transfer(NearToken::from_yoctonear(refund));
        }
        // Re-store poll (no changes but ensures borsh consistency)
        self.polls.insert(&poll_id, &poll);
    }

    pub fn remove_from_whitelist(&mut self, poll_id: u64, account: AccountId) {
        let poll = self.polls.get(&poll_id).expect("Poll not found");
        require!(env::predecessor_account_id() == poll.creator, "Only creator can whitelist");
        self.whitelist.remove(&(poll_id, account));
        // No refunds on removal (storage freed to contract balance per NEAR model)
    }

    pub fn is_whitelisted(&self, poll_id: u64, account: AccountId) -> bool {
        self.whitelist.get(&(poll_id, account)).unwrap_or(false)
    }

    #[payable]
    pub fn create_poll(
        &mut self,
        title: String,
        criteria: String,
        details: Option<String>,
        options: Vec<OptionInput>,
        duration_minutes: Option<u64>,
        is_open: bool,
        reward_yocto: Option<u128>,
        allow_creator_vote: Option<bool>,
    ) -> u64 {
        // Normalize inputs
        let title = title.trim().to_string();
        let criteria = criteria.trim().to_string();
        let details = details.map(|d| d.trim().to_string()).filter(|d| !d.is_empty());

        // Validate title
        require!(!title.is_empty(), "Title cannot be empty");
        require!(title.chars().count() <= TITLE_MAX, "Title exceeds max length");

        // Validate criteria
        let crit_len = criteria.chars().count();
        require!(crit_len >= CRITERIA_MIN, "Criteria is too short");
        require!(crit_len <= CRITERIA_MAX, "Criteria exceeds max length");

        // Validate details
        if let Some(ref d) = details {
            require!(d.chars().count() <= DETAILS_MAX, "Details exceed max length");
        }

        // Validate options count
        require!(options.len() >= 2, "Poll must include at least two options");
        require!(options.len() <= MAX_OPTIONS_PER_POLL, "Too many options (max 100)");

        let mut opt_entries: Vec<OptionEntry> = Vec::with_capacity(options.len());
        for (index, option) in options.into_iter().enumerate() {
            let label = option.label.trim().to_string();
            require!(!label.is_empty(), format!("Option {} cannot be empty", index));
            opt_entries.push(OptionEntry { label, recipient: option.recipient });
        }

        // Validate duration
        if let Some(minutes) = duration_minutes {
            require!(minutes > 0, "Duration must be positive");
            require!(minutes <= MAX_DURATION_MINUTES, "Duration exceeds maximum (1 year)");
        }

        // Validate reward amount
        let reward = reward_yocto.unwrap_or(0);
        if reward > 0 {
            require!(reward >= MIN_REWARD_YOCTO, "Reward must be at least 0.1 NEAR");
            require!(reward <= MAX_REWARD_YOCTO, "Reward exceeds maximum (100M NEAR)");
        }

        let initial_storage = env::storage_usage();
        let attached_deposit = env::attached_deposit();

        let poll_id = self.next_poll_id;
        let creator = env::predecessor_account_id();
        let created_at = env::block_timestamp();
        let ends_at = duration_minutes.map(|minutes| {
            const SECONDS_PER_MINUTE: u64 = 60;
            const NANOS_PER_SECOND: u64 = 1_000_000_000;

            let duration_ns = (minutes as u128)
                .checked_mul(SECONDS_PER_MINUTE as u128)
                .and_then(|seconds| seconds.checked_mul(NANOS_PER_SECOND as u128))
                .expect("Duration is too large");

            let created_at_u128 = created_at as u128;
            let ends_at_u128 = created_at_u128
                .checked_add(duration_ns)
                .expect("Duration causes overflow");

            u64::try_from(ends_at_u128).expect("Duration exceeds supported range")
        });
        
        let votes = vec![0; opt_entries.len()];

        let poll = Poll {
            id: poll_id,
            title,
            criteria,
            description: details,
            options: opt_entries,
            votes,
            creator,
            is_active: true,
            is_open,
            created_at,
            ends_at,
            reward_yocto: reward,
            payout_done: false,
            total_voters: 0,
            allow_creator_vote: allow_creator_vote.unwrap_or(false),
        };
        
        self.polls.insert(&poll_id, &poll);
        self.next_poll_id += 1;

        let storage_used = env::storage_usage().saturating_sub(initial_storage);
        let storage_cost = u128::from(storage_used) * env::storage_byte_cost().as_yoctonear();
        let reward_needed = poll.reward_yocto;
        let required_deposit_raw = storage_cost.saturating_add(reward_needed);
        let attached_deposit_raw = attached_deposit.as_yoctonear();
        
        if required_deposit_raw > 0 {
            require!(
                attached_deposit_raw >= required_deposit_raw,
                format!(
                    "Attached deposit {} is less than required amount {} (storage + reward)",
                    attached_deposit_raw, required_deposit_raw
                )
            );
            let refund_raw = attached_deposit_raw - required_deposit_raw;
            if refund_raw > 0 {
                Promise::new(env::predecessor_account_id()).transfer(NearToken::from_yoctonear(refund_raw));
            }
        } else if attached_deposit_raw > 0 {
            Promise::new(env::predecessor_account_id()).transfer(attached_deposit);
        }
        
        poll_id
    }

    #[payable]
    pub fn vote(&mut self, poll_id: u64, option_index: u64) {
        let initial_storage = env::storage_usage();
        let attached_deposit = env::attached_deposit();
        let voter = env::predecessor_account_id();

        // Check if poll exists and is active
        let mut poll = self.polls.get(&poll_id).expect("Poll not found");
        require!(poll.is_active, "Poll is not active");
        if !poll.is_open {
            require!(self.whitelist.get(&(poll_id, voter.clone())).unwrap_or(false), "Not whitelisted for this poll");
        }

        // Check if poll has expired
        if let Some(ends_at) = poll.ends_at {
            require!(env::block_timestamp() < ends_at, "Poll has expired");
        }

        // Enforce creator voting policy
        if !poll.allow_creator_vote && voter == poll.creator {
            env::panic_str("Creator voting is disabled for this poll");
        }

        // Check if option index is valid
        require!((option_index as usize) < poll.options.len(), "Invalid option index");

        // Check if user has already voted and enforce voter limit
        let vote_key = (voter.clone(), poll_id);
        let is_new_voter = !self.user_votes.contains_key(&vote_key);

        if is_new_voter {
            // Enforce maximum voters limit (100 million)
            require!(
                poll.total_voters < MAX_VOTERS_PER_POLL,
                "Poll has reached maximum voters (100 million)"
            );
            poll.total_voters += 1;
        } else {
            // User is changing their vote - remove previous vote count
            let previous_vote = self.user_votes.get(&vote_key).unwrap();
            // Use saturating_sub to prevent underflow in case of state corruption
            poll.votes[previous_vote as usize] = poll.votes[previous_vote as usize].saturating_sub(1);
        }

        // Add new vote
        poll.votes[option_index as usize] += 1;
        self.user_votes.insert(&vote_key, &option_index);
        self.polls.insert(&poll_id, &poll);

        let storage_used = env::storage_usage() 
            .saturating_sub(initial_storage);
        let required_deposit_raw =
            u128::from(storage_used) * env::storage_byte_cost().as_yoctonear();
        let attached_deposit_raw = attached_deposit.as_yoctonear();
        
        if required_deposit_raw > 0 {
        require!(
            attached_deposit_raw >= required_deposit_raw,
            format!("Attached deposit {} is less than required storage cost {}",
                attached_deposit_raw,
                required_deposit_raw)
        );
            let refund_raw = attached_deposit_raw - required_deposit_raw;
            if refund_raw > 0 {
                Promise::new(env::predecessor_account_id()).transfer(NearToken::from_yoctonear(refund_raw));
            }
        } else if attached_deposit_raw > 0 {
            Promise::new(env::predecessor_account_id()).transfer(attached_deposit);
        }
    }

    pub fn get_poll(&self, poll_id: u64) -> Option<Poll> {
        self.polls.get(&poll_id)
    }

    pub fn get_user_vote(&self, poll_id: u64, user: AccountId) -> Option<u64> {
        self.user_votes.get(&(user, poll_id))
    }

    pub fn close_poll(&mut self, poll_id: u64) {
        let mut poll = self.polls.get(&poll_id).expect("Poll not found");
        assert_eq!(poll.creator, env::predecessor_account_id(), "Only creator can close poll");
        require!(poll.is_active, "Poll already closed");
        poll.is_active = false;

        // Payout if reward is present and not yet paid
        if !poll.payout_done && poll.reward_yocto > 0 {
            let mut max_votes = 0u64;
            for &v in &poll.votes { if v > max_votes { max_votes = v; } }
            let mut winner_indices: Vec<usize> = Vec::new();
            for (i, &v) in poll.votes.iter().enumerate() { if v == max_votes && max_votes > 0 { winner_indices.push(i); } }

            // Compute platform fee and per-winner payout
            let fee = (poll.reward_yocto as u128) * (self.platform_fee_bps as u128) / 10_000u128;
            let net = poll.reward_yocto.saturating_sub(fee);
            if fee > 0 {
                Promise::new(self.owner.clone()).transfer(NearToken::from_yoctonear(fee));
            }
            if !winner_indices.is_empty() && net > 0 {
                let winners_count = winner_indices.len() as u128;
                let per = net / winners_count;
                // Use saturating_sub for safety (per * winners_count should always be <= net)
                let mut remainder = net.saturating_sub(per * winners_count);
                // Deduplicate recipients if necessary
                let mut paid: Vec<AccountId> = Vec::new();
                for idx in winner_indices {
                    let to = poll.options[idx].recipient.clone();
                    if !paid.contains(&to) {
                        if per > 0 { Promise::new(to.clone()).transfer(NearToken::from_yoctonear(per)); }
                        paid.push(to);
                    } else {
                        // If duplicate recipient and per>0 already sent once, add remainder bucket
                        remainder = remainder.saturating_add(per);
                    }
                }
                if remainder > 0 {
                    Promise::new(poll.creator.clone()).transfer(NearToken::from_yoctonear(remainder));
                }
            } else if net > 0 {
                // No winners (all zero) -> refund creator
                Promise::new(poll.creator.clone()).transfer(NearToken::from_yoctonear(net));
            }
            poll.reward_yocto = 0;
            poll.payout_done = true;
        }

        self.polls.insert(&poll_id, &poll);
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;
    use near_sdk::NearToken;

    fn get_context(predecessor: AccountId, attached_deposit: NearToken) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .predecessor_account_id(predecessor)
            .attached_deposit(attached_deposit)
            .block_timestamp(0);
        builder
    }

    fn init_contract() -> VotingContract {
        VotingContract::new()
    }

    // ========================================
    // Initialization Tests
    // ========================================

    #[test]
    fn test_contract_initialization() {
        let context = get_context(accounts(0), NearToken::from_yoctonear(0));
        testing_env!(context.build());
        let contract = init_contract();
        
        // Verify initial state
        assert_eq!(contract.next_poll_id, 1, "Initial poll ID should be 1");
    }

    // ========================================
    // Poll Creation Tests
    // ========================================

    #[test]
    fn test_create_poll_basic() {
        let context = get_context(accounts(0), NearToken::from_near(1));
        testing_env!(context.build());
        let mut contract = init_contract();

        let poll_id = contract.create_poll(
            "Test Poll".to_string(),
            "Pick the best".to_string(),
            Some("Longer details here".to_string()),
            vec![
                OptionInput { label: "Option A".to_string(), recipient: accounts(1) },
                OptionInput { label: "Option B".to_string(), recipient: accounts(2) },
            ],
            Some(60),
            true,
            Some(0),
            None,
        );

        assert_eq!(poll_id, 1, "First poll should have ID 1");

        let poll = contract.get_poll(poll_id).unwrap();
        assert_eq!(poll.title, "Test Poll");
        assert_eq!(poll.criteria, "Pick the best");
        assert_eq!(poll.description.as_deref(), Some("Longer details here"));
        assert_eq!(poll.options.len(), 2);
        assert!(poll.is_active);
        assert_eq!(poll.creator, accounts(0));
    }

    #[test]
    fn test_create_poll_with_multiple_options() {
        let context = get_context(accounts(0), NearToken::from_near(1));
        testing_env!(context.build());
        let mut contract = init_contract();

        let options = vec![
            OptionInput { label: "Option 1".to_string(), recipient: accounts(1) },
            OptionInput { label: "Option 2".to_string(), recipient: accounts(2) },
            OptionInput { label: "Option 3".to_string(), recipient: accounts(3) },
            OptionInput { label: "Option 4".to_string(), recipient: accounts(4) },
            OptionInput { label: "Option 5".to_string(), recipient: accounts(5) },
        ];

        let poll_id = contract.create_poll(
            "Multi-Option Poll".to_string(),
            "Choose wisely".to_string(),
            None,
            options.clone(),
            None,
            true,
            Some(0),
            None,
        );

        let poll = contract.get_poll(poll_id).unwrap();
        assert_eq!(poll.options.len(), 5);
        assert_eq!(poll.votes.len(), 5);
        assert!(poll.votes.iter().all(|&v| v == 0));
    }

    #[test]
    fn test_create_poll_without_duration() {
        let context = get_context(accounts(0), NearToken::from_near(1));
        testing_env!(context.build());
        let mut contract = init_contract();

        let poll_id = contract.create_poll(
            "Unlimited Poll".to_string(),
            "No criteria".to_string(),
            None,
            vec![
                OptionInput { label: "Yes".to_string(), recipient: accounts(1) },
                OptionInput { label: "No".to_string(), recipient: accounts(2) },
            ],
            None,
            true,
            Some(0),
            None,
        );

        let poll = contract.get_poll(poll_id).unwrap();
        assert!(poll.ends_at.is_none(), "Poll without duration should have no end time");
    }

    #[test]
    fn test_create_poll_with_exact_duration() {
        let mut context = get_context(accounts(0), NearToken::from_near(1));
        let start_time = 1_000_000_000_000u64;
        testing_env!(context.block_timestamp(start_time).build());
        let mut contract = init_contract();

        let duration_minutes = 120u64; // 2 hours
        let poll_id = contract.create_poll(
            "Timed Poll".to_string(),
            "Short criteria".to_string(),
            None,
            vec![
                OptionInput { label: "A".to_string(), recipient: accounts(1) },
                OptionInput { label: "B".to_string(), recipient: accounts(2) },
            ],
            Some(duration_minutes),
            true,
            Some(0),
            None,
        );

        let poll = contract.get_poll(poll_id).unwrap();
        require!(poll.ends_at.is_some());
        
        let expected_end = start_time + (duration_minutes * 60 * 1_000_000_000);
        assert_eq!(poll.ends_at.unwrap(), expected_end);
    }

    #[test]
    #[should_panic(expected = "Poll must include at least two options")]
    fn test_create_poll_too_few_options() {
        let context = get_context(accounts(0), NearToken::from_near(1));
        testing_env!(context.build());
        let mut contract = init_contract();

        contract.create_poll(
            "Invalid Poll".to_string(),
            "Only one option".to_string(),
            None,
            vec![OptionInput { label: "Only Option".to_string(), recipient: accounts(1) }],
            None,
            true,
            Some(0),
            None,
        );
    }

    #[test]
    #[should_panic(expected = "Option 0 cannot be empty")]
    fn test_create_poll_empty_option() {
        let context = get_context(accounts(0), NearToken::from_near(1));
        testing_env!(context.build());
        let mut contract = init_contract();

        contract.create_poll(
            "Poll with Empty Option".to_string(),
            "Has empty option".to_string(),
            None,
            vec![
                OptionInput { label: "".to_string(), recipient: accounts(1) },
                OptionInput { label: "Valid Option".to_string(), recipient: accounts(2) },
            ],
            None,
            true,
            Some(0),
            None,
        );
    }

    #[test]
    #[should_panic(expected = "Option 1 cannot be empty")]
    fn test_create_poll_whitespace_only_option() {
        let context = get_context(accounts(0), NearToken::from_near(1));
        testing_env!(context.build());
        let mut contract = init_contract();

        contract.create_poll(
            "Poll with Whitespace Option".to_string(),
            "Has whitespace-only option".to_string(),
            None,
            vec![
                OptionInput { label: "Valid".to_string(), recipient: accounts(1) },
                OptionInput { label: "   ".to_string(), recipient: accounts(2) },
            ],
            None,
            true,
            Some(0),
            None,
        );
    }

    #[test]
    fn test_create_multiple_polls_sequential() {
        let context = get_context(accounts(0), NearToken::from_near(1));
        testing_env!(context.build());
        let mut contract = init_contract();

        let poll_id1 = contract.create_poll(
            "Poll 1".to_string(),
            "First poll".to_string(),
            None,
            vec![
                OptionInput { label: "A".to_string(), recipient: accounts(1) },
                OptionInput { label: "B".to_string(), recipient: accounts(2) },
            ],
            None,
            true,
            Some(0),
            None,
        );

        let poll_id2 = contract.create_poll(
            "Poll 2".to_string(),
            "Second poll".to_string(),
            None,
            vec![
                OptionInput { label: "X".to_string(), recipient: accounts(1) },
                OptionInput { label: "Y".to_string(), recipient: accounts(2) },
            ],
            None,
            true,
            Some(0),
            None,
        );

        let poll_id3 = contract.create_poll(
            "Poll 3".to_string(),
            "Third poll".to_string(),
            None,
            vec![
                OptionInput { label: "1".to_string(), recipient: accounts(1) },
                OptionInput { label: "2".to_string(), recipient: accounts(2) },
            ],
            None,
            true,
            Some(0),
            None,
        );

        assert_eq!(poll_id1, 1);
        assert_eq!(poll_id2, 2);
        assert_eq!(poll_id3, 3);
    }

    // ========================================
    // Voting Tests
    // ========================================

    #[test]
    fn test_vote_basic() {
        let mut context = get_context(accounts(0), NearToken::from_near(1));
        testing_env!(context.build());
        let mut contract = init_contract();

        let poll_id = contract.create_poll(
            "Test Poll".to_string(),
            "Description".to_string(),
            None,
            vec![
                OptionInput { label: "Option A".to_string(), recipient: accounts(2) },
                OptionInput { label: "Option B".to_string(), recipient: accounts(3) },
            ],
            None,
            true,
            Some(0),
            None,
        );

        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(NearToken::from_near(1)).build());
        contract.vote(poll_id, 0);

        let poll = contract.get_poll(poll_id).unwrap();
        assert_eq!(poll.votes[0], 1);
        assert_eq!(poll.votes[1], 0);

        let user_vote = contract.get_user_vote(poll_id, accounts(1));
        assert_eq!(user_vote, Some(0));
    }

    #[test]
    fn test_vote_multiple_users() {
        let mut context = get_context(accounts(0), NearToken::from_near(1));
        testing_env!(context.build());
        let mut contract = init_contract();

        let poll_id = contract.create_poll(
            "Multi-User Poll".to_string(),
            "Testing multiple voters".to_string(),
            None,
            vec![
                OptionInput { label: "A".to_string(), recipient: accounts(1) },
                OptionInput { label: "B".to_string(), recipient: accounts(2) },
                OptionInput { label: "C".to_string(), recipient: accounts(3) },
            ],
            None,
            true,
            Some(0),
            None,
        );

        // User 1 votes for option 0
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(NearToken::from_near(1)).build());
        contract.vote(poll_id, 0);

        // User 2 votes for option 1
        testing_env!(context.predecessor_account_id(accounts(2)).attached_deposit(NearToken::from_near(1)).build());
        contract.vote(poll_id, 1);

        // User 3 votes for option 0
        testing_env!(context.predecessor_account_id(accounts(3)).attached_deposit(NearToken::from_near(1)).build());
        contract.vote(poll_id, 0);

        // User 4 votes for option 2
        testing_env!(context.predecessor_account_id(accounts(4)).attached_deposit(NearToken::from_near(1)).build());
        contract.vote(poll_id, 2);

        let poll = contract.get_poll(poll_id).unwrap();
        assert_eq!(poll.votes[0], 2); // Users 1 and 3
        assert_eq!(poll.votes[1], 1); // User 2
        assert_eq!(poll.votes[2], 1); // User 4
    }

    #[test]
    fn test_vote_change() {
        let mut context = get_context(accounts(0), NearToken::from_near(1));
        testing_env!(context.build());
        let mut contract = init_contract();

        let poll_id = contract.create_poll(
            "Changeable Poll".to_string(),
            "Users can change votes".to_string(),
            None,
            vec![
                OptionInput { label: "Option 1".to_string(), recipient: accounts(1) },
                OptionInput { label: "Option 2".to_string(), recipient: accounts(2) },
            ],
            None,
            true,
            Some(0),
            None,
        );

        // Initial vote
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(NearToken::from_near(1)).build());
        contract.vote(poll_id, 0);

        let poll_after_first = contract.get_poll(poll_id).unwrap();
        assert_eq!(poll_after_first.votes[0], 1);
        assert_eq!(poll_after_first.votes[1], 0);

        // Change vote
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(NearToken::from_near(1)).build());
        contract.vote(poll_id, 1);

        let poll_after_change = contract.get_poll(poll_id).unwrap();
        assert_eq!(poll_after_change.votes[0], 0);
        assert_eq!(poll_after_change.votes[1], 1);

        let user_vote = contract.get_user_vote(poll_id, accounts(1));
        assert_eq!(user_vote, Some(1));
    }

    #[test]
    fn test_vote_same_option_twice() {
        let mut context = get_context(accounts(0), NearToken::from_near(1));
        testing_env!(context.build());
        let mut contract = init_contract();

        let poll_id = contract.create_poll(
            "Test Poll".to_string(),
            "Description".to_string(),
            None,
            vec![
                OptionInput { label: "A".to_string(), recipient: accounts(1) },
                OptionInput { label: "B".to_string(), recipient: accounts(2) },
            ],
            None,
            true,
            Some(0),
            None,
        );

        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(NearToken::from_near(1)).build());
        contract.vote(poll_id, 0);
        
        // Vote for same option again
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(NearToken::from_near(1)).build());
        contract.vote(poll_id, 0);

        let poll = contract.get_poll(poll_id).unwrap();
        assert_eq!(poll.votes[0], 1, "Voting for same option should not double-count");
    }

    #[test]
    #[should_panic(expected = "Poll not found")]
    fn test_vote_nonexistent_poll() {
        let context = get_context(accounts(0), NearToken::from_near(1));
        testing_env!(context.build());
        let mut contract = init_contract();

        contract.vote(999, 0);
    }

    #[test]
    #[should_panic(expected = "Invalid option index")]
    fn test_vote_invalid_option_index() {
        let mut context = get_context(accounts(0), NearToken::from_near(1));
        testing_env!(context.build());
        let mut contract = init_contract();

        let poll_id = contract.create_poll(
            "Test Poll".to_string(),
            "Description".to_string(),
            None,
            vec![
                OptionInput { label: "A".to_string(), recipient: accounts(1) },
                OptionInput { label: "B".to_string(), recipient: accounts(2) },
            ],
            None,
            true,
            Some(0),
            None,
        );

        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(NearToken::from_near(1)).build());
        contract.vote(poll_id, 2); // Only options 0 and 1 exist
    }

    #[test]
    #[should_panic(expected = "Poll is not active")]
    fn test_vote_on_closed_poll() {
        let mut context = get_context(accounts(0), NearToken::from_near(1));
        testing_env!(context.build());
        let mut contract = init_contract();

        let poll_id = contract.create_poll(
            "Test Poll".to_string(),
            "Description".to_string(),
            None,
            vec![
                OptionInput { label: "A".to_string(), recipient: accounts(1) },
                OptionInput { label: "B".to_string(), recipient: accounts(2) },
            ],
            None,
            true,
            Some(0),
            None,
        );

        // Close the poll
        contract.close_poll(poll_id);

        // Try to vote on closed poll
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(NearToken::from_near(1)).build());
        contract.vote(poll_id, 0);
    }

    #[test]
    #[should_panic(expected = "Poll has expired")]
    fn test_vote_on_expired_poll() {
        let mut context = get_context(accounts(0), NearToken::from_near(1));
        let start_time = 0u64;
        testing_env!(context.block_timestamp(start_time).build());
        let mut contract = init_contract();

        let duration_minutes = 60u64;
        let poll_id = contract.create_poll(
            "Expiring Poll".to_string(),
            "Will expire".to_string(),
            None,
            vec![
                OptionInput { label: "A".to_string(), recipient: accounts(1) },
                OptionInput { label: "B".to_string(), recipient: accounts(2) },
            ],
            Some(duration_minutes),
            true,
            Some(0),
            None,
        );

        // Fast forward past expiration
        let expired_time = (duration_minutes * 60 * 1_000_000_000) + 1;
        testing_env!(context.block_timestamp(expired_time).predecessor_account_id(accounts(1)).attached_deposit(NearToken::from_near(1)).build());
        contract.vote(poll_id, 0);
    }

    #[test]
    fn test_vote_just_before_expiration() {
        let mut context = get_context(accounts(0), NearToken::from_near(1));
        let start_time = 0u64;
        testing_env!(context.block_timestamp(start_time).build());
        let mut contract = init_contract();

        let duration_minutes = 60u64;
        let poll_id = contract.create_poll(
            "Expiring Poll".to_string(),
            "Will expire soon".to_string(),
            None,
            vec![
                OptionInput { label: "A".to_string(), recipient: accounts(1) },
                OptionInput { label: "B".to_string(), recipient: accounts(2) },
            ],
            Some(duration_minutes),
            true,
            Some(0),
            None,
        );

        // Vote just before expiration (1 nanosecond before)
        let almost_expired = (duration_minutes * 60 * 1_000_000_000) - 1;
        testing_env!(context.block_timestamp(almost_expired).predecessor_account_id(accounts(1)).attached_deposit(NearToken::from_near(1)).build());
        contract.vote(poll_id, 0);

        let poll = contract.get_poll(poll_id).unwrap();
        assert_eq!(poll.votes[0], 1, "Vote just before expiration should succeed");
    }

    // ========================================
    // Poll Closure Tests
    // ========================================

    #[test]
    fn test_close_poll_by_creator() {
        let mut context = get_context(accounts(0), NearToken::from_near(1));
        testing_env!(context.build());
        let mut contract = init_contract();

        let poll_id = contract.create_poll(
            "Closeable Poll".to_string(),
            "Can be closed".to_string(),
            None,
            vec![
                OptionInput { label: "A".to_string(), recipient: accounts(1) },
                OptionInput { label: "B".to_string(), recipient: accounts(2) },
            ],
            None,
            true,
            Some(0),
            None,
        );

        contract.close_poll(poll_id);

        let poll = contract.get_poll(poll_id).unwrap();
        assert!(!poll.is_active, "Poll should be inactive after closure");
    }

    #[test]
    #[should_panic(expected = "Only creator can close poll")]
    fn test_close_poll_by_non_creator() {
        let mut context = get_context(accounts(0), NearToken::from_near(1));
        testing_env!(context.build());
        let mut contract = init_contract();

        let poll_id = contract.create_poll(
            "Protected Poll".to_string(),
            "Only creator can close".to_string(),
            None,
            vec![
                OptionInput { label: "A".to_string(), recipient: accounts(1) },
                OptionInput { label: "B".to_string(), recipient: accounts(2) },
            ],
            None,
            true,
            Some(0),
            None,
        );

        // Different user tries to close
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        contract.close_poll(poll_id);
    }

    #[test]
    #[should_panic(expected = "Poll not found")]
    fn test_close_nonexistent_poll() {
        let context = get_context(accounts(0), NearToken::from_near(1));
        testing_env!(context.build());
        let mut contract = init_contract();

        contract.close_poll(999);
    }

    #[test]
    fn test_close_poll_preserves_votes() {
        let mut context = get_context(accounts(0), NearToken::from_near(1));
        testing_env!(context.build());
        let mut contract = init_contract();

        let poll_id = contract.create_poll(
            "Vote Preservation Test".to_string(),
            "Votes should persist".to_string(),
            None,
            vec![
                OptionInput { label: "A".to_string(), recipient: accounts(1) },
                OptionInput { label: "B".to_string(), recipient: accounts(2) },
            ],
            None,
            true,
            Some(0),
            None,
        );

        // Add some votes
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(NearToken::from_near(1)).build());
        contract.vote(poll_id, 0);
        testing_env!(context.predecessor_account_id(accounts(2)).attached_deposit(NearToken::from_near(1)).build());
        contract.vote(poll_id, 1);

        // Close poll
        testing_env!(context.predecessor_account_id(accounts(0)).build());
        contract.close_poll(poll_id);

        // Check votes are preserved
        let poll = contract.get_poll(poll_id).unwrap();
        assert_eq!(poll.votes[0], 1);
        assert_eq!(poll.votes[1], 1);
    }

    // ========================================
    // View Function Tests
    // ========================================

    #[test]
    fn test_get_poll_nonexistent() {
        let context = get_context(accounts(0), NearToken::from_yoctonear(0));
        testing_env!(context.build());
        let contract = init_contract();

        let result = contract.get_poll(999);
        assert!(result.is_none(), "Nonexistent poll should return None");
    }

    #[test]
    fn test_get_user_vote_nonexistent_poll() {
        let context = get_context(accounts(0), NearToken::from_yoctonear(0));
        testing_env!(context.build());
        let contract = init_contract();

        let result = contract.get_user_vote(999, accounts(1));
        assert!(result.is_none(), "Vote for nonexistent poll should return None");
    }

    #[test]
    fn test_get_user_vote_no_vote_cast() {
        let mut context = get_context(accounts(0), NearToken::from_near(1));
        testing_env!(context.build());
        let mut contract = init_contract();

        let poll_id = contract.create_poll(
            "Test Poll".to_string(),
            "Description".to_string(),
            None,
            vec![
                OptionInput { label: "A".to_string(), recipient: accounts(1) },
                OptionInput { label: "B".to_string(), recipient: accounts(2) },
            ],
            None,
            true,
            Some(0),
            None,
        );

        let result = contract.get_user_vote(poll_id, accounts(5));
        assert!(result.is_none(), "User who hasn't voted should return None");
    }

    // ========================================
    // Edge Case Tests
    // ========================================

    #[test]
    fn test_poll_with_maximum_reasonable_options() {
        let context = get_context(accounts(0), NearToken::from_near(10));
        testing_env!(context.build());
        let mut contract = init_contract();

        let mut options = Vec::new();
        for i in 0..100 {
            options.push(OptionInput { label: format!("Option {}", i), recipient: accounts(1) });
        }

        let poll_id = contract.create_poll(
            "Many Options Poll".to_string(),
            "Testing many options".to_string(),
            None,
            options,
            None,
            true,
            Some(0),
            None,
        );

        let poll = contract.get_poll(poll_id).unwrap();
        assert_eq!(poll.options.len(), 100);
        assert_eq!(poll.votes.len(), 100);
    }

    #[test]
    fn test_duration_maximum_allowed() {
        let mut context = get_context(accounts(0), NearToken::from_near(1));
        let start_time = 0u64;
        testing_env!(context.block_timestamp(start_time).build());
        let mut contract = init_contract();

        // Test with maximum allowed duration (1 year = 525,600 minutes)
        let poll_id = contract.create_poll(
            "Max Duration Poll".to_string(),
            "One year duration".to_string(),
            None,
            vec![
                OptionInput { label: "A".to_string(), recipient: accounts(1) },
                OptionInput { label: "B".to_string(), recipient: accounts(2) },
            ],
            Some(MAX_DURATION_MINUTES), // Exactly 1 year
            true,
            Some(0),
            None,
        );

        let poll = contract.get_poll(poll_id).unwrap();
        assert!(poll.ends_at.is_some(), "Poll with max duration should have end time");
    }

    #[test]
    #[should_panic(expected = "Duration exceeds maximum (1 year)")]
    fn test_duration_exceeds_maximum() {
        let context = get_context(accounts(0), NearToken::from_near(1));
        testing_env!(context.build());
        let mut contract = init_contract();

        // Try to create poll with duration exceeding 1 year
        contract.create_poll(
            "Too Long Poll".to_string(),
            "Duration too long".to_string(),
            None,
            vec![
                OptionInput { label: "A".to_string(), recipient: accounts(1) },
                OptionInput { label: "B".to_string(), recipient: accounts(2) },
            ],
            Some(MAX_DURATION_MINUTES + 1), // Over 1 year
            true,
            Some(0),
            None,
        );
    }

    #[test]
    #[should_panic(expected = "Too many options (max 100)")]
    fn test_create_poll_too_many_options() {
        let context = get_context(accounts(0), NearToken::from_near(10));
        testing_env!(context.build());
        let mut contract = init_contract();

        let mut options = Vec::new();
        for i in 0..101 {
            options.push(OptionInput { label: format!("Option {}", i), recipient: accounts(1) });
        }

        contract.create_poll(
            "Too Many Options".to_string(),
            "Exceeds limit".to_string(),
            None,
            options,
            None,
            true,
            Some(0),
            None,
        );
    }

    #[test]
    #[should_panic(expected = "Reward must be at least 0.1 NEAR")]
    fn test_create_poll_reward_too_small() {
        let context = get_context(accounts(0), NearToken::from_near(1));
        testing_env!(context.build());
        let mut contract = init_contract();

        contract.create_poll(
            "Small Reward Poll".to_string(),
            "Reward too small".to_string(),
            None,
            vec![
                OptionInput { label: "A".to_string(), recipient: accounts(1) },
                OptionInput { label: "B".to_string(), recipient: accounts(2) },
            ],
            None,
            true,
            Some(50_000_000_000_000_000_000_000), // 0.05 NEAR (below minimum)
            None,
        );
    }

    #[test]
    #[should_panic(expected = "Reward exceeds maximum (100M NEAR)")]
    fn test_create_poll_reward_too_large() {
        let reward = 100_000_001u128 * 1_000_000_000_000_000_000_000_000u128; // 100M + 1 NEAR
        let deposit = NearToken::from_yoctonear(reward + 1_000_000_000_000_000_000_000_000); // reward + extra for storage

        let context = get_context(accounts(0), deposit);
        testing_env!(context.build());
        let mut contract = init_contract();

        contract.create_poll(
            "Huge Reward Poll".to_string(),
            "Reward too large".to_string(),
            None,
            vec![
                OptionInput { label: "A".to_string(), recipient: accounts(1) },
                OptionInput { label: "B".to_string(), recipient: accounts(2) },
            ],
            None,
            true,
            Some(reward),
            None,
        );
    }

    #[test]
    fn test_unicode_in_options() {
        let context = get_context(accounts(0), NearToken::from_near(1));
        testing_env!(context.build());
        let mut contract = init_contract();

        let poll_id = contract.create_poll(
            "Unicode Poll 🗳️".to_string(),
            "Testing unicode characters".to_string(),
            None,
            vec![
                OptionInput { label: "Yes ✓".to_string(), recipient: accounts(1) },
                OptionInput { label: "No ✗".to_string(), recipient: accounts(2) },
                OptionInput { label: "Maybe 🤔".to_string(), recipient: accounts(3) },
            ],
            None,
            true,
            Some(0),
            None,
        );

        let poll = contract.get_poll(poll_id).unwrap();
        assert_eq!(poll.options[0].label, "Yes ✓");
        assert_eq!(poll.options[1].label, "No ✗");
        assert_eq!(poll.options[2].label, "Maybe 🤔");
    }

    #[test]
    fn test_special_characters_in_text() {
        let context = get_context(accounts(0), NearToken::from_near(1));
        testing_env!(context.build());
        let mut contract = init_contract();

        let poll_id = contract.create_poll(
            "Poll with 'quotes' and \"escapes\"".to_string(),
            "Testing <special> & {characters}".to_string(),
            None,
            vec![
                OptionInput { label: "Option #1".to_string(), recipient: accounts(1) },
                OptionInput { label: "Option @2".to_string(), recipient: accounts(2) },
            ],
            None,
            true,
            Some(0),
            None,
        );

        let poll = contract.get_poll(poll_id).unwrap();
        assert!(poll.title.contains("quotes"));
        assert_eq!(poll.criteria, "Testing <special> & {characters}");
        assert!(poll.description.is_none());
    }

    #[test]
    fn test_concurrent_voting_different_polls() {
        let mut context = get_context(accounts(0), NearToken::from_near(1));
        testing_env!(context.build());
        let mut contract = init_contract();

        // Create multiple polls
        let poll_id1 = contract.create_poll(
            "Poll 1".to_string(),
            "First poll".to_string(),
            None,
            vec![
                OptionInput { label: "A".to_string(), recipient: accounts(1) },
                OptionInput { label: "B".to_string(), recipient: accounts(2) },
            ],
            None,
            true,
            Some(0),
            None,
        );

        let poll_id2 = contract.create_poll(
            "Poll 2".to_string(),
            "Second poll".to_string(),
            None,
            vec![
                OptionInput { label: "X".to_string(), recipient: accounts(1) },
                OptionInput { label: "Y".to_string(), recipient: accounts(2) },
            ],
            None,
            true,
            Some(0),
            None,
        );

        // Same user votes on different polls
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(NearToken::from_near(1)).build());
        contract.vote(poll_id1, 0);
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(NearToken::from_near(1)).build());
        contract.vote(poll_id2, 1);

        // Verify votes are tracked separately
        let vote1 = contract.get_user_vote(poll_id1, accounts(1));
        let vote2 = contract.get_user_vote(poll_id2, accounts(1));
        
        assert_eq!(vote1, Some(0));
        assert_eq!(vote2, Some(1));

        let poll1 = contract.get_poll(poll_id1).unwrap();
        let poll2 = contract.get_poll(poll_id2).unwrap();
        
        assert_eq!(poll1.votes[0], 1);
        assert_eq!(poll2.votes[1], 1);
    }

    #[test]
    fn test_vote_count_accuracy_with_changes() {
        let mut context = get_context(accounts(0), NearToken::from_near(1));
        testing_env!(context.build());
        let mut contract = init_contract();

        let poll_id = contract.create_poll(
            "Accuracy Test".to_string(),
            "Testing vote count accuracy".to_string(),
            None,
            vec![
                OptionInput { label: "A".to_string(), recipient: accounts(1) },
                OptionInput { label: "B".to_string(), recipient: accounts(2) },
                OptionInput { label: "C".to_string(), recipient: accounts(3) },
            ],
            None,
            true,
            Some(0),
            None,
        );

        // Initial votes
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(NearToken::from_near(1)).build());
        contract.vote(poll_id, 0);
        testing_env!(context.predecessor_account_id(accounts(2)).attached_deposit(NearToken::from_near(1)).build());
        contract.vote(poll_id, 0);
        testing_env!(context.predecessor_account_id(accounts(3)).attached_deposit(NearToken::from_near(1)).build());
        contract.vote(poll_id, 1);

        let poll_mid = contract.get_poll(poll_id).unwrap();
        assert_eq!(poll_mid.votes[0], 2);
        assert_eq!(poll_mid.votes[1], 1);
        assert_eq!(poll_mid.votes[2], 0);

        // Change votes
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(NearToken::from_near(1)).build());
        contract.vote(poll_id, 2); // 0 -> 2
        testing_env!(context.predecessor_account_id(accounts(3)).attached_deposit(NearToken::from_near(1)).build());
        contract.vote(poll_id, 0); // 1 -> 0

        let poll_final = contract.get_poll(poll_id).unwrap();
        assert_eq!(poll_final.votes[0], 2); // accounts(2) and accounts(3)
        assert_eq!(poll_final.votes[1], 0); // None
        assert_eq!(poll_final.votes[2], 1); // accounts(1)
    }
}