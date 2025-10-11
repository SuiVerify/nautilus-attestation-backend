#!/bin/bash

# Quick registration script for deployed SuiVerify contract
# Uses the deployed contract details from transaction GfVdQBof37WFQJzJ39JCUjitqPa6KsB6D13HGa6NoUn2

# Deployed contract details - Hardcoded from deployment
PACKAGE_ID="0x5b1c4450aeb62e2eb6718b8446091045760d5d9a1c2695fbe5a1c20b7d13006d"
ENCLAVE_CONFIG_ID="0x6042e2f378fac25fdf5b8267d846c92a4a6f4b93a07520b49fc86a96014c92e1"
CAP_OBJECT_ID="0x7fad1c5d1032fcdc3f8990a4d7c25d89c023ec47d31fe2ac4e5a65e0a9b199bd"
ENCLAVE_URL="http://localhost:4000"

echo "=== Quick SuiVerify Enclave Registration ==="
echo "📋 Using deployed contract: $PACKAGE_ID"
echo "🔗 Transaction: 36VqYReFbmh93RnCu5xboR94BRv9jccJf1d2pw9vBVtm"
echo ""

# Check if attestation backend is accessible
echo "🔍 Checking attestation backend..."
if ! curl -s --connect-timeout 5 "$ENCLAVE_URL/health" > /dev/null 2>&1; then
    echo "❌ Attestation backend not accessible at $ENCLAVE_URL"
    echo "   Setup VSOCK forwarding with: ./setup_vsock.sh"
    exit 1
fi
echo "✅ Attestation backend is accessible"

# Check if config.json exists
if [ ! -f "config.json" ]; then
    echo "❌ Error: config.json file not found"
    echo "   Please ensure config.json exists with VERSION_CONTROL setting"
    exit 1
fi

# Read configuration from config.json
VERSION_CONTROL=$(jq -r '.VERSION_CONTROL' config.json)
OLD_ENCLAVE_ID=$(jq -r '.OLD_ENCLAVE_ID // ""' config.json)

echo "📋 Configuration from config.json:"
echo "   VERSION_CONTROL: $VERSION_CONTROL"
echo "   OLD_ENCLAVE_ID: $OLD_ENCLAVE_ID"

echo ""
echo "🚀 Running registration with deployed contract..."
echo ""

# Run the registration (no arguments needed - uses hardcoded values)
./register_enclave.sh

echo ""
echo "=== Quick Registration Complete ==="
