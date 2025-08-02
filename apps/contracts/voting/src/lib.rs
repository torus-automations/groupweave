use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault, Promise};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Poll {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub options: Vec<String>,
    pub votes: Vec<u64>,
    pub creator: AccountId,
    pub is_active: bool,
    pub created_at: u64,
    pub ends_at: Option<u64>,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct VotingContract {
    polls: LookupMap<u64, Poll>,
    user_votes: LookupMap<(AccountId, u64), u64>, // (user, poll_id) -> option_index
    next_poll_id: u64,
}

#[near_bindgen]
impl VotingContract {
    #[init]
    pub fn new() -> Self {
        Self {
            polls: LookupMap::new(b"p"),
            user_votes: LookupMap::new(b"v"),
            next_poll_id: 1,
        }
    }

    pub fn create_poll(
        &mut self,
        title: String,
        description: String,
        options: Vec<String>,
        duration_minutes: Option<u64>,
    ) -> u64 {
        let poll_id = self.next_poll_id;
        let creator = env::predecessor_account_id();
        let created_at = env::block_timestamp();
        let ends_at = duration_minutes.map(|d| created_at + d * 60 * 1_000_000_000);
        
        let votes = vec![0; options.len()];
        
        let poll = Poll {
            id: poll_id,
            title,
            description,
            options,
            votes,
            creator,
            is_active: true,
            created_at,
            ends_at,
        };
        
        self.polls.insert(&poll_id, &poll);
        self.next_poll_id += 1;
        
        poll_id
    }

    pub fn vote(&mut self, poll_id: u64, option_index: u64) {
        let voter = env::predecessor_account_id();
        
        // Check if poll exists and is active
        let mut poll = self.polls.get(&poll_id).expect("Poll not found");
        assert!(poll.is_active, "Poll is not active");
        
        // Check if poll has expired
        if let Some(ends_at) = poll.ends_at {
            assert!(env::block_timestamp() < ends_at, "Poll has expired");
        }
        
        // Check if option index is valid
        assert!((option_index as usize) < poll.options.len(), "Invalid option index");
        
        // Check if user has already voted
        let vote_key = (voter.clone(), poll_id);
        if let Some(previous_vote) = self.user_votes.get(&vote_key) {
            // Remove previous vote
            poll.votes[previous_vote as usize] -= 1;
        }
        
        // Add new vote
        poll.votes[option_index as usize] += 1;
        self.user_votes.insert(&vote_key, &option_index);
        self.polls.insert(&poll_id, &poll);
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
        
        poll.is_active = false;
        self.polls.insert(&poll_id, &poll);
    }
}