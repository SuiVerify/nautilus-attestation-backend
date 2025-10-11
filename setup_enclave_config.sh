#!/bin/bash

# Setup Enclave Configuration
# This creates the necessary EnclaveConfig object before registration

PACKAGE_ID="0xbf9a4a025fdd056d465993d4397bbfa9a69af9d3df29959672c836ee2edc968d"

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

echo "🚀 Creating Enclave Configuration..."
echo ""

# Step 1: Create Cap using witness
echo "📋 Step 1: Creating Cap with ENCLAVE witness..."
CAP_RESULT=$(sui client ptb \
    --assign witness "${PACKAGE_ID}::enclave::ENCLAVE" \
    --move-call "${PACKAGE_ID}::enclave::new_cap<${PACKAGE_ID}::enclave::ENCLAVE>" witness \
    --assign cap \
    --gas-budget 100000000 \
    --json)

if [ $? -ne 0 ]; then
    echo "❌ Failed to create Cap"
    exit 1
fi

# Extract Cap object ID from result
CAP_OBJECT_ID=$(echo "$CAP_RESULT" | jq -r '.objectChanges[] | select(.type == "created" and (.objectType | contains("Cap"))) | .objectId')

if [ -z "$CAP_OBJECT_ID" ]; then
    echo "❌ Failed to extract Cap object ID"
    exit 1
fi

echo "✅ Cap created: $CAP_OBJECT_ID"
echo ""

# Step 2: Create EnclaveConfig using the Cap
echo "📋 Step 2: Creating EnclaveConfig..."

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

CONFIG_RESULT=$(sui client ptb \
    --move-call "${PACKAGE_ID}::enclave::create_enclave_config<${PACKAGE_ID}::enclave::ENCLAVE>" @${CAP_OBJECT_ID} "\"SuiVerify Enclave\"" "vector${PCR0_ARRAY}" "vector${PCR1_ARRAY}" "vector${PCR2_ARRAY}" \
    --gas-budget 100000000 \
    --json)

if [ $? -ne 0 ]; then
    echo "❌ Failed to create EnclaveConfig"
    exit 1
fi

# Extract EnclaveConfig object ID
CONFIG_OBJECT_ID=$(echo "$CONFIG_RESULT" | jq -r '.objectChanges[] | select(.type == "created" and (.objectType | contains("EnclaveConfig"))) | .objectId')

if [ -z "$CONFIG_OBJECT_ID" ]; then
    echo "❌ Failed to extract EnclaveConfig object ID"
    exit 1
fi

echo "✅ EnclaveConfig created: $CONFIG_OBJECT_ID"
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
