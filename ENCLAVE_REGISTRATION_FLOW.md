# SuiVerify Enclave Registration & Update Flow

## Overview

This document outlines the complete flow for registering and updating Nautilus enclaves on the Sui blockchain using the deployed SuiVerify contracts.

## Deployed Contract Information

**Package ID**: `0x6ec40d30e636afb906e621748ee60a9b72bc59a39325adda43deadd28dc89e09`  
**Transaction Digest**: `GfVdQBof37WFQJzJ39JCUjitqPa6KsB6D13HGa6NoUn2`

### Created Objects

| Object Type | Object ID | Owner | Description |
|-------------|-----------|-------|-------------|
| `PaymentRegistry` | `0x000af5ea941c01e426968d91a420018b9746c493e6fb2512dac4f20f93005748` | Shared | Payment management registry |
| `DIDRegistry` | `0x2c6962f40c84a7df1d40c74ab05c7f60c9afdbae8129cfe507ced948a02cbdc4` | Shared | DID registry for identity management |
| `GovWhitelist` | `0x5db149489d68ece83a08559773a1d1f898e4fa4b31d9807b7bb24c88dc8ffb26` | Shared | Government whitelist registry |
| `PaymentCap` | `0x8471c94622d5a48bab2871469df3fa8d20b1061090c6e7bb48703e353bdd9ce7` | Account | Payment capability object |
| `RegistryCap` | `0x9aa20287121e2d325405097c54b5a2519a5d3f745ca74d47358a490dc94914cc` | Account | Registry capability object |
| `GovCap` | `0xd4cbc702c861bd25c638d5025e7327ebc383ea253eafd30449cddc18f85eba63` | Account | Government capability object |
| `UpgradeCap` | `0xd5eb26b17faa1682a42f7545dee29251ee6bd3e70ba1acab838b310cba36dd44` | Account | Package upgrade capability |

## Registration Flow Types

The registration process supports three different modes based on the `VERSION_CONTROL` field in `secrets.json`:

### 1. Update Mode (`"update"`)
**When to use**: Codebase changes that affect both PCRs and public key
- Updates PCRs on-chain
- Destroys old enclave (if `OLD_ENCLAVE_ID` specified)
- Registers new enclave with new public key

### 2. No Update Mode (`"no_update"`)
**When to use**: Server restart only (no codebase changes)
- Only updates public key
- PCRs remain unchanged
- Registers new enclave

### 3. Destroy Only Mode (`"destroy_only"`)
**When to use**: Cleanup operations
- Only destroys specified old enclave
- No new registration

## Prerequisites

### 1. Build the Enclave
```bash
make
```
This generates `out/nitro.pcrs` with PCR values.

### 2. Configure secrets.json
Create or update `secrets.json` with appropriate configuration:

```json
{
  "VERSION_CONTROL": "update",
  "OLD_ENCLAVE_ID": "0x..."
}
```

### 3. Start Attestation Backend
The attestation backend must be running to provide the `/get_attestation` endpoint:

```bash
cd src/attestation-backend
cargo run
```

The server runs on `http://0.0.0.0:4000` and provides:
- `/get_attestation` - Returns attestation data for enclave registration
- `/health` - Health check endpoint
- `/process_kyc` - KYC processing endpoint

## Registration Commands

### Basic Registration
```bash
./register_enclave.sh \
  <enclave_package_id> \
  <enclave_config_id> \
  <cap_object_id> \
  <enclave_url> \
  <original_package_id>
```

### Example with Deployed Contract
```bash
./register_enclave.sh \
  0x6ec40d30e636afb906e621748ee60a9b72bc59a39325adda43deadd28dc89e09 \
  0x2c6962f40c84a7df1d40c74ab05c7f60c9afdbae8129cfe507ced948a02cbdc4 \
  0x9aa20287121e2d325405097c54b5a2519a5d3f745ca74d47358a490dc94914cc \
  http://localhost:4000 \
  0x6ec40d30e636afb906e621748ee60a9b72bc59a39325adda43deadd28dc89e09
```

## Detailed Process Flow

### Step 1: Attestation Retrieval
The script calls the attestation backend:
```bash
curl -s $ENCLAVE_URL/get_attestation
```

Expected response:
```json
{
  "attestation": "hex_encoded_attestation_data"
}
```

### Step 2: PCR Extraction (Update Mode Only)
From `out/nitro.pcrs`:
```
PCR0: <hash_value>
PCR1: <hash_value>
PCR2: <hash_value>
```

### Step 3: On-chain Operations

#### Update PCRs (Update Mode)
```bash
sui client call \
  --function update_pcrs \
  --module enclave \
  --package $ENCLAVE_PACKAGE_ID \
  --type-args "${ORIGINAL_PACKAGE_ID}::enclave::ENCLAVE" \
  --args $ENCLAVE_CONFIG_OBJECT_ID $CAP_OBJECT_ID 0x$PCR0 0x$PCR1 0x$PCR2 \
  --gas-budget 100000000
```

#### Register Enclave
```bash
sui client ptb \
  --assign v "vector[attestation_bytes]" \
  --move-call "0x2::nitro_attestation::load_nitro_attestation" v @0x6 \
  --assign result \
  --move-call "${ENCLAVE_PACKAGE_ID}::enclave::register_enclave<${ORIGINAL_PACKAGE_ID}::enclave::ENCLAVE>" \
    @${ENCLAVE_CONFIG_OBJECT_ID} @${CAP_OBJECT_ID} result \
  --gas-budget 100000000
```

#### Destroy Old Enclave (If Specified)
```bash
sui client call \
  --function destroy_old_enclave \
  --module enclave \
  --package $ENCLAVE_PACKAGE_ID \
  --type-args "${ORIGINAL_PACKAGE_ID}::enclave::ENCLAVE" \
  --args $OLD_ENCLAVE_ID $ENCLAVE_CONFIG_OBJECT_ID $CAP_OBJECT_ID \
  --gas-budget 100000000
```

## Testing the Attestation Endpoint

### Manual Test
```bash
curl -s http://localhost:4000/get_attestation | jq
```

### Health Check
```bash
curl -s http://localhost:4000/health
```

### Expected Attestation Response Format
```json
{
  "attestation": "3082...hex_data"
}
```

## Error Handling

### Common Issues

1. **Missing PCR file**
   ```
   Error: out/nitro.pcrs file not found. Please run 'make' to build the enclave first.
   ```
   **Solution**: Run `make` in the project root

2. **Empty attestation**
   ```
   Error: Attestation is empty. Please check status of http://localhost:4000 and its get_attestation endpoint.
   ```
   **Solution**: Ensure attestation backend is running and accessible

3. **Missing secrets.json**
   ```
   Error: secrets.json file not found in current directory
   ```
   **Solution**: Create `secrets.json` with required configuration

4. **Invalid VERSION_CONTROL**
   ```
   Unknown VERSION_CONTROL value: invalid_value
   ```
   **Solution**: Use valid values: `update`, `no_update`, or `destroy_only`

## Security Considerations

1. **Capability Objects**: All operations require appropriate capability objects
2. **Attestation Verification**: Attestations are verified on-chain using Sui's nitro attestation module
3. **PCR Validation**: PCR values ensure enclave integrity
4. **Key Rotation**: New public keys are generated on each restart

## Next Steps

After successful registration:
1. **Save Enclave Object ID**: Required for future operations
2. **Update Configuration**: Store new enclave ID in your configuration
3. **SDK Integration**: Use the registered enclave with the SuiVerify SDK
4. **Monitoring**: Monitor enclave health and attestation status

## Troubleshooting

### Debug Mode
Add debug output to scripts:
```bash
set -x  # Enable debug mode
```

### Verify Registration
Check if enclave is properly registered:
```bash
sui client object <enclave_object_id>
```

### Check Transaction Status
```bash
sui client tx-block <transaction_digest>
```
