#!/bin/bash
# Deploy the deposits contract with proper initialization

set -e

ACCOUNT="eveeiaoelefj.testnet"
WASM="target/wasm32-unknown-unknown/release/dreamweave_deposits.wasm"

echo "🚀 Deploying Dreamweave Deposits Contract"
echo "Account: $ACCOUNT"
echo "WASM: $WASM"
echo ""

# First, check if account has keys available
echo "Checking account access..."
near keys $ACCOUNT --networkId testnet

echo ""
echo "Deploying contract with initialization..."
echo ""

# Deploy with force flag in correct position
near deploy --force $ACCOUNT $WASM \
  --initFunction new \
  --initArgs "{\"owner_id\":\"$ACCOUNT\",\"treasury_account_id\":\"$ACCOUNT\"}" \
  --networkId testnet

if [ $? -eq 0 ]; then
  echo ""
  echo "✅ CONTRACT DEPLOYED SUCCESSFULLY!"
  echo ""
  echo "Verifying deployment..."
  near state $ACCOUNT --networkId testnet
  echo ""
  echo "Checking contract state..."
  near view-state $ACCOUNT --networkId testnet | head -10
  echo ""
  echo "View your contract:"
  echo "https://explorer.testnet.near.org/accounts/$ACCOUNT"
else
  echo ""
  echo "❌ DEPLOYMENT FAILED"
  echo ""
  echo "If you see 'credentials' error, you need to add the key:"
  echo ""
  echo "Option 1 - Export from wallet:"
  echo "  near login"
  echo ""
  echo "Option 2 - If you have the private key:"
  echo "  near add-credentials $ACCOUNT"
  echo ""
  echo "Then run this script again"
fi
