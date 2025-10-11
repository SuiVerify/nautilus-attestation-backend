#!/bin/bash

# Setup Enclave Configuration
# This creates the necessary EnclaveConfig object before registration

PACKAGE_ID="0x106e1ebf3dc76ef2fecd1d72275bfae0a265144b266495f61e2a4c3b00193764"

echo "=== SuiVerify Enclave Config Setup ==="
echo "📋 Package ID: $PACKAGE_ID"
echo ""

# Check if PCR file exists
if [ ! -f "out/nitro.pcrs" ]; then
    echo "❌ Error: out/nitro.pcrs file not found"
    echo "   Please run 'make' to build the enclave first"
    exit 1
fi

# Extract PCR values
PCR0=$(grep "PCR0" out/nitro.pcrs | awk '{print $1}')
PCR1=$(grep "PCR1" out/nitro.pcrs | awk '{print $1}')
PCR2=$(grep "PCR2" out/nitro.pcrs | awk '{print $1}')

if [ -z "$PCR0" ] || [ -z "$PCR1" ] || [ -z "$PCR2" ]; then
    echo "❌ Error: Could not extract PCR values from out/nitro.pcrs"
    exit 1
fi

echo "📋 Current PCRs:"
echo "   PCR0: $PCR0"
echo "   PCR1: $PCR1" 
echo "   PCR2: $PCR2"
echo ""

echo "🔍 Looking for existing EnclaveConfig and Cap objects..."
echo ""

# The init() function creates these automatically when deployed
# We need to find them by querying objects owned by the deployer

echo "📋 Searching for Cap and EnclaveConfig objects..."

# Get all objects and filter for our types
OBJECTS=$(sui client objects --json)

# Find Cap object
CAP_OBJECT_ID=$(echo "$OBJECTS" | jq -r --arg pkg "$PACKAGE_ID" '.[] | select(.type | contains($pkg + "::enclave::Cap")) | .objectId' | head -1)

# Find EnclaveConfig object  
CONFIG_OBJECT_ID=$(echo "$OBJECTS" | jq -r --arg pkg "$PACKAGE_ID" '.[] | select(.type | contains($pkg + "::enclave::EnclaveConfig")) | .objectId' | head -1)

if [ -z "$CAP_OBJECT_ID" ]; then
    echo "❌ Cap object not found. Was the contract deployed correctly?"
    echo "   The init() function should create these automatically."
    exit 1
fi

if [ -z "$CONFIG_OBJECT_ID" ]; then
    echo "❌ EnclaveConfig object not found. Was the contract deployed correctly?"
    echo "   The init() function should create these automatically."
    exit 1
fi

echo "✅ Found Cap: $CAP_OBJECT_ID"
echo "✅ Found EnclaveConfig: $CONFIG_OBJECT_ID"
echo ""

# Now update the EnclaveConfig with real PCR values
echo "📋 Updating EnclaveConfig with real PCR values..."

# Convert PCR hex strings to byte arrays
PCR0_ARRAY=$(python3 -c "
hex_str = '$PCR0'
bytes_list = [str(int(hex_str[i:i+2], 16)) + 'u8' for i in range(0, len(hex_str), 2)]
print('[' + ', '.join(bytes_list) + ']')
")

PCR1_ARRAY=$(python3 -c "
hex_str = '$PCR1'
bytes_list = [str(int(hex_str[i:i+2], 16)) + 'u8' for i in range(0, len(hex_str), 2)]
print('[' + ', '.join(bytes_list) + ']')
")

PCR2_ARRAY=$(python3 -c "
hex_str = '$PCR2'
bytes_list = [str(int(hex_str[i:i+2], 16)) + 'u8' for i in range(0, len(hex_str), 2)]
print('[' + ', '.join(bytes_list) + ']')
")

# Update PCRs in the existing config
UPDATE_RESULT=$(sui client ptb \
    --move-call "${PACKAGE_ID}::enclave::update_pcrs<${PACKAGE_ID}::enclave::ENCLAVE>" @${CONFIG_OBJECT_ID} @${CAP_OBJECT_ID} "vector${PCR0_ARRAY}" "vector${PCR1_ARRAY}" "vector${PCR2_ARRAY}" \
    --gas-budget 100000000 \
    --json)

if [ $? -ne 0 ]; then
    echo "❌ Failed to update PCRs in EnclaveConfig"
    exit 1
fi

echo "✅ EnclaveConfig updated with real PCR values"
echo ""

# Save the configuration for future use
cat > enclave_objects.json << EOF
{
  "cap_object_id": "$CAP_OBJECT_ID",
  "enclave_config_id": "$CONFIG_OBJECT_ID",
  "package_id": "$PACKAGE_ID",
  "pcr0": "$PCR0",
  "pcr1": "$PCR1",
  "pcr2": "$PCR2"
}
EOF

echo "🎉 Enclave configuration setup complete!"
echo ""
echo "📋 Created objects:"
echo "   Cap ID: $CAP_OBJECT_ID"
echo "   EnclaveConfig ID: $CONFIG_OBJECT_ID"
echo ""
echo "💾 Configuration saved to: enclave_objects.json"
echo ""
echo "🚀 Next step: Run registration with:"
echo "   ./quick_register.sh"
