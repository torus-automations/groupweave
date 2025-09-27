# NEAR Smart Contract for Bounty Prediction Market

This workspace contains the NEAR smart contract for the GroupWeave platform's bounty prediction market.

## Structure

```
contracts/
├── Cargo.toml                      # Workspace manifest
└── bounty-prediction-market/       # 💸 Bounty Prediction Market Contract
    ├── src/lib.rs
    └── Cargo.toml
```

## Bounty Prediction Market Contract

This contract manages bounties, staking on bounty options, and reward distribution. It combines the functionalities of staking and prediction markets into a single, efficient contract.

### Key Features:
- **Bounty Creation**: Users can create bounties with a title, description, multiple options, a maximum stake amount per user, and a duration.
- **Staking on Options**: Users can stake NEAR tokens on the bounty option they predict will win.
- **Reward Distribution**: When a bounty is closed, the contract determines the winning option based on the total stake for each option. The prize pool (total staked amount minus a platform fee) is distributed proportionally among the users who staked on the winning option.
- **Owner Controls**: The contract owner can manage the platform fee rate, maximum stake amounts, and pause the contract in case of emergencies.
- **Migration Support**: The contract includes a migration function to handle state transitions during upgrades.

## Development

### Prerequisites
- Rust 1.70+
- NEAR CLI
- cargo-near

### Building
```bash
# Build the contract
cargo build -p bounty-prediction-market --release
```

### Testing
```bash
# Run all tests
cargo test -p bounty-prediction-market
```

### Deployment
```bash
# Deploy the contract
near deploy --wasmFile target/wasm32-unknown-unknown/release/bounty_prediction_market.wasm --accountId your-contract.testnet --initFunction new --initArgs '{"reward_rate": "10", "min_stake_amount": "1000000000000000000000000", "max_stake_amount": "100000000000000000000000000"}'
```

## Usage Examples

### Create a Bounty
```bash
# Create a new bounty
near call your-contract.testnet create_bounty '{"title": "Next Big Feature", "description": "What feature should we build next?", "options": ["Feature X", "Feature Y"], "max_stake_per_user": "10000000000000000000000000", "duration_blocks": 86400}' --accountId your-account.testnet
```

### Stake on a Bounty Option
```bash
# Stake 5 NEAR on the first option of bounty with ID 1
near call your-contract.testnet stake_on_option '{"bounty_id": 1, "option_index": 0}' --accountId your-account.testnet --deposit 5
```

### Get Bounty Information
```bash
# Get details for a specific bounty
near view your-contract.testnet get_bounty '{"bounty_id": 1}'
```

### Get User's Stake Information
```bash
# Get a user's stake information for a specific bounty
near view your-contract.testnet get_participant_stake '{"account_id": "your-account.testnet", "bounty_id": 1}'
```

### Close a Bounty (Owner only)
```bash
# Close a bounty after it has expired
near call your-contract.testnet close_bounty '{"bounty_id": 1}' --accountId your-contract.testnet
```

### Get Bounty Results
```bash
# Get the results of a closed bounty
near view your-contract.testnet get_bounty_results '{"bounty_id": 1}'
```