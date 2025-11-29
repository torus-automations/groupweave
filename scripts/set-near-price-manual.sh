#!/bin/bash
# Set NEAR price in deposit contract

set -e

CONTRACT_ID="dreamweave-credits.eveeiaoelefj.testnet"
SIGNER="eveeiaoelefj.testnet"
NEAR_PRICE_USD="3.25"

# Convert to micro-USD (multiply by 1,000,000)
PRICE_MICROS=$(echo "$NEAR_PRICE_USD * 1000000" | bc | cut -d'.' -f1)

echo "================================================"
echo "Setting NEAR price in contract"
echo "================================================"
echo "Contract: $CONTRACT_ID"
echo "Price: \$$NEAR_PRICE_USD USD"
echo "Price (micro-USD): $PRICE_MICROS"
echo "Signer: $SIGNER"
echo ""

near contract call-function as-transaction \
  "$CONTRACT_ID" \
  update_token_price \
  json-args "{\"token_id\":\"NEAR\",\"price_usd_micros\":\"$PRICE_MICROS\"}" \
  prepaid-gas '30.0 Tgas' \
  attached-deposit '0 NEAR' \
  sign-as "$SIGNER" \
  network-config testnet \
  sign-with-keychain send

echo ""
echo "✅ Price set successfully!"
echo ""
echo "Verify with:"
echo "near contract call-function as-read-only $CONTRACT_ID list_token_configs json-args '{}' network-config testnet now"
