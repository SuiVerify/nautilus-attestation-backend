#!/bin/bash

# SuiVerify VSOCK Setup Script
# Sets up VSOCK forwarding to access enclave attestation backend
# Run this after starting the enclave with 'make run'

echo "=== SuiVerify VSOCK Setup ==="

# Get enclave CID
ENCLAVE_CID=$(sudo nitro-cli describe-enclaves | jq -r '.[0].EnclaveCID // empty')

if [ -z "$ENCLAVE_CID" ]; then
    echo "❌ No enclave found or not running"
    echo "   Run: make run"
    exit 1
fi

echo "📋 Enclave CID: $ENCLAVE_CID"

# Check which socat binaries are available
echo "🔍 Checking socat binaries..."
if [ -f "/usr/local/bin/socat" ]; then
    echo "✅ Found VSOCK-capable socat: /usr/local/bin/socat"
    SOCAT_BIN="/usr/local/bin/socat"
elif [ -f "/usr/bin/socat" ]; then
    echo "⚠️  Found system socat: /usr/bin/socat (may not support VSOCK)"
    SOCAT_BIN="/usr/bin/socat"
else
    echo "❌ No socat binary found"
    exit 1
fi

# Kill existing processes on port 4000
echo "🧹 Cleaning up existing processes on port 4000..."
sudo pkill -f "socat.*4000" 2>/dev/null || true
sudo pkill vsock-proxy 2>/dev/null || true
sleep 1

# Set up VSOCK connection using the correct socat binary
echo "🚀 Setting up VSOCK connection..."
echo "   Using: $SOCAT_BIN"
echo "   Connecting host port 4000 to enclave CID $ENCLAVE_CID port 4000"

# Use the VSOCK-capable socat binary
sudo $SOCAT_BIN TCP-LISTEN:4000,reuseaddr,fork VSOCK:$ENCLAVE_CID:4000 &
SOCAT_PID=$!

# Wait for socat to start
sleep 3

echo "🧪 Testing connection..."

# Check if socat is running without errors
if ps -p $SOCAT_PID > /dev/null 2>&1; then
    echo "✅ Socat process is running (PID: $SOCAT_PID)"
    
    # Test the connection
    if curl -s --connect-timeout 5 http://localhost:4000/health > /dev/null 2>&1; then
        echo "✅ Health endpoint responding!"
        
        # Test attestation endpoint
        echo "🔍 Testing attestation endpoint..."
        ATTESTATION_RESPONSE=$(curl -s http://localhost:4000/get_attestation 2>/dev/null)
        if [ ! -z "$ATTESTATION_RESPONSE" ]; then
            echo "✅ Attestation endpoint working!"
            echo "📊 Response preview: $(echo "$ATTESTATION_RESPONSE" | head -c 100)..."
            echo ""
            echo "🎉 SUCCESS! Enclave attestation backend is now accessible!"
            echo ""
            echo "📋 You can now run:"
            echo "   curl http://localhost:4000/health | jq"
            echo "   curl http://localhost:4000/get_attestation | jq"
            echo "   ./test_attestation.sh"
            echo "   ./quick_register.sh"
        else
            echo "⚠️  Attestation endpoint returned empty response"
            echo "   Health check worked, trying direct curl..."
            curl -v http://localhost:4000/get_attestation
        fi
    else
        echo "❌ Port 4000 not responding to HTTP requests"
        echo "   Testing raw connection..."
        timeout 5 nc -zv localhost 4000 || echo "   Raw connection failed"
    fi
else
    echo "❌ Socat process died - checking for errors"
    echo "   Last few lines of system log:"
    sudo journalctl -n 5 --no-pager | grep socat || echo "   No socat errors in journal"
fi

echo ""
echo "📋 Final Status:"
echo "   Socat Binary: $SOCAT_BIN"
echo "   Socat PID: $SOCAT_PID"
echo "   Command: $SOCAT_BIN TCP-LISTEN:4000,reuseaddr,fork VSOCK:$ENCLAVE_CID:4000"
echo ""
echo "🔍 Processes listening on port 4000:"
sudo lsof -i :4000 2>/dev/null || echo "   No processes listening on port 4000"
