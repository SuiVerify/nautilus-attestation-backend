#!/usr/bin/env python3
"""
Sui CLI Proxy Services
Runs on the host and provides HTTP API for Sui CLI calls from the enclave

Requirements: pip install flask python-dotenv requests
"""

from flask import Flask, request, jsonify
import subprocess
import json
import logging
import os
import requests
from datetime import datetime, timedelta
from dotenv import load_dotenv

app = Flask(__name__)
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

@app.route('/health', methods=['GET'])
def health_check():
    """Health check endpoint"""
    return jsonify({"status": "healthy", "service": "sui-proxy"})

@app.route('/sui/client/active-address', methods=['GET'])
def get_active_address():
    """Get the active Sui address"""
    try:
        result = subprocess.run(['sui', 'client', 'active-address'], 
                              capture_output=True, text=True, timeout=10)
        return jsonify({
            'success': result.returncode == 0,
            'stdout': result.stdout.strip(),
            'stderr': result.stderr.strip(),
            'returncode': result.returncode
        })
    except Exception as e:
        logger.error(f"Error getting active address: {e}")
        return jsonify({'success': False, 'error': str(e)}), 500

@app.route('/sui/client/gas', methods=['GET'])
def get_gas():
    """Get gas coins"""
    try:
        result = subprocess.run(['sui', 'client', 'gas'], 
                              capture_output=True, text=True, timeout=10)
        return jsonify({
            'success': result.returncode == 0,
            'stdout': result.stdout.strip(),
            'stderr': result.stderr.strip(),
            'returncode': result.returncode
        })
    except Exception as e:
        logger.error(f"Error getting gas: {e}")
        return jsonify({'success': False, 'error': str(e)}), 500

@app.route('/sui/client/call', methods=['POST'])
def call_contract():
    """Execute a contract call"""
    try:
        data = request.json
        package_id = data.get('package_id')
        module = data.get('module')
        function = data.get('function')
        args = data.get('args', [])
        type_args = data.get('type_args', [])
        gas_budget = data.get('gas_budget', '10000000')
        
        # Build sui client call command
        cmd = ['sui', 'client', 'call', 
               '--package', package_id,
               '--module', module,
               '--function', function,
               '--gas-budget', gas_budget]
        
        # Add type arguments if provided
        for type_arg in type_args:
            cmd.extend(['--type-args', type_arg])
        
        # Add function arguments
        for arg in args:
            cmd.extend(['--args', str(arg)])
        
        logger.info(f"Executing command: {' '.join(cmd)}")
        
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
        
        return jsonify({
            'success': result.returncode == 0,
            'stdout': result.stdout.strip(),
            'stderr': result.stderr.strip(),
            'returncode': result.returncode,
            'command': ' '.join(cmd)
        })
        
    except Exception as e:
        logger.error(f"Error executing contract call: {e}")
        return jsonify({'success': False, 'error': str(e)}), 500

@app.route('/sui/client/ptb', methods=['POST'])
def execute_ptb():
    """Execute a Programmable Transaction Block"""
    try:
        data = request.json
        ptb_commands = data.get('commands', [])
        gas_budget = data.get('gas_budget', '10000000')
        
        # For now, we'll use a simple approach
        # In production, you might want to build the PTB more carefully
        cmd = ['sui', 'client', 'ptb', '--gas-budget', gas_budget]
        
        # Add PTB commands (this is simplified - you may need to adjust based on your needs)
        for command in ptb_commands:
            cmd.extend(['--assign', command])
        
        logger.info(f"Executing PTB command: {' '.join(cmd)}")
        
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
        
        return jsonify({
            'success': result.returncode == 0,
            'stdout': result.stdout.strip(),
            'stderr': result.stderr.strip(),
            'returncode': result.returncode,
            'command': ' '.join(cmd)
        })
        
    except Exception as e:
        logger.error(f"Error executing PTB: {e}")
        return jsonify({'success': False, 'error': str(e)}), 500

if __name__ == '__main__':
    # Load environment variables from .env file
    env_file = os.path.join(os.path.dirname(__file__), 'src', 'attestation-backend', '.env')
    if os.path.exists(env_file):
        load_dotenv(env_file)
        logger.info(f"Loaded environment variables from: {env_file}")
    else:
        logger.warning(f"No .env file found at: {env_file}")
    
    # Check if sui CLI is available
    try:
        result = subprocess.run(['sui', '--version'], capture_output=True, text=True)
        if result.returncode == 0:
            logger.info(f"Sui CLI available: {result.stdout.strip()}")
        else:
            logger.error("Sui CLI not available or not working")
    except Exception as e:
        logger.error(f"Error checking Sui CLI: {e}")
    
    # Verify government API credentials are available
    try:
        api_key = os.getenv('GOVT_API_KEY')
        api_secret = os.getenv('GOVT_API_SECRET')
        if api_key and api_secret:
            logger.info(f"Government API credentials loaded: Key={api_key[:10]}..., Secret={api_secret[:10]}...")
        else:
            logger.warning("Government API credentials not found in environment")
    except Exception as e:
        logger.error(f"Error checking government API credentials: {e}")

    # Start the Flask server
    logger.info("Starting Sui Proxy Service on port 9999")
    app.run(host='0.0.0.0', port=9999, debug=False)

# Government API Proxy Endpoints
# JWT token cache
jwt_token_cache = {
    'token': None,
    'expires_at': None
}

def get_govt_api_credentials():
    """Get government API credentials from environment"""
    api_key = os.getenv('GOVT_API_KEY')
    api_secret = os.getenv('GOVT_API_SECRET')
    if not api_key or not api_secret:
        raise ValueError("GOVT_API_KEY and GOVT_API_SECRET environment variables required")
    return api_key, api_secret

def get_valid_jwt_token():
    """Get valid JWT token, refresh if needed"""
    global jwt_token_cache
    
    # Check if token is still valid (with 1 hour buffer)
    if (jwt_token_cache['token'] and jwt_token_cache['expires_at'] and 
        datetime.now() < jwt_token_cache['expires_at'] - timedelta(hours=1)):
        return jwt_token_cache['token']
    
    # Authenticate and get new token
    api_key, api_secret = get_govt_api_credentials()
    
    auth_response = requests.post(
        'https://api.sandbox.co.in/authenticate',
        headers={
            'accept': 'application/json',
            'x-api-key': api_key,
            'x-api-secret': api_secret
        },
        timeout=30
    )
    
    if not auth_response.ok:
        raise Exception(f"Authentication failed: {auth_response.status_code} - {auth_response.text}")
    
    auth_data = auth_response.json()
    token = auth_data.get('access_token')
    if not token:
        raise Exception("No access_token in authentication response")
    
    # Cache token (expires in 24 hours)
    jwt_token_cache['token'] = token
    jwt_token_cache['expires_at'] = datetime.now() + timedelta(hours=23)
    
    logger.info("Successfully authenticated with government API")
    return token

@app.route('/govt-api/pan/verify', methods=['POST'])
def govt_api_pan_verify():
    """Proxy PAN verification requests to government API"""
    try:
        # Get valid JWT token
        token = get_valid_jwt_token()
        
        # Get request data from enclave
        request_data = request.get_json()
        if not request_data:
            return jsonify({"error": "No JSON data provided"}), 400
        
        logger.info(f"Proxying PAN verification request: {request_data.get('pan', 'N/A')}")
        
        # Make request to government API
        api_key, _ = get_govt_api_credentials()
        
        response = requests.post(
            'https://api.sandbox.co.in/kyc/pan/verify',
            headers={
                'accept': 'application/json',
                'content-type': 'application/json',
                'authorization': token,  # Raw JWT token
                'x-api-key': api_key
            },
            json=request_data,
            timeout=60
        )
        
        if not response.ok:
            logger.error(f"Government API error: {response.status_code} - {response.text}")
            return jsonify({
                "error": f"Government API error: {response.status_code}",
                "details": response.text
            }), response.status_code
        
        result = response.json()
        logger.info(f"PAN verification successful: {result.get('data', {}).get('status', 'N/A')}")
        
        return jsonify(result)
        
    except Exception as e:
        logger.error(f"Error in government API proxy: {e}")
        return jsonify({"error": str(e)}), 500
