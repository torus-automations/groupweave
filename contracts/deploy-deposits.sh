#!/bin/bash

# Dreamweave Deposits Contract Deployment Script
# Deploy to NEAR testnet with initialization

set -e

ACCOUNT_ID="eveeiaoelefj.testnet"
CONTRACT_PATH="target/wasm32-unknown-unknown/release/dreamweave_deposits.wasm"
NETWORK="testnet"

echo "======================================"
echo "Deploying Dreamweave Deposits Contract"
echo "======================================"
echo "Account: $ACCOUNT_ID"
echo "Network: $NETWORK"
echo "Contract: $CONTRACT_PATH"
echo "======================================"

# Check if contract file exists
if [ ! -f "$CONTRACT_PATH" ]; then
    echo "Error: Contract file not found at $CONTRACT_PATH"
    echo "Building contract..."
    cargo build --target wasm32-unknown-unknown --release
fi

# Method 1: Try with near-cli (requires login first)
echo ""
echo "Attempting deployment with near-cli..."
echo "If this fails, you need to run: near login"
echo ""

near deploy $ACCOUNT_ID $CONTRACT_PATH \
    --initFunction new \
    --initArgs "{\"owner_id\":\"$ACCOUNT_ID\",\"treasury_account_id\":\"$ACCOUNT_ID\"}" \
    --networkId $NETWORK

if [ $? -eq 0 ]; then
    echo "✅ Contract deployed successfully!"
    echo ""
    echo "View your contract at:"
    echo "https://explorer.testnet.near.org/accounts/$ACCOUNT_ID"
    exit 0
fi

echo ""
echo "======================================"
echo "Deployment failed. Alternative methods:"
echo "======================================"
echo ""
echo "1. Login to NEAR CLI first:"
echo "   near login"
echo ""
echo "2. Then run this script again"
echo ""
echo "3. OR deploy without this script:"
echo "   near deploy $ACCOUNT_ID $CONTRACT_PATH \\"
echo "     --initFunction new \\"
echo "     --initArgs '{\"owner_id\":\"$ACCOUNT_ID\",\"treasury_account_id\":\"$ACCOUNT_ID\"}' \\"
echo "     --networkId testnet"
echo ""
echo "4. OR if using near-cli-rs (new near CLI):"
echo "   near contract deploy $ACCOUNT_ID \\"
echo "     use-file $CONTRACT_PATH \\"
echo "     with-init-call new json-args '{\"owner_id\":\"$ACCOUNT_ID\",\"treasury_account_id\":\"$ACCOUNT_ID\"}' \\"
echo "     prepaid-gas '30.0 Tgas' attached-deposit '0 NEAR' \\"
echo "     network-config testnet sign-with-keychain send"
echo ""

exit 1
