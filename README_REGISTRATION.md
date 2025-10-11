# SuiVerify Enclave Registration - Quick Start

## 🚀 Quick Start

For immediate registration with the deployed SuiVerify contract:

### **Step 1: Configure Registration Mode**

Edit `config.json` to set your registration mode:

```json
{
    "RUN_MODE": "both",
    "VERSION_CONTROL": "update",     // "update" or "no_update"
    "OLD_ENCLAVE_ID": "",           // Set if destroying old enclave
    "RUST_LOG": "info",
    "PYTHON_ENV": "production",
    "ENCLAVE_MODE": "production"
}
```

**Registration Modes:**
- **`"update"`** - Updates PCRs first, then registers enclave (use for first registration or code changes)
- **`"no_update"`** - Only registers enclave without PCR update (use for server restart with same code)

### **Step 2: Run Registration**

```bash
# 1. Start parent forwarder (before make)
./parent_forwarder.sh &

# 2. Build and run enclave (with AWS feature)
make

# 3. Setup VSOCK forwarding
./setup_vsock.sh

# 4. Register enclave (reads config.json automatically)
./quick_register.sh
```

## 📋 Deployed Contract Details

**Package ID**: `0x5b1c4450aeb62e2eb6718b8446091045760d5d9a1c2695fbe5a1c20b7d13006d`  
**Transaction**: `36VqYReFbmh93RnCu5xboR94BRv9jccJf1d2pw9vBVtm`

### Auto-Created Enclave Objects (via init())
- **EnclaveConfig**: `0x6042e2f378fac25fdf5b8267d846c92a4a6f4b93a07520b49fc86a96014c92e1`
- **Cap**: `0x7fad1c5d1032fcdc3f8990a4d7c25d89c023ec47d31fe2ac4e5a65e0a9b199bd`

### Other Contract Objects  
- **DID Registry**: `0x2c6962f40c84a7df1d40c74ab05c7f60c9afdbae8129cfe507ced948a02cbdc4`
- **Registry Cap**: `0x9aa20287121e2d325405097c54b5a2519a5d3f745ca74d47358a490dc94914cc`

## ⚙️ Configuration Guide

### **config.json Settings**

| Setting | Values | Description |
|---------|--------|-------------|
| `VERSION_CONTROL` | `"update"` | Updates PCRs first, then registers enclave. Use for first registration or when enclave code changes. |
| | `"no_update"` | Only registers enclave without updating PCRs. Use when restarting with same code. |
| `OLD_ENCLAVE_ID` | `""` | Empty for first registration |
| | `"0x123..."` | Object ID of previous enclave to destroy (for updates) |

### **When to Use Each Mode**

**Use `"update"` mode when:**
- First time registering the enclave
- You've modified the enclave code (PCRs changed)
- You want to replace an existing enclave

**Use `"no_update"` mode when:**
- Restarting the enclave server (same code, new public key)
- PCRs haven't changed but you need a new enclave instance

## 📁 Files Overview

| File | Purpose |
|------|---------|
| `register_enclave.sh` | Main registration script (updated with contract details) |
| `quick_register.sh` | One-command registration for deployed contract |
| `test_attestation.sh` | Test script for attestation backend endpoints |
| `config.json` | Configuration file for registration modes and enclave settings |


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
