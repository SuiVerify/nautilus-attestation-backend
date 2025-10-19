// DEPRECATED: This module has been migrated to verification_processor.rs
// 
// The Redis-Sui integration functionality has been successfully moved to 
// verification_processor.rs which now handles the complete end-to-end flow:
// 
// 1. Government API verification
// 2. Evidence hash generation  
// 3. Sui contract calls (start_verification + update_verification_status)
// 4. UserDID object creation and management
// 
// This file is kept for reference but the active implementation is in verification_processor.rs

use anyhow::Result;
use fastcrypto::ed25519::Ed25519KeyPair;
use tracing::info;

// Placeholder function to maintain compatibility
pub async fn start_redis_sui_processor(_keypair: Ed25519KeyPair) -> Result<()> {
    info!("DEPRECATED: Redis-Sui processor functionality moved to verification_processor.rs");
    info!("Please use start_verification_processor() instead");
    Ok(())
}
