# NEAR Smart Contracts

This workspace contains the NEAR smart contracts for the GroupWeave platform.

## Structure

```
contracts/
├── Cargo.toml          # Workspace manifest
├── voting/             # 🗳️ Voting Contract
│   ├── src/lib.rs
│   └── Cargo.toml
├── staking/            # 💰 Staking Contract
│   ├── src/lib.rs
│   └── Cargo.toml
└── zkp-verifier/       # 🔒 ZKP Verifier Contract
    ├── src/lib.rs
    └── Cargo.toml
```

## Contracts

### Voting Contract
Handles decentralized voting and polling functionality:
- Create polls with multiple options
- Vote on active polls
- Track voting results
- Time-limited polls
- Vote changing capability

### Staking Contract
Manages token staking and rewards with flexible betting amounts:
- Stake any amount of NEAR tokens within the contract's defined limits.
- A minimum and maximum stake amount can be configured.
- Earn rewards over time.
- Unstake with reward claims.
- Configurable reward rates and staking limits.

### ZKP Verifier Contract
Zero-knowledge proof verification:
- Submit ZK proofs
- Verify proofs with authorized verifiers
- Store verification results
- Simple hash-based proof verification

## Development

### Prerequisites
- Rust 1.70+
- NEAR CLI
- cargo-near

### Building
```bash
# Build all contracts
cargo build --release

# Build specific contract
cargo build -p voting-contract --release
```

### Testing
```bash
# Run all tests
cargo test

# Test specific contract
cargo test -p voting-contract
```

### Deployment
```bash
# Deploy voting contract
near deploy --wasmFile target/wasm32-unknown-unknown/release/voting_contract.wasm --accountId your-contract.testnet

# Deploy staking contract
near deploy --wasmFile target/wasm32-unknown-unknown/release/staking_contract.wasm --accountId your-staking.testnet --initFunction new --initArgs '{"reward_rate": "10", "min_stake_amount": "1000000000000000000000000", "max_stake_amount": "100000000000000000000000000"}'

# Deploy ZKP verifier contract
near deploy --wasmFile target/wasm32-unknown-unknown/release/zkp_verifier_contract.wasm --accountId your-zkp.testnet
```

## Usage Examples

### Voting Contract
```bash
# Create a poll
near call your-contract.testnet create_poll '{"title": "Best Feature", "description": "Vote for the best feature", "options": ["Feature A", "Feature B", "Feature C"], "duration_minutes": 1440}' --accountId your-account.testnet

# Vote on a poll
near call your-contract.testnet vote '{"poll_id": 1, "option_index": 0}' --accountId your-account.testnet

# Get poll results
near view your-contract.testnet get_poll '{"poll_id": 1}'
```

### Staking Contract
```bash
# Stake tokens (e.g., 10 NEAR)
near call your-staking.testnet stake --deposit 10 --accountId your-account.testnet

# Check stake info
near view your-staking.testnet get_stake_info '{"account": "your-account.testnet"}'

# Claim rewards
near call your-staking.testnet claim_rewards --accountId your-account.testnet

# Update max stake amount (owner only)
near call your-staking.testnet update_max_stake_amount '{"new_max_amount": "200000000000000000000000000"}' --accountId your-staking.testnet
```

### ZKP Verifier Contract
```bash
# Submit a proof
near call your-zkp.testnet submit_proof '{"proof_id": "proof1", "proof_data": "base64_proof", "public_inputs": ["input1"], "verification_key": "vk_data"}' --accountId your-account.testnet

# Verify a proof (authorized verifier only)
near call your-zkp.testnet verify_proof '{"proof_id": "proof1", "is_valid": true}' --accountId verifier.testnet
```