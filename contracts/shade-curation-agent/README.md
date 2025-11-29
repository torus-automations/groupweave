Shade Curation Agent Contract

Purpose
- Minimal custom contract used by a Shade Agent (running in a Phala TEE) to:
  - store dataset metadata (hash/URI) and a single bound community ID;
  - accept agent-signed logs of Q&A interactions for accounting/audit.

Build
```
cargo build -p shade-curation-agent --target wasm32-unknown-unknown --release
```

Deploy (near-cli-rs)
```
cargo install near-cli-rs
near contract deploy \
  --account-id <your.testnet> \
  --wasm-file target/wasm32-unknown-unknown/release/shade_curation_agent.wasm
```

Init
```
near contract call --account-id <your.testnet> \
  --contract-id <contract.testnet> \
  --method-name new \
  --args '{
    "owner_id": "<your.testnet>",
    "agent_account_id": "<agent.testnet>",
    "dataset_hash": "<sha256>",
    "dataset_uri": "ipfs://...",
    "community_id": "<exclusive-community-id>"
  }'
```

Notes
- Contract is bound to a single `community_id`. Use `set_community` (owner-only) to reassign.
- Only `agent_account_id` can call `log_interaction`.
- Store only hashes/digests on-chain. Never store private data or plaintext
  user prompts/answers.
