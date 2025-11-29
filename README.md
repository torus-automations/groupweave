# Groupweave

Groupweave is a decentralized platform for **content creation and curation**, powered by **human-in-the-loop Shade agents**. This repository contains the on-chain smart contracts and agent components that enable this ecosystem.

## Structure

- `contracts/`: NEAR smart contracts (Rust).
- `agents/shade/`: **Python-based** Shade agent components (Curation & Classification).
    *   *Note:* These agents use local, private models (Qwen3-VL for VLM, Phi-3 for Curation) running on CPU TEEs. This ensures data privacy and cost-effectiveness by avoiding expensive, always-on large GPU instances.
- `scripts/`: Utility scripts for deployment and management.

## Smart Contracts

### Core Contracts
*   **`contracts/voting/`**: The primary engine for decentralized decision-making.
    *   **Logic:** Handles poll creation, vote casting, whitelisting, and poll closure. Supports generic options and metadata, making it adaptable for various governance or curation tasks.
*   **`contracts/deposits/`**: A flexible payment handler.
    *   **Logic:** Accepts deposits in both native NEAR and Fungible Tokens (FTs). It serves as the on-chain anchor for crediting user accounts in the off-chain platform, bridging the gap between blockchain assets and platform credits.
*   **`contracts/content-bounty-market/`**: A competitive marketplace for content creation.
    *   **Logic:** Users create bounties with a base prize. Creators submit content. The community "stakes" NEAR on their favorite submissions to vote.
    *   **Incentives:** The winning submission takes the base prize. Stakers on the winner share a portion of the total pool (incentivizing good curation), while the platform takes a small fee. Includes economic defenses against self-staking attacks.

### Agent Coordination
*   **`contracts/shade-curation-agent/`**: On-chain logic for the autonomous curation agent, managing task assignment and verification. The off-chain Python agent (Phi-3) performs **context-aware RAG** over private community data.
*   **`contracts/shade-classifier-agent/`**: On-chain logic for the Visual Language Model (VLM) classifier agent. The off-chain Python agent (Qwen3-VL) performs **context-aware image classification** using private community data.

### Utility & Templates
*   **`contracts/staking/`**: A generic time-based staking template. Allows users to stake NEAR to earn rewards over time. Currently serves as a foundation for future governance or "stake-to-access" features.
*   **`contracts/state-clearer/`**: A critical dev-tool. It contains no state logic and provides a `clear_all_state` method.
    *   **Usage:** Deploy this to an account to wipe its storage if a contract upgrade fails due to state corruption (serialization mismatches).
*   **`contracts/deposits-minimal/` & `contracts/deposits-simple/`**: Simplified reference implementations of the deposits logic, primarily used for testing and distinct feature isolation.

## Oracle & Price Feeds

The **Deposits** contract (`contracts/deposits/`) relies on an external price feed to calculate credit allocations.
*   **Security:** The contract enforces a strict **1-hour staleness check**. If the price hasn't been updated within 1 hour, all deposits are rejected to prevent arbitrage during crashes.
*   **Keeper Bot:** In production, a "Keeper" bot (cron job) must call `update_token_price` every 10-30 minutes.
## Frontend Integration

### Recommended Structure
For consuming applications (e.g., React), we recommend the following structure:
```
src/
├── lib/near/
│   ├── wallet-provider.ts       # Wallet initialization & selector setup
│   ├── contract-calls.ts        # Typed voting contract methods
│   ├── deposit-calls.ts         # Deposit & transaction verification
│   └── amounts.ts               # YoctoNEAR conversion utilities
├── hooks/
│   └── use-near-wallet.ts       # React hook for wallet state
└── components/
    └── NearWalletButton.tsx     # UI: Connect button, status, guard
```

### Installation
```bash
npm i near-api-js@6.4.0 @near-js/providers @near-js/accounts
npm i @near-wallet-selector/core @near-wallet-selector/modal-ui @near-wallet-selector/my-near-wallet
```

### Environment Configuration
```bash
NEAR_NETWORK_ID="testnet" # or "mainnet"
NEAR_VOTING_CONTRACT_ID="voting.groupweave.testnet"
NEAR_DEPOSIT_CONTRACT_ID="deposits.groupweave.testnet"
NEAR_RPC_URL="https://rpc.testnet.near.org"
```

### Integration Examples

**1. Create a Poll:**
```typescript
import { createPoll } from '@/lib/near/contract-calls'

const result = await createPoll(selector, contractId, {
  title: 'Poll Title',
  criteria: 'Voting Criteria',
  options: [{ label: 'Opt A', recipient: 'user1.near' }, { label: 'Opt B', recipient: 'user2.near' }],
  durationMinutes: 1440,
  isPublic: true,
  rewardYocto: '5000000000000000000000000' // 5 NEAR
});
```

**2. Cast Vote:**
```typescript
import { vote } from '@/lib/near/contract-calls'
await vote(selector, contractId, { pollId: 1, optionIndex: 0 });
```

**3. Verify Deposit:**
```typescript
import { verifyDepositTransaction } from '@/lib/near/deposit-calls'

const result = await verifyDepositTransaction('testnet', txHash, accountId);
if (result.verified) {
  console.log(`Confirmed deposit of ${result.amount} yoctoNEAR`);
}
```
