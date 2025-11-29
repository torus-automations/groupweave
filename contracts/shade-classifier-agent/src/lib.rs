//! Shade Classifier Agent – user-owned, exclusive per user, logs classification events
//! with optional human-in-the-loop review.

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::store::UnorderedMap;
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault, Promise};
use serde::{Deserialize, Serialize};

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ClassifyLog {
    pub session_id: String,
    pub image_hash: String,    // sha256 of image bytes or canonical url
    pub prompt_hash: String,   // hash of prompt/instructions
    pub label: String,
    pub confidence_bps: u32,   // 0..10000 basis points
    pub model: String,         // model identifier
    pub created_at_ns: u64,
    pub reviewed: bool,
    pub final_label: Option<String>,
    pub reviewer: Option<String>,
    pub reviewed_at_ns: Option<u64>,
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct Contract {
    pub owner_id: AccountId,
    pub agent_account_id: AccountId,
    pub model_kind: String, // LLM | VLM
    pub logs: UnorderedMap<String, ClassifyLog>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId, agent_account_id: AccountId, model_kind: String) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Self { owner_id, agent_account_id, model_kind, logs: UnorderedMap::new(b"cl".to_vec()) }
    }

    pub fn set_agent_account(&mut self, agent_account_id: AccountId) {
        self.assert_owner();
        self.agent_account_id = agent_account_id;
    }

    // Agent-only: log classification
    pub fn log_classification(
        &mut self,
        session_id: String,
        image_hash: String,
        prompt_hash: String,
        label: String,
        confidence_bps: u32,
        model: String,
    ) {
        self.assert_agent();
        let before = env::storage_usage();
        let log = ClassifyLog {
            session_id: session_id.clone(),
            image_hash,
            prompt_hash,
            label,
            confidence_bps,
            model,
            created_at_ns: env::block_timestamp(),
            reviewed: false,
            final_label: None,
            reviewer: None,
            reviewed_at_ns: None,
        };
        self.logs.insert(session_id.clone(), log);

        let after = env::storage_usage();
        if after > before {
            let delta = u128::from(after - before);
            let required: u128 = delta * env::storage_byte_cost().as_yoctonear();
            let deposit: u128 = env::attached_deposit().as_yoctonear();
            assert!(deposit >= required, "insufficient deposit for storage");
            let refund = deposit - required;
            if refund > 0 { Promise::new(env::predecessor_account_id()).transfer(near_sdk::NearToken::from_yoctonear(refund)); }
        }
    }

    // Owner-only: record human review result (accept or override)
    pub fn record_review(&mut self, session_id: String, final_label: String) {
        self.assert_owner();
        let mut log = self.logs.get(&session_id).expect("session not found").clone();
        log.reviewed = true;
        log.final_label = Some(final_label);
        log.reviewer = Some(env::predecessor_account_id().to_string());
        log.reviewed_at_ns = Some(env::block_timestamp());
        self.logs.insert(session_id, log);
    }

    // Views
    pub fn get_classification(&self, session_id: String) -> Option<ClassifyLog> { self.logs.get(&session_id).cloned() }

    // Guards
    fn assert_owner(&self) {
        assert_eq!(env::predecessor_account_id(), self.owner_id, "owner only");
    }
    fn assert_agent(&self) {
        assert_eq!(env::predecessor_account_id(), self.agent_account_id, "agent only");
    }
    // Views for owner and model kind
    pub fn get_owner_id(&self) -> AccountId { self.owner_id.clone() }
    pub fn get_model_kind(&self) -> String { self.model_kind.clone() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::testing_env;

    fn set_predecessor(predecessor: &str) {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(predecessor.parse().unwrap());
        testing_env!(builder.build());
    }

    fn set_actor_with_deposit(predecessor: &str, deposit: u128) {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(predecessor.parse().unwrap());
        builder.attached_deposit(near_sdk::NearToken::from_yoctonear(deposit));
        testing_env!(builder.build());
    }

    #[test]
    fn init_and_log_review() {
        set_predecessor("owner.testnet");
        let mut c = Contract::new(
            "owner.testnet".parse().unwrap(),
            "agent.testnet".parse().unwrap(),
            "VLM".into(),
        );
        set_actor_with_deposit("agent.testnet", 10_000_000_000_000_000_000_000); // 0.01 NEAR
        c.log_classification("s1".into(), "ihash".into(), "phash".into(), "cat".into(), 8123, "gpt-4o".into());
        set_predecessor("owner.testnet");
        c.record_review("s1".into(), "cat".to_string());
        let log = c.get_classification("s1".into()).unwrap();
        assert!(log.reviewed);
        assert_eq!(log.final_label.unwrap(), "cat");
    }
}
