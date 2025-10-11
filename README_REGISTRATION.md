# SuiVerify Enclave Registration - Quick Start

## 🚀 Quick Start

For immediate registration with the deployed SuiVerify contract:

```bash
# 1. Start parent forwarder (before make)
./parent_forwarder.sh &

# 2. Build and run enclave (with AWS feature)
make

# 3. Setup VSOCK forwarding
./setup_vsock.sh

# 4. Register enclave (uses hardcoded object IDs)
./quick_register.sh
```

## 📋 Deployed Contract Details

**Package ID**: `0x106e1ebf3dc76ef2fecd1d72275bfae0a265144b266495f61e2a4c3b00193764`  
**Transaction**: `GsMJs8VGfm3tDpbELuj9yjZB3a1cvLjQtSxex5dRQS3D`

### Auto-Created Enclave Objects (via init())
- **EnclaveConfig**: `0x3dea6c7ec46b60f07f2f3cdd82848836b38a0ffe5b0b7566227aa71c02934671`
- **Cap**: `0xd3a9e73d75743164b75f8a73e5aa75a0dac5aed9c42b6a81a856dbec1e5abcff`

### Other Contract Objects  
- **DID Registry**: `0x2c6962f40c84a7df1d40c74ab05c7f60c9afdbae8129cfe507ced948a02cbdc4`
- **Registry Cap**: `0x9aa20287121e2d325405097c54b5a2519a5d3f745ca74d47358a490dc94914cc`

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
