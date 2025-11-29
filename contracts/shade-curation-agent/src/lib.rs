//! Shade Curation Agent – Custom NEAR contract for logging agent interactions
//! and storing minimal metadata/guardrails for community-curation assistance.
//!
//! This contract intentionally keeps on-chain state minimal. The private data
//! and LLM remain inside the Shade agent (TEE on Phala Cloud). The contract:
//! - Stores owner and the single allowed `agent_account_id` (the Shade agent's NEAR account).
//! - Stores dataset metadata (hash/uri) and a small allowlist of community IDs.
//! - Allows the agent to log interaction digests for audit and cost accounting.

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault, Promise};
use serde::{Deserialize, Serialize};

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct DatasetMeta {
    pub dataset_hash: String, // e.g., SHA256 of a tarball or manifest
    pub dataset_uri: String,  // off-chain reference (ipfs://, https://, etc.)
    pub updated_at_ns: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct InteractionLog {
    pub session_id: String,       // caller-provided ULID/UUID
    pub query_hash: String,       // hash of user query (not plaintext)
    pub answer_hash: String,      // hash of final answer (not plaintext)
    pub cost_microusd: u64,       // approx cost in micro-USD for accounting
    pub community_id: Option<String>,
    pub created_at_ns: u64,
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct Contract {
    pub owner_id: AccountId,
    pub agent_account_id: AccountId,
    pub dataset: DatasetMeta,
    pub community_id: String, // exclusive community assignment
    pub logs: UnorderedMap<String, InteractionLog>, // keyed by session_id
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        owner_id: AccountId,
        agent_account_id: AccountId,
        dataset_hash: String,
        dataset_uri: String,
        community_id: String,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");

        let dataset = DatasetMeta {
            dataset_hash,
            dataset_uri,
            updated_at_ns: env::block_timestamp(),
        };

        Self { owner_id, agent_account_id, dataset, community_id, logs: UnorderedMap::new(b"l".to_vec()) }
    }

    // Owner-only config
    pub fn set_agent_account(&mut self, agent_account_id: AccountId) {
        self.assert_owner();
        self.agent_account_id = agent_account_id;
    }

    pub fn set_dataset_meta(&mut self, dataset_hash: String, dataset_uri: String) {
        self.assert_owner();
        self.dataset = DatasetMeta { dataset_hash, dataset_uri, updated_at_ns: env::block_timestamp() };
    }

    pub fn set_community(&mut self, community_id: String) {
        self.assert_owner();
        self.community_id = community_id;
    }

    // Agent-only logging
    pub fn log_interaction(
        &mut self,
        session_id: String,
        query_hash: String,
        answer_hash: String,
        cost_microusd: u64,
        community_id: Option<String>,
    ) {
        self.assert_agent();

        if let Some(cid) = &community_id {
            assert!(cid == &self.community_id, "community mismatch");
        }

        let before = env::storage_usage();

        let log = InteractionLog {
            session_id: session_id.clone(),
            query_hash,
            answer_hash,
            cost_microusd,
            community_id,
            created_at_ns: env::block_timestamp(),
        };
        self.logs.insert(&session_id, &log);

        // Storage cost handling: require attached deposit >= delta * cost, refund extra
        let after = env::storage_usage();
        if after > before {
            let delta = u128::from(after - before);
            let required: u128 = delta * env::storage_byte_cost().as_yoctonear();
            let deposit: u128 = env::attached_deposit().as_yoctonear();
            assert!(deposit >= required, "insufficient deposit for storage");
            let refund = deposit - required;
            if refund > 0 {
                Promise::new(env::predecessor_account_id()).transfer(near_sdk::NearToken::from_yoctonear(refund));
            }
        }
    }

    // Views
    pub fn get_dataset_meta(&self) -> DatasetMeta { self.dataset.clone() }

    pub fn get_community_id(&self) -> String { self.community_id.clone() }

    pub fn get_interaction(&self, session_id: String) -> Option<InteractionLog> { self.logs.get(&session_id) }

    // Internal guards
    fn assert_owner(&self) {
        assert_eq!(env::predecessor_account_id(), self.owner_id, "owner only");
    }

    fn assert_agent(&self) {
        assert_eq!(env::predecessor_account_id(), self.agent_account_id, "agent only");
    }
}

// Unit tests (basic)
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
    fn init_and_views() {
        set_predecessor("owner.testnet");
        let c = Contract::new(
            "owner.testnet".parse().unwrap(),
            "agent.testnet".parse().unwrap(),
            "hash".into(),
            "uri".into(),
            "dw".into(),
        );
        let ds = c.get_dataset_meta();
        assert_eq!(ds.dataset_hash, "hash");
    }

    #[test]
    fn owner_sets_agent_and_allowlist() {
        set_predecessor("owner.testnet");
        let mut c = Contract::new(
            "owner.testnet".parse().unwrap(),
            "agent.testnet".parse().unwrap(),
            "h".into(),
            "u".into(),
            "dw".into(),
        );
        c.set_agent_account("agent2.testnet".parse().unwrap());
        c.set_community("dw-community".into());
        assert_eq!(c.get_community_id(), "dw-community");
    }

    #[test]
    fn agent_logs_interaction() {
        // init
        set_predecessor("owner.testnet");
        let mut c = Contract::new(
            "owner.testnet".parse().unwrap(),
            "agent.testnet".parse().unwrap(),
            "h".into(),
            "u".into(),
            "dw".into(),
        );

        // agent call
        set_actor_with_deposit("agent.testnet", 10_000_000_000_000_000_000_000); // 0.01 NEAR
        c.log_interaction("s1".into(), "q".into(), "a".into(), 1234, Some("dw".into()));
        let l = c.get_interaction("s1".into()).unwrap();
        assert_eq!(l.session_id, "s1");
    }

    #[test]
    #[should_panic]
    fn agent_logs_with_mismatched_community_panics() {
        set_predecessor("owner.testnet");
        let mut c = Contract::new(
            "owner.testnet".parse().unwrap(),
            "agent.testnet".parse().unwrap(),
            "h".into(),
            "u".into(),
            "dw".into(),
        );
        set_actor_with_deposit("agent.testnet", 10_000_000_000_000_000_000_000); // 0.01 NEAR
        c.log_interaction("s2".into(), "q".into(), "a".into(), 0, Some("other".into()));
    }
}
