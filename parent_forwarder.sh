#!/bin/bash
# Kill any existing forwarders
pkill -f "VSOCK-LISTEN:9999"
pkill -f "VSOCK-LISTEN:6379"
pkill -f "VSOCK-LISTEN:8443"

echo "Starting parent forwarder script..."

# Start Sui CLI proxy service
echo "Starting Sui CLI proxy service on port 9999..."
python3 sui_proxy.py &
SUI_PROXY_PID=$!
echo "Sui proxy started with PID: $SUI_PROXY_PID"

# Forward VSOCK port 9999 to local Sui proxy
echo "Setting up VSOCK forwarding for Sui proxy..."
/usr/local/bin/socat VSOCK-LISTEN:9999,fork,reuseaddr TCP:127.0.0.1:9999 &
SUI_VSOCK_PID=$!
echo "Sui VSOCK forwarder started with PID: $SUI_VSOCK_PID"

# Forward VSOCK port 6379 to YOUR Redis Cloud instance
echo "Setting up VSOCK forwarding for Redis Cloud..."
/usr/local/bin/socat VSOCK-LISTEN:6379,fork,reuseaddr TCP:redis-14701.crce217.ap-south-1-1.ec2.cloud.redislabs.com:14701 &
REDIS_VSOCK_PID=$!
echo "Redis VSOCK forwarder started with PID: $REDIS_VSOCK_PID"

# Forward VSOCK port 8443 to Government API (sandbox.co.in)
echo "Setting up VSOCK forwarding for Government API..."
/usr/local/bin/socat VSOCK-LISTEN:8443,fork,reuseaddr TCP:api.sandbox.co.in:443 &
GOVT_API_VSOCK_PID=$!
echo "Government API VSOCK forwarder started with PID: $GOVT_API_VSOCK_PID"

echo "Parent forwarders setup complete"
echo "PIDs: Sui Proxy=$SUI_PROXY_PID, Sui VSOCK=$SUI_VSOCK_PID, Redis VSOCK=$REDIS_VSOCK_PID, Govt API VSOCK=$GOVT_API_VSOCK_PID"

# Keep script running
wait
