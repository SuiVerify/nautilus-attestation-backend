// New verification processor that integrates government API with Redis and Sui
use anyhow::{Result, anyhow};
use redis::{Client, RedisResult, Value, streams::StreamReadReply};
use serde::{Deserialize, Serialize};
use tokio::time::{Duration, Instant, sleep};
use tracing::{error, info, warn};
use fastcrypto::ed25519::Ed25519KeyPair;
use std::collections::HashMap;

use super::government_api::{GovernmentApiClient, VerificationRequest};

// DID type constants (matching your Move contract)
const DID_PAN_VERIFY: u8 = 0; // PAN covers all verification types now

// Verification result message for Sui contract
#[derive(Debug, Clone, Deserialize, Serialize)]
struct SuiVerificationMessage {
    user_wallet: String,
    did_id: u8,
    result: String,
    evidence_hash: String,
    verified_at: String,
}

// Throughput tracker
#[derive(Debug)]
pub struct ThroughputTracker {
    total_messages: u64,
    start_time: Instant,
    last_report_time: Instant,
}

impl ThroughputTracker {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            total_messages: 0,
            start_time: now,
            last_report_time: now,
        }
    }

    pub fn record_message(&mut self) {
        self.total_messages += 1;
    }

    pub fn get_throughput(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.total_messages as f64 / elapsed
        } else {
            0.0
        }
    }

    pub fn maybe_report(&mut self, interval_secs: u64) -> bool {
        let elapsed = self.last_report_time.elapsed();
        
        if elapsed >= Duration::from_secs(interval_secs) {
            let throughput = self.get_throughput();
            info!("THROUGHPUT: {:.1} messages/sec (total: {})", throughput, self.total_messages);
            self.last_report_time = Instant::now();
            true
        } else {
            false
        }
    }
}

pub struct VerificationProcessor {
    keypair: Ed25519KeyPair,
    redis_client: Client,
    government_api: GovernmentApiClient,
    stream_name: String,
    consumer_group: String,
    consumer_name: String,
    throughput_tracker: ThroughputTracker,
    // Sui contract parameters
    package_id: String,
    registry_id: String,
    cap_id: String,
    clock_id: String,
    // Redis authentication
    redis_username: String,
    redis_password: String,
}

impl VerificationProcessor {
    const REPORT_INTERVAL_SECS: u64 = 10;
    const POLL_INTERVAL_MS: u64 = 1000; // 1 second polling

    pub fn new(keypair: Ed25519KeyPair) -> Result<Self> {
        // Redis configuration
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());
        
        info!("Redis configuration source: .env files");
        info!("Redis URL: {}", 
              if redis_url.contains("redis-cloud.com") { 
                  "Redis Cloud (credentials hidden)" 
              } else { 
                  &redis_url 
              });
        
        let client = Client::open(redis_url.as_str())
            .map_err(|e| anyhow!("Failed to create Redis client: {}", e))?;

        // Get Redis authentication credentials
        let redis_username = std::env::var("REDIS_USERNAME")
            .unwrap_or_else(|_| "default".to_string());
        let redis_password = std::env::var("REDIS_PASSWORD")
            .map_err(|_| anyhow!("REDIS_PASSWORD environment variable is required"))?;

        // Initialize government API client
        let government_api = GovernmentApiClient::new()
            .map_err(|e| anyhow!("Failed to initialize government API client: {}", e))?;

        Ok(VerificationProcessor {
            keypair,
            redis_client: client,
            government_api,
            stream_name: std::env::var("REDIS_STREAM_NAME")
                .unwrap_or_else(|_| "verification_stream".to_string()),
            consumer_group: std::env::var("REDIS_CONSUMER_GROUP")
                .unwrap_or_else(|_| "attestation_processors".to_string()),
            consumer_name: std::env::var("REDIS_CONSUMER_NAME")
                .unwrap_or_else(|_| "rust_processor_1".to_string()),
            throughput_tracker: ThroughputTracker::new(),
            package_id: std::env::var("SUI_PACKAGE_ID")
                .unwrap_or_else(|_| "0x6ec40d30e636afb906e621748ee60a9b72bc59a39325adda43deadd28dc89e09".to_string()),
            registry_id: std::env::var("SUI_REGISTRY_ID")
                .unwrap_or_else(|_| "0x2c6962f40c84a7df1d40c74ab05c7f60c9afdbae8129cfe507ced948a02cbdc4".to_string()),
            cap_id: std::env::var("SUI_CAP_ID")
                .unwrap_or_else(|_| "0x9aa20287121e2d325405097c54b5a2519a5d3f745ca74d47358a490dc94914cc".to_string()),
            clock_id: std::env::var("SUI_CLOCK_ID")
                .unwrap_or_else(|_| "0x0000000000000000000000000000000000000000000000000000000000000006".to_string()),
            redis_username,
            redis_password,
        })
    }

    /// Helper method to get an authenticated Redis connection
    async fn get_authenticated_connection(&self) -> Result<redis::aio::Connection> {
        let mut conn = self.redis_client.get_async_connection().await
            .map_err(|e| anyhow!("Failed to connect to Redis: {}", e))?;
        
        // Explicit authentication required for Redis Cloud
        info!("Authenticating with Redis using username: {}", self.redis_username);
        let auth_result: RedisResult<String> = redis::cmd("AUTH")
            .arg(&self.redis_username)
            .arg(&self.redis_password)
            .query_async(&mut conn)
            .await;

        match auth_result {
            Ok(_) => {
                info!("Successfully authenticated with Redis");
            }
            Err(e) => {
                error!("Redis authentication failed: {}", e);
                return Err(anyhow!("Redis authentication failed: {}", e));
            }
        }
        
        Ok(conn)
    }

    pub async fn start_processing(&mut self) -> Result<()> {
        info!("Starting Verification Processor with Government API integration...");
        info!("Contract parameters:");
        info!("   Package: {}", self.package_id);
        info!("   Registry: {}", self.registry_id);
        info!("   Cap: {}", self.cap_id);
        info!("   Stream: {}", self.stream_name);
        info!("   Consumer Group: {}", self.consumer_group);
        info!("   Consumer Name: {}", self.consumer_name);
        
        // Create consumer group if it doesn't exist
        self.create_consumer_group().await?;
        
        // Main processing loop
        loop {
            match self.process_pending_messages().await {
                Ok(processed_count) => {
                    if processed_count == 0 {
                        // No messages, sleep briefly
                        sleep(Duration::from_millis(Self::POLL_INTERVAL_MS)).await;
                    }
                    
                    // Report throughput periodically
                    self.throughput_tracker.maybe_report(Self::REPORT_INTERVAL_SECS);
                }
                Err(e) => {
                    error!("Error processing messages: {}", e);
                    sleep(Duration::from_secs(5)).await; // Back off on error
                }
            }
        }
    }

    async fn create_consumer_group(&mut self) -> Result<()> {
        let mut conn = self.get_authenticated_connection().await?;

        // Try to create consumer group (ignore if it already exists)
        let result: RedisResult<String> = redis::cmd("XGROUP")
            .arg("CREATE")
            .arg(&self.stream_name)
            .arg(&self.consumer_group)
            .arg("0")
            .arg("MKSTREAM")
            .query_async(&mut conn)
            .await;

        match result {
            Ok(_) => info!("Created consumer group: {}", self.consumer_group),
            Err(e) => {
                if e.to_string().contains("BUSYGROUP") {
                    info!("Consumer group already exists: {}", self.consumer_group);
                } else {
                    warn!("Failed to create consumer group: {}", e);
                }
            }
        }

        Ok(())
    }

    async fn process_pending_messages(&mut self) -> Result<usize> {
        let mut conn = self.get_authenticated_connection().await?;

        // Read messages from the stream
        let result: RedisResult<StreamReadReply> = redis::cmd("XREADGROUP")
            .arg("GROUP")
            .arg(&self.consumer_group)
            .arg(&self.consumer_name)
            .arg("COUNT")
            .arg("10") // Process up to 10 messages at once
            .arg("BLOCK")
            .arg("1000") // Block for 1 second
            .arg("STREAMS")
            .arg(&self.stream_name)
            .arg(">") // Only new messages
            .query_async(&mut conn)
            .await;

        match result {
            Ok(reply) => {
                let mut processed_count = 0;
                
                for stream_key in reply.keys {
                    for stream_id in stream_key.ids {
                        match self.process_verification_message(&stream_id.id, &stream_id.map).await {
                            Ok(_) => {
                                // Acknowledge the message
                                let _: RedisResult<i32> = redis::cmd("XACK")
                                    .arg(&self.stream_name)
                                    .arg(&self.consumer_group)
                                    .arg(&stream_id.id)
                                    .query_async(&mut conn)
                                    .await;
                                
                                processed_count += 1;
                                self.throughput_tracker.record_message();
                            }
                            Err(e) => {
                                error!("Failed to process message {}: {}", stream_id.id, e);
                                // Don't acknowledge failed messages - they'll be retried
                            }
                        }
                    }
                }
                
                Ok(processed_count)
            }
            Err(e) => {
                if e.to_string().contains("NOGROUP") {
                    warn!("Consumer group doesn't exist, recreating...");
                    self.create_consumer_group().await?;
                    Ok(0)
                } else {
                    Err(anyhow!("Redis stream read error: {}", e))
                }
            }
        }
    }

    async fn process_verification_message(&mut self, message_id: &str, fields: &HashMap<String, Value>) -> Result<()> {
        info!("Processing verification message: {}", message_id);

        // Parse Redis message into VerificationRequest
        let verification_request = self.parse_verification_request(fields)?;

        info!("Processing verification for wallet: {} - Type: {}", 
              verification_request.user_wallet, verification_request.verification_type);

        // Process with government API
        let (verification_result, evidence_hash) = self.government_api
            .process_verification_request(&verification_request)
            .await?;

        // Convert DID string to u8
        let did_id = verification_request.did_id.parse::<u8>()
            .unwrap_or(DID_PAN_VERIFY); // Default to PAN verification

        // Create Sui verification message
        let sui_message = SuiVerificationMessage {
            user_wallet: verification_request.user_wallet.clone(),
            did_id,
            result: verification_result,
            evidence_hash,
            verified_at: chrono::Utc::now().to_rfc3339(),
        };

        // Execute Sui contract call
        self.execute_sui_contract(&sui_message).await?;

        info!("Successfully processed verification for wallet: {}", verification_request.user_wallet);

        Ok(())
    }

    fn parse_verification_request(&self, fields: &HashMap<String, Value>) -> Result<VerificationRequest> {
        let get_field = |key: &str| -> Result<String> {
            fields.get(key)
                .and_then(|v| {
                    // Convert Redis Value to String
                    match v {
                        Value::Data(bytes) => String::from_utf8(bytes.clone()).ok(),
                        Value::Int(i) => Some(i.to_string()),
                        Value::Status(s) => Some(s.clone()),
                        _ => {
                            // For other types, try to use Debug formatting as fallback
                            Some(format!("{:?}", v))
                        }
                    }
                })
                .ok_or_else(|| anyhow!("Missing or invalid field: {}", key))
        };

        Ok(VerificationRequest {
            user_wallet: get_field("user_wallet")?,
            did_id: get_field("did_id")?,
            verification_type: get_field("verification_type")?,
            document_data: get_field("document_data")?,
            extracted_data: get_field("extracted_data").ok(),
            user_corrections: get_field("user_corrections").ok(),
            timestamp: get_field("timestamp")?,
            status: get_field("status")?,
        })
    }

    async fn execute_sui_contract(&self, message: &SuiVerificationMessage) -> Result<()> {
        info!("Executing Sui contract for wallet: {} using HTTP calls to Flask proxy", message.user_wallet);

        // Step 1: Execute start_verification via HTTP call to Flask proxy
        let user_did_id = self.call_start_verification(
            &message.user_wallet,
            message.did_id,
        ).await?;

        if let Some(did_id) = user_did_id {
            info!("✅ Step 1: start_verification successful for wallet: {} with DID ID: {}", 
                  message.user_wallet, did_id);
            
            // Step 2: Execute update_verification_status with evidence hash (only if verified)
            if message.result == "verified" {
                info!("✅ Step 2: Executing update_verification_status with evidence hash");
                
                // Generate signature for the verification
                let signature = self.generate_verification_signature(message)?;
                
                // Parse the original verification timestamp to milliseconds
                let verification_timestamp_ms = chrono::DateTime::parse_from_rfc3339(&message.verified_at)
                    .map_err(|e| anyhow!("Failed to parse verified_at timestamp: {}", e))?
                    .timestamp_millis() as u64;
                
                self.call_update_verification_status(
                    &message.user_wallet,
                    &did_id,
                    true, // is_verified = true
                    signature,
                    verification_timestamp_ms,
                    &message.evidence_hash,
                ).await?;
                
                info!("🎉 Complete Sui contract execution successful for wallet: {}", message.user_wallet);
                info!("Evidence hash recorded on-chain: {}", message.evidence_hash);
            } else {
                info!("⚠️ Verification result is '{}', skipping update_verification_status", message.result);
            }
        } else {
            warn!("❌ start_verification returned None for wallet: {}", message.user_wallet);
        }

        Ok(())
    }

    async fn call_start_verification(
        &self,
        user_address: &str,
        redis_did_id: u8,
    ) -> Result<Option<String>> {
        info!("Calling start_verification via HTTP for user: {}", user_address);
        
        // Map Redis DID ID to contract DID type
        let contract_did_type = match redis_did_id {
            0 => 1, // DID_AGE_VERIFY
            1 => 2, // DID_CITIZENSHIP_VERIFY
            _ => {
                warn!("Unknown DID ID: {}, defaulting to age verification", redis_did_id);
                1
            }
        };

        let call_data = serde_json::json!({
            "package_id": self.package_id,
            "module": "did_registry",
            "function": "start_verification",
            "args": [
                self.registry_id,
                self.cap_id,
                user_address,
                contract_did_type,
                "0x0000000000000000000000000000000000000000000000000000000000000006"  // Clock object ID
            ],
            "gas_budget": "10000000"
        });

        let client = reqwest::Client::new();
        let response = client
            .post("http://localhost:9999/sui/client/call")
            .json(&call_data)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;

        if result["success"].as_bool().unwrap_or(false) {
            info!("start_verification executed successfully for user: {}", user_address);
            let output_str = result["stdout"].as_str().unwrap_or("");
            info!("Output: {}", output_str);
            
            // Extract UserDID object ID from the transaction output using the same logic as redis_sui_processor
            if let Some(user_did_id) = self.extract_user_did_id(output_str) {
                info!("Extracted UserDID ID: {}", user_did_id);
                return Ok(Some(user_did_id));
            } else {
                warn!("Could not extract UserDID ID from transaction output");
            }
            
            let stderr = result["stderr"].as_str().unwrap_or("");
            if !stderr.is_empty() {
                warn!("Warnings: {}", stderr);
            }
        } else {
            let stderr = result["stderr"].as_str().unwrap_or("unknown error");
            let stdout = result["stdout"].as_str().unwrap_or("");
            let returncode = result["returncode"].as_i64().unwrap_or(-1);
            
            error!("start_verification failed for user: {}", user_address);
            error!("Exit code: {}", returncode);
            error!("STDERR: {}", stderr);
            error!("STDOUT: {}", stdout);
        }

        Ok(None)
    }

    async fn call_update_verification_status(
        &self,
        user_address: &str,
        user_did_id: &str,
        verified: bool,
        nautilus_signature: Vec<u8>,
        signature_timestamp_ms: u64,
        evidence_hash: &str,
    ) -> Result<()> {
        info!("Calling update_verification_status via HTTP for user: {}", user_address);

        let call_data = serde_json::json!({
            "package_id": self.package_id,
            "module": "did_registry",
            "function": "update_verification_status",
            "args": [
                self.registry_id,
                self.cap_id,
                user_did_id,
                verified.to_string().to_lowercase(),
                nautilus_signature,
                signature_timestamp_ms.to_string(),
                evidence_hash,
                "0x0000000000000000000000000000000000000000000000000000000000000006"  // Clock object ID
            ],
            "gas_budget": "10000000"
        });

        let client = reqwest::Client::new();
        let response = client
            .post("http://localhost:9999/sui/client/call")
            .json(&call_data)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;

        if result["success"].as_bool().unwrap_or(false) {
            info!("update_verification_status executed successfully for user: {}", user_address);
            let output_str = result["stdout"].as_str().unwrap_or("");
            info!("Output: {}", output_str);
        } else {
            let stderr = result["stderr"].as_str().unwrap_or("unknown error");
            return Err(anyhow!("update_verification_status failed: {}", stderr));
        }

        Ok(())
    }

    /// Extract UserDID object ID from Sui transaction output (replicated from redis_sui_processor.rs)
    fn extract_user_did_id(&self, output: &str) -> Option<String> {
        let lines: Vec<&str> = output.lines().collect();
        let mut i = 0;
        
        // Look for Created Objects section and find the UserDID object
        while i < lines.len() {
            let line = lines[i];
            
            // Look for ObjectID line
            if line.contains("ObjectID:") && line.contains("0x") {
                // Extract the object ID
                if let Some(start) = line.find("0x") {
                    let id_part = &line[start..];
                    let object_id = if let Some(end) = id_part.find(char::is_whitespace) {
                        &id_part[..end]
                    } else {
                        id_part.trim()
                    };
                    
                    // Look ahead for ObjectType line to check if this is a UserDID
                    for j in (i+1)..(i+5).min(lines.len()) {
                        let next_line = lines[j];
                        if next_line.contains("ObjectType:") && next_line.contains("::did_registry::UserDID") {
                            info!("Found UserDID object: {}", object_id);
                            return Some(object_id.to_string());
                        }
                        // Stop looking if we hit another ObjectID (next object)
                        if next_line.contains("ObjectID:") {
                            break;
                        }
                    }
                }
            }
            i += 1;
        }
        
        warn!("Could not find UserDID object in transaction output");
        None
    }

    fn generate_verification_signature(&self, message: &SuiVerificationMessage) -> Result<Vec<u8>> {
        // Create a payload to sign (matching the format expected by the contract)
        // Use the original verification timestamp, not current time
        let payload = format!(
            "{}:{}:{}:{}:{}",
            message.user_wallet,
            message.did_id,
            message.result,
            message.evidence_hash,
            message.verified_at  // Use original verification timestamp
        );
        
        // Sign the payload with the enclave keypair
        use fastcrypto::traits::Signer;
        let signature = self.keypair.sign(payload.as_bytes());
        
        info!("Generated verification signature for wallet: {}", message.user_wallet);
        
        Ok(signature.as_ref().to_vec())
    }
}

// Main entry point for the verification processor
pub async fn start_verification_processor(keypair: Ed25519KeyPair) -> Result<()> {
    let mut processor = VerificationProcessor::new(keypair)?;
    processor.start_processing().await
}