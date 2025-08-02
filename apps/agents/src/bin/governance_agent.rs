use anyhow::Result;
use clap::Parser;
use groupweave_agents::{
    agents::GovernanceAgent, AgentConfig, Agent, init_logging
};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use tracing::{info, error};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Agent ID
    #[arg(short, long, default_value = "governance-agent-1")]
    agent_id: String,

    /// Network to connect to
    #[arg(short, long, default_value = "testnet")]
    network: String,

    /// RPC endpoint
    #[arg(short, long, default_value = "https://rpc.testnet.near.org")]
    rpc_endpoint: String,

    /// Voting contract address
    #[arg(long)]
    voting_contract: Option<String>,

    /// Staking contract address
    #[arg(long)]
    staking_contract: Option<String>,

    /// Polling interval in seconds
    #[arg(short, long, default_value = "30")]
    polling_interval: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logging();
    let args = Args::parse();

    info!("Starting Governance Agent: {}", args.agent_id);

    let mut contract_addresses = HashMap::new();
    if let Some(voting) = args.voting_contract {
        contract_addresses.insert("voting".to_string(), voting);
    }
    if let Some(staking) = args.staking_contract {
        contract_addresses.insert("staking".to_string(), staking);
    }

    let config = AgentConfig {
        agent_id: args.agent_id.clone(),
        network: args.network,
        rpc_endpoint: args.rpc_endpoint,
        contract_addresses,
        private_key: None, // Should be loaded from environment or secure storage
        polling_interval: args.polling_interval,
    };

    let mut agent = GovernanceAgent::new();
    agent.initialize(config)?;

    info!("Governance Agent initialized successfully");

    // Main execution loop
    loop {
        match agent.execute() {
            Ok(actions) => {
                if !actions.is_empty() {
                    info!("Executed {} actions", actions.len());
                    for action in actions {
                        info!("Action: {:?}", action);
                    }
                }
            }
            Err(e) => {
                error!("Agent execution failed: {}", e);
            }
        }

        sleep(Duration::from_secs(args.polling_interval)).await;
    }
}