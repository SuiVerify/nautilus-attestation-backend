# SuiVerify Enclave Registration - Quick Start

## 🚀 Quick Start

For immediate registration with the deployed SuiVerify contract:

```bash
# 1. Start parent forwarder (before make)
./parent_forwarder.sh &

# 2. Build and run enclave (now with AWS feature + ENCLAVE witness)
make

# 3. Setup VSOCK forwarding
./setup_vsock.sh

# 4. Create EnclaveConfig (one-time setup)
./setup_enclave_config.sh

# 5. Register enclave with the config
./quick_register.sh
```

## 📋 Deployed Contract Details

**Package ID**: `0xbf9a4a025fdd056d465993d4397bbfa9a69af9d3df29959672c836ee2edc968d`  
**Transaction**: `ViouAvr4NZzF9bnAqmMvS7LL8Ee6kesMTuE5mVHUWjP`

### Key Object IDs
- **DID Registry**: `0x2c6962f40c84a7df1d40c74ab05c7f60c9afdbae8129cfe507ced948a02cbdc4`
- **Registry Cap**: `0x9aa20287121e2d325405097c54b5a2519a5d3f745ca74d47358a490dc94914cc`
- **Gov Whitelist**: `0x5db149489d68ece83a08559773a1d1f898e4fa4b31d9807b7bb24c88dc8ffb26`
- **Payment Registry**: `0x000af5ea941c01e426968d91a420018b9746c493e6fb2512dac4f20f93005748`

## 📁 Files Overview

| File | Purpose |
|------|---------|
| `ENCLAVE_REGISTRATION_FLOW.md` | Complete documentation of the registration process |
| `register_enclave.sh` | Main registration script (updated with contract details) |
| `quick_register.sh` | One-command registration for deployed contract |
| `test_attestation.sh` | Test script for attestation backend endpoints |
| `secrets.json` | Configuration file for registration modes |

## 🔧 Registration Modes

Configure in `secrets.json`:

### 1. No Update Mode (Default)
```json
{
  "VERSION_CONTROL": "no_update"
}
```
- **Use case**: Server restart only, no codebase changes
- **Actions**: Register new enclave with new public key
- **PCRs**: Unchanged

### 2. Update Mode
```json
{
  "VERSION_CONTROL": "update",
  "OLD_ENCLAVE_ID": "0x..."
}
```
- **Use case**: Codebase changes affecting PCRs and public key
- **Actions**: Update PCRs → Destroy old enclave → Register new enclave

### 3. Destroy Only Mode
```json
{
  "VERSION_CONTROL": "destroy_only",
  "OLD_ENCLAVE_ID": "0x..."
}
```
- **Use case**: Cleanup operations
- **Actions**: Only destroy specified enclave

## 🧪 Testing

### Test Attestation Backend
```bash
./test_attestation.sh
# or with custom URL
./test_attestation.sh http://your-enclave:4000
```

### Manual Endpoint Tests
```bash
# Health check
curl http://localhost:4000/health

# Get attestation
curl http://localhost:4000/get_attestation | jq

# Root endpoint
curl http://localhost:4000/
```

## 🔄 Complete Registration Flow

### Prerequisites
1. **Build enclave**: `make` (generates `out/nitro.pcrs`)
2. **Start backend**: `cd src/attestation-backend && cargo run`
3. **Configure**: Create/update `secrets.json`

### Manual Registration
```bash
./register_enclave.sh \
  0x6ec40d30e636afb906e621748ee60a9b72bc59a39325adda43deadd28dc89e09 \
  0x2c6962f40c84a7df1d40c74ab05c7f60c9afdbae8129cfe507ced948a02cbdc4 \
  0x9aa20287121e2d325405097c54b5a2519a5d3f745ca74d47358a490dc94914cc \
  http://localhost:4000 \
  0x6ec40d30e636afb906e621748ee60a9b72bc59a39325adda43deadd28dc89e09
```

### Quick Registration
```bash
./quick_register.sh
```

## 🚨 Troubleshooting

### Common Issues

1. **Attestation backend not running**
   ```
   ❌ Error: Cannot reach attestation backend at http://localhost:4000
   ```
   **Solution**: `cd src/attestation-backend && cargo run`

2. **Missing PCR file**
   ```
   Error: out/nitro.pcrs file not found
   ```
   **Solution**: Run `make` to build the enclave

3. **Empty attestation**
   ```
   ❌ Error: Attestation is empty or invalid
   ```
   **Solution**: Check backend logs and endpoint accessibility

### Debug Commands
```bash
# Check if backend is running
curl http://localhost:4000/health

# Verify PCR file exists
ls -la out/nitro.pcrs

# Check secrets.json format
cat secrets.json | jq

# Enable debug mode in scripts
export DEBUG=1
```

## 📚 Next Steps

After successful registration:

1. **Save Enclave Object ID** from transaction output
2. **Update configuration** with new enclave ID
3. **Proceed to SDK integration**
4. **Monitor enclave health**

For detailed information, see `ENCLAVE_REGISTRATION_FLOW.md`.
