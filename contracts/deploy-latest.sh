#!/bin/bash

# Deploy Latest Deposits Contract to NEAR Testnet
# Uses near-cli-rs (modern Rust CLI)

set -e

# Configuration
CONTRACT_ACCOUNT="dreamweave-deposits.eveeiaoelefj.testnet"
PARENT_ACCOUNT="eveeiaoelefj.testnet"
WASM_FILE="target/wasm32-unknown-unknown/release/dreamweave_deposits.wasm"
NETWORK="testnet"

echo "╔══════════════════════════════════════════════════════════╗"
echo "║     Dreamweave Deposits Contract Deployment             ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""
echo "  Contract: $CONTRACT_ACCOUNT"
echo "  Owner:    $PARENT_ACCOUNT"
echo "  Network:  $NETWORK"
echo "  WASM:     $WASM_FILE ($(ls -lh $WASM_FILE | awk '{print $5}'))"
echo ""

# Check if WASM exists
if [ ! -f "$WASM_FILE" ]; then
    echo "❌ WASM file not found. Building contract..."
    cd deposits
    cargo build --target wasm32-unknown-unknown --release
    cd ..
fi

echo "════════════════════════════════════════════════════════════"
echo ""
echo "Step 1: Checking if contract account exists..."
echo ""

# Check if contract account exists
if near account view-account-summary "$CONTRACT_ACCOUNT" network-config "$NETWORK" now > /dev/null 2>&1; then
    echo "✅ Contract account exists: $CONTRACT_ACCOUNT"
    echo ""
    echo "📝 This will REDEPLOY (update) the existing contract."
    echo "   Existing state will be preserved."
    echo ""
    read -p "Continue with redeployment? (y/N): " confirm
    if [[ ! $confirm =~ ^[Yy]$ ]]; then
        echo "Deployment cancelled."
        exit 0
    fi
    
    DEPLOY_MODE="redeploy"
else
    echo "⚠️  Contract account does not exist: $CONTRACT_ACCOUNT"
    echo ""
    echo "📝 Will create subaccount and deploy new contract."
    echo "   This requires ~3 NEAR for account creation + storage."
    echo ""
    read -p "Create new account and deploy? (y/N): " confirm
    if [[ ! $confirm =~ ^[Yy]$ ]]; then
        echo "Deployment cancelled."
        exit 0
    fi
    
    DEPLOY_MODE="new"
fi

echo ""
echo "════════════════════════════════════════════════════════════"
echo ""

if [ "$DEPLOY_MODE" = "new" ]; then
    echo "Step 2: Creating contract account..."
    echo ""
    
    # Create subaccount (requires parent account to be logged in)
    near account create-account fund-myself "$CONTRACT_ACCOUNT" \
        "3 NEAR" autogenerate-new-keypair save-to-keychain \
        sign-as "$PARENT_ACCOUNT" \
        network-config "$NETWORK" \
        sign-with-keychain send
    
    if [ $? -ne 0 ]; then
        echo "❌ Failed to create contract account"
        echo ""
        echo "💡 Make sure you're logged in:"
        echo "   near account import-account using-web-wallet network-config $NETWORK"
        exit 1
    fi
    
    echo ""
    echo "✅ Contract account created!"
fi

echo ""
echo "Step $([ "$DEPLOY_MODE" = "new" ] && echo "3" || echo "2"): Deploying contract..."
echo ""

# Deploy contract with initialization
near contract deploy "$CONTRACT_ACCOUNT" \
    use-file "$WASM_FILE" \
    with-init-call new json-args "{\"owner_id\":\"$PARENT_ACCOUNT\",\"treasury_account_id\":\"$PARENT_ACCOUNT\"}" \
    prepaid-gas '30.0 Tgas' \
    attached-deposit '0 NEAR' \
    network-config "$NETWORK" \
    sign-with-keychain send

if [ $? -ne 0 ]; then
    echo "❌ Deployment failed!"
    echo ""
    echo "💡 Troubleshooting:"
    echo "   1. Make sure you're logged in to $PARENT_ACCOUNT"
    echo "   2. Check account has enough NEAR balance"
    echo "   3. Try: near account import-account using-web-wallet network-config $NETWORK"
    exit 1
fi

echo ""
echo "╔══════════════════════════════════════════════════════════╗"
echo "║              ✅ DEPLOYMENT SUCCESSFUL! ✅                 ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""
echo "  Contract ID: $CONTRACT_ACCOUNT"
echo "  Owner:       $PARENT_ACCOUNT"
echo "  Treasury:    $PARENT_ACCOUNT"
echo ""
echo "🔗 View contract:"
echo "   https://testnet.nearblocks.io/address/$CONTRACT_ACCOUNT"
echo ""
echo "════════════════════════════════════════════════════════════"
echo ""
echo "Next Steps:"
echo ""
echo "1. Configure token prices:"
echo "   near contract call-function as-transaction $CONTRACT_ACCOUNT update_token_price json-args '{\"token_id\":\"NEAR\",\"price_usd_micros\":\"3250000\"}' prepaid-gas '30.0 Tgas' attached-deposit '0 NEAR' sign-as $PARENT_ACCOUNT network-config $NETWORK sign-with-keychain send"
echo ""
echo "2. Add USDT support (optional):"
echo "   near contract call-function as-transaction $CONTRACT_ACCOUNT upsert_token_config json-args '{\"token_id\":\"usdt.tether-token.near\",\"symbol\":\"USDT\",\"decimals\":6,\"price_usd_micros\":\"1000000\",\"is_enabled\":true,\"is_native\":false}' prepaid-gas '30.0 Tgas' attached-deposit '0 NEAR' sign-as $PARENT_ACCOUNT network-config $NETWORK sign-with-keychain send"
echo ""
echo "3. Update .env file with new contract ID (if changed)"
echo ""
echo "4. Test deposit:"
echo "   near contract call-function as-transaction $CONTRACT_ACCOUNT deposit_native json-args '{\"beneficiary_id\":\"test-user-123\",\"credits_hint\":600}' prepaid-gas '30.0 Tgas' attached-deposit '6 NEAR' sign-as YOUR_WALLET.testnet network-config $NETWORK sign-with-keychain send"
echo ""
echo "✨ Deployment complete!"
