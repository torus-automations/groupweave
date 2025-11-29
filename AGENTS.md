# Groupweave Project Context

## 1. Overview
Groupweave is a decentralized curation and automation platform built on NEAR Protocol, emphasizing **human-in-the-loop** processes for content creation and moderation. It consists of:
- **Smart Contracts:** Rust-based contracts for voting, deposits, and agent coordination.
- **Agents:** TypeScript-based autonomous agents (Curation, VLM Classifier) using `@neardefi/shade-agent-js`.

## 2. Architecture & Structure

### `/contracts`
NEAR smart contracts written in Rust.
- **`voting/`**: Main voting logic (polls, votes, whitelists).
- **`deposits/`**: Handling NEAR and FT deposits.
- **`content-bounty-market/`**: Bounty management.
- **`shade-*/`**: Agent coordination contracts.

### `/agents`
Autonomous agents written in TypeScript (Node.js/Bun).
- **`shade/curation-agent/`**: Handles content curation logic.
- **`shade/vlm-classifier-agent/`**: Visual Language Model for classification.

### `/scripts`
Deployment and utility scripts (Bash, TypeScript).

## 2.1. Economic & Architectural Rationale
Running large foundational models (e.g., Llama-3-70B, GPT-4-scale) on dedicated TEE hardware (H100/H200) is prohibitively expensive for individual user or community agents that need to be "always-on."

**Groupweave's Strategy:**
- **Small, Specialized Models:** We utilize efficient models like **Qwen3-VL-2B** and **Phi-3-mini** that can run on standard CPU TEEs (Intel TDX).
- **Cost Efficiency:** This drastically reduces the hourly cost per agent, making it viable for thousands of communities to deploy their own private curators.
- **Privacy First:** By baking the model into the container, we ensure zero data leakage. Data is processed locally in the TEE, not sent to a centralized API or shared GPU cluster.

## 3. Tech Stack
- **Blockchain:** NEAR Protocol (Rust SDK)
- **Runtime:** Node.js >= 20 / Bun
- **Language:** TypeScript, Rust
- **Testing:** Vitest (Agents), Cargo Test (Contracts)
- **Tools:** `near-api-js`, `zod`, `tsx`

## 4. Operational Commands

### Agents
Run these within specific agent directories (e.g., `agents/shade/curation-agent/`):
- **Install:** `bun install` (or `npm install`)
- **Dev:** `bun run dev` (Watches `src/server.ts`)
- **Build:** `bun run build` (TypeScript compilation)
- **Test:** `bun test` (Vitest)
- **Typecheck:** `bun run typecheck`

### Contracts
Run these within specific contract directories (e.g., `contracts/voting/`):
- **Test:** `cargo test`
- **Build:** `cargo build --target wasm32-unknown-unknown --release`
- **Lint:** `cargo clippy`

## 5. Coding Conventions
- **TypeScript:** Strict typing, functional patterns, Zod for validation.
- **Rust:** Idiomatic Rust, `near_bindgen` macros, unit tests in same file or `tests/` module.
- **Commits:** Use descriptive messages.

## 6. Key Files
- `NEAR_INTEGRATION_GUIDE.md`: Detailed guide on NEAR integration.
- `agents/shade/curation-agent/src/server.ts`: Entry point for curation agent.
- `contracts/voting/src/lib.rs`: Main voting contract logic.
