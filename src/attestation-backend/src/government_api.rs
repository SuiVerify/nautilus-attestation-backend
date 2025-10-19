// Government API integration for PAN verification
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use reqwest::Client;
use chrono::{DateTime, Utc, Duration};
use sha2::{Sha256, Digest};
use serde_json;
use tracing::{info, warn, error};
use hex;

// JWT token management
#[derive(Debug, Clone)]
pub struct JwtManager {
    client: Client,
    auth_url: String,
    api_key: String,
    api_secret: String,
    current_token: Option<String>,
    token_expires_at: Option<DateTime<Utc>>,
}

// Government API response structures
#[derive(Debug, Deserialize, Serialize)]
pub struct GovernmentApiResponse {
    pub code: u16,
    pub timestamp: u64,
    pub data: PanVerificationData,
    pub transaction_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PanVerificationData {
    #[serde(rename = "@entity")]
    pub entity: String,
    pub pan: String,
    pub status: String,
    pub remarks: Option<String>,
    pub name_as_per_pan_match: bool,
    pub date_of_birth_match: bool,
    pub category: String,
    pub aadhaar_seeding_status: String,
}

// Evidence hash input structure (stable fields + actual data)
#[derive(Debug, Serialize)]
pub struct EvidenceHashInput {
    pub pan: String,
    pub status: String,
    pub name_as_per_pan: String,
    pub date_of_birth: String,
    pub name_as_per_pan_match: bool,
    pub date_of_birth_match: bool,
    pub category: String,
    pub aadhaar_seeding_status: String,
}

// Verification request from Redis
#[derive(Debug, Deserialize)]
pub struct VerificationRequest {
    pub user_wallet: String,
    pub did_id: String,
    pub verification_type: String,
    pub document_data: String, // JSON string containing PAN verification data
    pub extracted_data: Option<String>, // JSON string containing OCR extracted data
    pub user_corrections: Option<String>, // JSON string containing user corrections
    pub timestamp: String,
    pub status: String,
}

// Document data structure from Redis message
#[derive(Debug, Deserialize)]
pub struct DocumentData {
    #[serde(rename = "@entity")]
    pub entity: Option<String>,
    pub pan: String,
    pub name_as_per_pan: String,
    pub date_of_birth: String,
    pub phone_number: Option<String>,
    pub consent: String,
    pub reason: String,
}

impl JwtManager {
    pub fn new() -> Result<Self> {
        // Check if running in enclave mode
        let enclave_mode_str = std::env::var("ENCLAVE_MODE")
            .unwrap_or_else(|_| "false".to_string());
        let enclave_mode = enclave_mode_str.parse::<bool>().unwrap_or(false);
        info!("🔧 JwtManager ENCLAVE_MODE: '{}' -> {}", enclave_mode_str, enclave_mode);
            
        let auth_url = if enclave_mode {
            // In enclave: force localhost:8443 (forwarded via VSOCK)
            let url = "https://localhost:8443/authenticate".to_string();
            info!("🔧 ENCLAVE_MODE=true: Forcing auth URL: {}", url);
            url
        } else {
            // Outside enclave: use env var or direct API
            let url = std::env::var("GOVT_API_AUTH_URL")
                .unwrap_or_else(|_| "https://api.sandbox.co.in/authenticate".to_string());
            info!("🔧 ENCLAVE_MODE=false: Using auth URL: {}", url);
            url
        };
        
        let api_key = std::env::var("GOVT_API_KEY")
            .map_err(|_| anyhow!("GOVT_API_KEY environment variable not set"))?;
        let api_secret = std::env::var("GOVT_API_SECRET")
            .map_err(|_| anyhow!("GOVT_API_SECRET environment variable not set"))?;

        let client = if enclave_mode {
            // In enclave: disable SSL verification for localhost proxy
            Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .danger_accept_invalid_certs(true)
                .danger_accept_invalid_hostnames(true)
                .build()?
        } else {
            // Outside enclave: normal SSL verification
            Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()?
        };

        Ok(Self {
            client,
            auth_url,
            api_key,
            api_secret,
            current_token: None,
            token_expires_at: None,
        })
    }

    // Check if token is valid (not expired within 1 hour buffer)
    pub fn is_token_valid(&self) -> bool {
        match (&self.current_token, &self.token_expires_at) {
            (Some(_), Some(expires_at)) => {
                let now = Utc::now();
                let buffer = Duration::hours(1); // 1 hour buffer before expiry
                *expires_at > now + buffer
            }
            _ => false,
        }
    }

    // Authenticate and get new JWT token
    pub async fn authenticate(&mut self) -> Result<String> {
        info!("Authenticating with government API...");

        let response = self.client
            .post(&self.auth_url)
            .header("accept", "application/json")
            .header("x-api-key", &self.api_key)
            .header("x-api-secret", &self.api_secret)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Authentication failed: {}", response.status()));
        }

        let auth_response: serde_json::Value = response.json().await?;
        
        let token = auth_response["access_token"]
            .as_str()
            .ok_or_else(|| anyhow!("No access_token in auth response"))?
            .to_string();

        // Set expiry to 23 hours from now (24-hour tokens with 1-hour buffer)
        self.token_expires_at = Some(Utc::now() + Duration::hours(23));
        self.current_token = Some(token.clone());

        info!("Successfully authenticated with government API");
        Ok(token)
    }

    // Get valid token (authenticate if needed)
    pub async fn get_valid_token(&mut self) -> Result<String> {
        if !self.is_token_valid() {
            warn!("JWT token expired or invalid, re-authenticating...");
            self.authenticate().await
        } else {
            Ok(self.current_token.as_ref().unwrap().clone())
        }
    }
}

pub struct GovernmentApiClient {
    client: Client,
    jwt_manager: JwtManager,
    api_base_url: String,
}

impl GovernmentApiClient {
    pub fn new() -> Result<Self> {
        // Check if running in enclave mode
        let enclave_mode_str = std::env::var("ENCLAVE_MODE")
            .unwrap_or_else(|_| "false".to_string());
        let enclave_mode = enclave_mode_str.parse::<bool>().unwrap_or(false);
        info!("🔧 GovernmentApiClient ENCLAVE_MODE: '{}' -> {}", enclave_mode_str, enclave_mode);
            
        let api_base_url = if enclave_mode {
            // In enclave: force localhost:8443 (forwarded via VSOCK)
            let url = "https://localhost:8443".to_string();
            info!("🔧 ENCLAVE_MODE=true: Forcing base URL: {}", url);
            url
        } else {
            // Outside enclave: use env var or direct API
            let url = std::env::var("GOVT_API_BASE_URL")
                .unwrap_or_else(|_| "https://api.sandbox.co.in".to_string());
            info!("🔧 ENCLAVE_MODE=false: Using base URL: {}", url);
            url
        };

        let client = if enclave_mode {
            // In enclave: disable SSL verification for localhost proxy
            Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .danger_accept_invalid_certs(true)
                .danger_accept_invalid_hostnames(true)
                .build()?
        } else {
            // Outside enclave: normal SSL verification
            Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build()?
        };

        let jwt_manager = JwtManager::new()?;

        Ok(Self {
            client,
            jwt_manager,
            api_base_url,
        })
    }

    // Verify PAN with government API
    pub async fn verify_pan(&mut self, document_data: &DocumentData) -> Result<GovernmentApiResponse> {
        info!("Starting PAN verification for PAN: {}", document_data.pan);

        // Get valid JWT token
        let token = self.jwt_manager.get_valid_token().await?;

        // Prepare PAN verification payload (match exact API format)
        let verification_payload = serde_json::json!({
            "@entity": "in.co.sandbox.kyc.pan_verification.request",
            "pan": document_data.pan,
            "name_as_per_pan": document_data.name_as_per_pan,
            "date_of_birth": document_data.date_of_birth,
            "consent": document_data.consent,
            "reason": document_data.reason
        });

        let url = format!("{}/kyc/pan/verify", self.api_base_url);

        info!("Making PAN verification API call to: {}", url);

        let response = self.client
            .post(&url)
            .header("authorization", token)  // Use raw JWT token without "Bearer" prefix
            .header("Content-Type", "application/json")
            .header("x-api-key", &self.jwt_manager.api_key)  // Add missing API key header
            .json(&verification_payload)
            .send()
            .await?;

        let status = response.status();
        let response_text = response.text().await?;

        info!("Government API response status: {}", status);

        if !status.is_success() {
            error!("Government API call failed: {} - {}", status, response_text);
            return Err(anyhow!("Government API call failed: {} - {}", status, response_text));
        }

        // Parse response
        let api_response: GovernmentApiResponse = serde_json::from_str(&response_text)
            .map_err(|e| anyhow!("Failed to parse government API response: {} - Response: {}", e, response_text))?;

        info!("PAN verification completed successfully. Status: {}", api_response.data.status);

        Ok(api_response)
    }

    // Generate evidence hash from government API response and user data
    pub fn generate_evidence_hash(
        &self,
        api_response: &GovernmentApiResponse,
        user_name: &str,
        user_dob: &str,
    ) -> Result<String> {
        // Create evidence hash input with stable fields + actual verified data
        let evidence_input = EvidenceHashInput {
            pan: api_response.data.pan.clone(),
            status: api_response.data.status.clone(),
            name_as_per_pan: user_name.to_string(),
            date_of_birth: user_dob.to_string(),
            name_as_per_pan_match: api_response.data.name_as_per_pan_match,
            date_of_birth_match: api_response.data.date_of_birth_match,
            category: api_response.data.category.clone(),
            aadhaar_seeding_status: api_response.data.aadhaar_seeding_status.clone(),
        };

        // Serialize to JSON with consistent ordering
        let json_string = serde_json::to_string(&evidence_input)?;
        
        info!("Evidence hash input: {}", json_string);

        // Generate SHA256 hash
        let mut hasher = Sha256::new();
        hasher.update(json_string.as_bytes());
        let hash_bytes = hasher.finalize();
        let evidence_hash = hex::encode(hash_bytes);

        info!("Generated evidence hash: {}", evidence_hash);

        Ok(evidence_hash)
    }

    // Process verification request from Redis
    pub async fn process_verification_request(&mut self, request: &VerificationRequest) -> Result<(String, String)> {
        info!("Processing verification request for wallet: {}", request.user_wallet);

        // Parse document data from JSON string
        info!("Raw document_data JSON: {}", request.document_data);
        let document_data: DocumentData = serde_json::from_str(&request.document_data)
            .map_err(|e| anyhow!("Failed to parse document_data: {} - JSON: {}", e, request.document_data))?;

        // Make government API call
        let api_response = self.verify_pan(&document_data).await?;

        // Determine verification result
        let verification_result = if api_response.data.status == "valid" 
            && api_response.data.name_as_per_pan_match 
            && api_response.data.date_of_birth_match {
            "verified"
        } else {
            "failed"
        };

        // Generate evidence hash
        let evidence_hash = self.generate_evidence_hash(
            &api_response,
            &document_data.name_as_per_pan,
            &document_data.date_of_birth,
        )?;

        info!("Verification completed for wallet: {} - Result: {} - Evidence Hash: {}", 
               request.user_wallet, verification_result, evidence_hash);

        Ok((verification_result.to_string(), evidence_hash))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evidence_hash_generation() {
        let client = GovernmentApiClient::new().unwrap();
        
        let api_response = GovernmentApiResponse {
            code: 200,
            timestamp: 1760865505809,
            data: PanVerificationData {
                entity: "in.co.sandbox.kyc.pan_verification.response".to_string(),
                pan: "HJTPB9891M".to_string(),
                status: "valid".to_string(),
                remarks: None,
                name_as_per_pan_match: true,
                date_of_birth_match: true,
                category: "individual".to_string(),
                aadhaar_seeding_status: "y".to_string(),
            },
            transaction_id: "2bfc9f4c-e3c9-43d0-aef6-27c9082d7ce0".to_string(),
        };

        let evidence_hash = client.generate_evidence_hash(
            &api_response,
            "Ashwin Balaguru",
            "27/10/2004",
        ).unwrap();

        // Verify hash is generated and is 64 characters (SHA256 hex)
        assert_eq!(evidence_hash.len(), 64);
        assert!(evidence_hash.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
