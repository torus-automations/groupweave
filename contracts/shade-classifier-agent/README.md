Shade Classifier Agent Contract (User-Owned)

Purpose
- Exclusive to a single user; logs classification results from a VLM/LLM agent and supports human-in-the-loop review.

Interface
- `new(owner_id, agent_account_id, model_kind)`
- `set_agent_account(..)` (owner-only)
- `log_classification(session_id, image_hash, prompt_hash, label, confidence_bps, model)` (agent-only)
- `record_review(session_id, final_label)` (owner-only)
- `get_classification(session_id)` (view)

Build
```
cargo build -p shade-classifier-agent --target wasm32-unknown-unknown --release
```

Deploy (near-cli-rs)
```
cargo install near-cli-rs
near contract deploy \
  --account-id <your.testnet> \
  --wasm-file target/wasm32-unknown-unknown/release/shade_classifier_agent.wasm

near contract call --account-id <your.testnet> \
  --contract-id <contract.testnet> \
  --method-name new \
  --args '{
    "owner_id":"<your.testnet>",
    "agent_account_id":"<agent.testnet>",
    "model_kind":"VLM"
  }'
```

