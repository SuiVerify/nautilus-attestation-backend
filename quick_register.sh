#!/bin/bash

# Quick registration script for deployed SuiVerify contract
# Uses the deployed contract details from transaction GfVdQBof37WFQJzJ39JCUjitqPa6KsB6D13HGa6NoUn2

# Deployed contract details
PACKAGE_ID="0x6ec40d30e636afb906e621748ee60a9b72bc59a39325adda43deadd28dc89e09"
DID_REGISTRY="0x2c6962f40c84a7df1d40c74ab05c7f60c9afdbae8129cfe507ced948a02cbdc4"
REGISTRY_CAP="0x9aa20287121e2d325405097c54b5a2519a5d3f745ca74d47358a490dc94914cc"
ENCLAVE_URL="http://localhost:4000"

echo "=== Quick SuiVerify Enclave Registration ==="
echo "📋 Using deployed contract: $PACKAGE_ID"
echo "🔗 Transaction: GfVdQBof37WFQJzJ39JCUjitqPa6KsB6D13HGa6NoUn2"
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
