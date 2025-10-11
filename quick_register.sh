#!/bin/bash

# Quick registration script for deployed SuiVerify contract
# Uses the deployed contract details from transaction GfVdQBof37WFQJzJ39JCUjitqPa6KsB6D13HGa6NoUn2

# Deployed contract details - Hardcoded from deployment
PACKAGE_ID="0x106e1ebf3dc76ef2fecd1d72275bfae0a265144b266495f61e2a4c3b00193764"
ENCLAVE_CONFIG_ID="0x3dea6c7ec46b60f07f2f3cdd82848836b38a0ffe5b0b7566227aa71c02934671"
CAP_OBJECT_ID="0xd3a9e73d75743164b75f8a73e5aa75a0dac5aed9c42b6a81a856dbec1e5abcff"
ENCLAVE_URL="http://localhost:4000"

echo "=== Quick SuiVerify Enclave Registration ==="
echo "📋 Using deployed contract: $PACKAGE_ID"
echo "🔗 Transaction: GsMJs8VGfm3tDpbELuj9yjZB3a1cvLjQtSxex5dRQS3D"
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
