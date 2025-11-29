#!/bin/bash
# Deploy dreamweave deposits contract to a FRESH sub-account

set -e

ACCOUNT_NAME="dw-deposits.eveeiaoelefj.testnet"

echo "🚀 Deploying Dreamweave Deposits Contract to Fresh Account"
echo "Account: $ACCOUNT_NAME"
echo ""

# Step 1: Create fresh sub-account
echo "Step 1: Creating fresh sub-account..."
near account create-account fund-myself \
  $ACCOUNT_NAME '1 NEAR' \
  autogenerate-new-keypair \
  save-to-keychain \
  sign-as eveeiaoelefj.testnet \
  network-config testnet \
  sign-with-keychain \
  send

echo ""
echo "✅ Sub-account created!"
echo ""

# Wait a moment for account to be fully created
sleep 2

# Step 2: Deploy with initialization
echo "Step 2: Deploying contract with initialization..."
near contract deploy $ACCOUNT_NAME \
  use-file target/wasm32-unknown-unknown/release/dreamweave_deposits.wasm \
  with-init-call new \
  json-args "{\"owner_id\":\"eveeiaoelefj.testnet\",\"treasury_account_id\":\"$ACCOUNT_NAME\"}" \
  prepaid-gas '30.0 Tgas' \
  attached-deposit '0 NEAR' \
  network-config testnet \
  sign-with-keychain \
  send

echo ""
echo "✅ CONTRACT DEPLOYED SUCCESSFULLY!"
echo ""

# Step 3: Verify deployment
echo "Step 3: Verifying deployment..."
near contract view $ACCOUNT_NAME list_token_configs \
  json-args '{}' \
  network-config testnet \
  now

echo ""
echo "✅ DEPLOYMENT COMPLETE!"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📋 Next Steps:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "1. Update your .env file:"
echo "   NEAR_DEPOSIT_CONTRACT_ID=$ACCOUNT_NAME"
echo ""
echo "2. Update NEAR token price (required for deposits):"
echo "   near contract call-function as-transaction \\"
echo "     $ACCOUNT_NAME update_token_price \\"
echo "     json-args '{\"token_id\":\"NEAR\",\"price_usd_micros\":\"5000000\"}' \\"
echo "     prepaid-gas '30.0 Tgas' attached-deposit '0 NEAR' \\"
echo "     sign-as eveeiaoelefj.testnet \\"
echo "     network-config testnet sign-with-keychain send"
echo ""
echo "3. View your contract:"
echo "   https://explorer.testnet.near.org/accounts/$ACCOUNT_NAME"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
