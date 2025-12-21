# NEAR Smart Contracts

This workspace contains the NEAR smart contracts for the Dreamweave platform.

## Structure

```
contracts/
├── Cargo.toml                 # Workspace manifest
├── deposits/                  # Main Deposit Contract
│   ├── src/lib.rs
│   └── Cargo.toml
├── content-bounty-market/     # Content Bounty Prediction Market
│   ├── src/lib.rs
│   ├── Cargo.toml
│   └── README.md             # Detailed documentation
├── voting/                    # Curation Voting Contract
│   ├── src/lib.rs
│   └── Cargo.toml
├── staking/                   # Staking Contract
│   ├── src/lib.rs
│   └── Cargo.toml
├── shade-curation-agent/      # Agent Coordination Contract
│   ├── src/lib.rs
│   └── Cargo.toml
├── shade-classifier-agent/    # Agent Coordination Contract
│   ├── src/lib.rs
│   └── Cargo.toml
```

## Contracts

### Deposit Contract
**Location:** `deposits/`  
**Status:** Active

Smart escrow for NEAR-native and fungible token deposits:
- Accepts native NEAR via `deposit_native` with configurable minimums (default $5 USD equivalent)
- Handles NEAR fungible tokens (FT) via `ft_transfer_call` standard
- Maintains on-chain USD price oracle for supported tokens
- Emits structured `EVENT_JSON` logs for off-chain credit reconciliation
- Auto-forwards native NEAR to treasury account
- Owner-only FT withdrawal to treasury (`withdraw_ft`)
- Supports multiple tokens with per-token configuration

**Key Methods:**
- `deposit_native` – Deposit NEAR with beneficiary ID and credit hint
- `ft_on_transfer` – Receive FT deposits via NEP-141 standard
- `upsert_token_config` – Configure supported tokens (owner only)
- `update_token_price` – Update USD price oracle (owner only)
- `get_deposit_record` – Query deposit history
- `get_token_config` – View token configuration

### Content Bounty Market Contract
**Location:** `content-bounty-market/`  
**Status:** Active  
**Docs:** See `content-bounty-market/README.md`

Decentralized content creation competitions with community staking:
- Bounty creators post challenges with base prize and requirements
- Content creators submit their work (links to Dreamweave creations)
- Community members stake NEAR on submissions they believe should win
- Winner determined by most NEAR staked
- Configurable reward splits (default: 90% creator, 10% backers, 5% platform fee)
- Supports up to 100 submissions per bounty
- Time-limited bounties (1-365 days)

**Key Methods:**
- `create_content_bounty` – Start a new bounty with base prize
- `submit_content` – Submit creation to bounty
- `stake_on_submission` – Stake NEAR on a submission
- `close_bounty` – Finalize and distribute rewards (creator/owner only)
- `get_bounty` – View bounty details and submissions
- `get_active_bounties` – List all active bounties

### Voting Contract
**Location:** `voting/`  
**Status:** Active

Curation voting for community content:
- Create polls with multiple options
- Vote on active polls
- Time-limited voting periods
- Vote changing capability
- Results tracking

### Staking Contract
**Location:** `staking/`  
**Status:** Active

Token staking with rewards:
- Stake NEAR tokens within configurable limits
- Minimum and maximum stake amounts
- Earn rewards over time based on configured rate
- Unstake with automatic reward claims
- Admin-configurable reward rates and limits

## Development

### Prerequisites
- Rust 1.70+ with `wasm32-unknown-unknown` target
- NEAR CLI v4+ (near-cli-rs recommended)
- cargo-near for builds and deployment

### Building
```bash
# Build all contracts in workspace
cargo build --target wasm32-unknown-unknown --release

# Build specific contract
cd deposits
cargo build --target wasm32-unknown-unknown --release

# Output location
# target/wasm32-unknown-unknown/release/{contract_name}.wasm
```

### Testing
```bash
# Run all workspace tests
cargo test

# Test specific contract
cd deposits
cargo test

# Test with output
cargo test -- --nocapture
```

### Deployment
```bash
# Deploy voting contract
cargo near deploy your-contract.testnet without-init-call network-config testnet sign-with-keychain send

# Deploy staking contract
cargo near deploy your-staking.testnet with-init-call new json-args '{"reward_rate": "10", "min_stake_amount": "1000000000000000000000000", "max_stake_amount": "100000000000000000000000000"}' prepaid-gas '100.0 Tgas' attached-deposit '0 NEAR' network-config testnet sign-with-keychain send

# Deploy deposit contract (example owner + treasury)
cargo near deploy deposits.testnet with-init-call new json-args '{"owner_id":"YOUR_ACCOUNT.testnet","treasury_account_id":"treasury.testnet"}' prepaid-gas '100.0 Tgas' attached-deposit '0 NEAR' network-config testnet sign-with-keychain send
  --accountId deposits.your-account.testnet \
  --initFunction new \
  --initArgs '{"owner_id":"your-account.testnet","treasury_account_id":"treasury.testnet"}'

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


### Deposit Contract
```bash
# Register NEAR token price (owner)
near call deposits.your-account.testnet update_token_price '{"token_id":"NEAR","price_usd_micros":"4500000"}' --accountId your-account.testnet

# Register USDT on testnet (owner)
near call deposits.your-account.testnet upsert_token_config '{"token_id":"usdt.tether-token.near","symbol":"USDT","decimals":6,"price_usd_micros":"1000000","is_enabled":true,"is_native":false}' --accountId your-account.testnet

# Deposit 6 NEAR (user)
near call deposits.your-account.testnet deposit_native '{"beneficiary_id":"user-uuid","credits_hint":600}' --deposit 6 --accountId alice.testnet

# Deposit 10 USDT via ft_transfer_call (user)
near call usdt.tether-token.near ft_transfer_call '{"receiver_id":"deposits.your-account.testnet","amount":"10000000","memo":"credit top-up","msg":"{\"beneficiary_id\":\"user-uuid\",\"credits_hint\":1000}"}' --accountId alice.testnet --depositYocto 1

# Withdraw accumulated USDT to the treasury (owner)
near call deposits.your-account.testnet withdraw_ft '{"token_id":"usdt.tether-token.near","amount":"5000000"}' --accountId your-account.testnet --depositYocto 1
```

---

**Last Updated:** 2025-11-29
