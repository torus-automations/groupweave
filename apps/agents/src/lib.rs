use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn, error};

pub mod agents;
pub mod blockchain;
pub mod governance;
pub mod rewards;
pub mod utils;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub agent_id: String,
    pub network: String,
    pub rpc_endpoint: String,
    pub contract_addresses: HashMap<String, String>,
    pub private_key: Option<String>,
    pub polling_interval: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingData {
    pub poll_id: u64,
    pub title: String,
    pub options: Vec<String>,
    pub votes: Vec<u64>,
    pub total_votes: u64,
    pub is_active: bool,
    pub created_at: u64,
    pub ends_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingData {
    pub user: String,
    pub amount: u128,
    pub rewards: u128,
    pub staked_at: u64,
    pub last_claim: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceProposal {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub proposer: String,
    pub votes_for: u64,
    pub votes_against: u64,
    pub status: ProposalStatus,
    pub created_at: u64,
    pub voting_ends_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Executed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAction {
    pub action_type: ActionType,
    pub target: String,
    pub data: serde_json::Value,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    VoteAggregation,
    RewardDistribution,
    GovernanceExecution,
    DataSync,
    SecurityCheck,
}

pub trait Agent {
    fn initialize(&mut self, config: AgentConfig) -> Result<()>;
    fn execute(&mut self) -> Result<Vec<AgentAction>>;
    fn get_status(&self) -> AgentStatus;
    fn shutdown(&mut self) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatus {
    pub agent_id: String,
    pub is_running: bool,
    pub last_execution: Option<u64>,
    pub actions_performed: u64,
    pub errors: Vec<String>,
}

pub fn init_logging() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
}

pub fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}