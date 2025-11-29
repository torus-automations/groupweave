# Build
- Agents (Docker): `cd agents/shade/<agent-name> && ./scripts/deploy.sh`
- Contracts (Wasm): `cd contracts/voting && cargo build --target wasm32-unknown-unknown --release`

# Install
- Agents: `cd agents/shade/<agent-name> && pip install -r requirements.txt`

# Run
- Agents (dev): `cd agents/shade/<agent-name> && python src/main.py`
- Agents (sim): `cd agents/shade/<agent-name> && docker-compose up --build`

# Test
- Contracts: `cd contracts/voting && cargo test`

# Code Style
- Python: Pydantic for validation, FastAPI for serving. Use `fetchWithRetry` logic for any external calls.
- Rust: Idiomatic NEAR SDK patterns. **Security critical:** prevent reentrancy and overflows.