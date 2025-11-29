#!/usr/bin/env bash
set -euo pipefail

# Usage: scripts/near/deploy-voting.sh <account-id> [network]
# Example (testnet): scripts/near/deploy-voting.sh voting.your.testnet testnet

ACCOUNT_ID=${1:-}
NETWORK=${2:-testnet}

if [[ -z "$ACCOUNT_ID" ]]; then
  echo "Usage: $0 <account-id> [network]" >&2
  exit 1
fi

echo "Building voting contract (wasm32-unknown-unknown, release)…"
pushd contracts >/dev/null
cargo build -p voting-contract --target wasm32-unknown-unknown --release
popd >/dev/null

WASM=contracts/target/wasm32-unknown-unknown/release/voting_contract.wasm
if [[ ! -f "$WASM" ]]; then
  echo "WASM not found at $WASM" >&2
  exit 1
fi

echo "Deploying to $ACCOUNT_ID on $NETWORK…"
near contract deploy --accountId "$ACCOUNT_ID" --wasmFile "$WASM" --networkId "$NETWORK"

echo "Initializing contract (new)…"
near call "$ACCOUNT_ID" new '{}' --accountId "$ACCOUNT_ID" --networkId "$NETWORK"

echo "Verifying deployment…"
near view "$ACCOUNT_ID" get_platform_fee_bps '{}' --networkId "$NETWORK"

echo "Done. Set NEAR_VOTING_CONTRACT_ID=$ACCOUNT_ID in apps/dreamweave/wrangler.jsonc (testing/prod) and redeploy the Worker."
