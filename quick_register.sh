#!/bin/bash

# Quick registration script for deployed SuiVerify contract
# Uses the deployed contract details from transaction GfVdQBof37WFQJzJ39JCUjitqPa6KsB6D13HGa6NoUn2

# Deployed contract details
PACKAGE_ID="0x106e1ebf3dc76ef2fecd1d72275bfae0a265144b266495f61e2a4c3b00193764"
ENCLAVE_URL="http://localhost:4000"

# Check if enclave configuration exists
if [ ! -f "enclave_objects.json" ]; then
    echo "❌ EnclaveConfig not found!"
    echo "   Please run: ./setup_enclave_config.sh first"
    exit 1
fi

# Load enclave configuration
ENCLAVE_CONFIG_ID=$(jq -r '.enclave_config_id' enclave_objects.json)
CAP_OBJECT_ID=$(jq -r '.cap_object_id' enclave_objects.json)

echo "=== Quick SuiVerify Enclave Registration ==="
echo "📋 Using deployed contract: $PACKAGE_ID"
echo "🔗 Transaction: ViouAvr4NZzF9bnAqmMvS7LL8Ee6kesMTuE5mVHUWjP"
echo ""

# Check if attestation backend is accessible
echo "🔍 Checking attestation backend..."
if ! curl -s --connect-timeout 5 "$ENCLAVE_URL/health" > /dev/null 2>&1; then
    echo "❌ Attestation backend not accessible at $ENCLAVE_URL"
    echo "   Setup VSOCK forwarding with: ./setup_vsock.sh"
    exit 1
fi
echo "✅ Attestation backend is accessible"

# Check if secrets.json exists
if [ ! -f "secrets.json" ]; then
    echo "⚠️  Creating default secrets.json..."
    cat > secrets.json << EOF
{
  "VERSION_CONTROL": "no_update",
  "OLD_ENCLAVE_ID": ""
}
EOF
    echo "✅ Created secrets.json with default settings"
fi

echo ""
echo "🚀 Running registration with deployed contract..."
echo ""

# Run the registration
./register_enclave.sh \
  "$PACKAGE_ID" \
  "$DID_REGISTRY" \
  "$REGISTRY_CAP" \
  "$ENCLAVE_URL" \
  "$PACKAGE_ID"

echo ""
echo "=== Quick Registration Complete ==="
