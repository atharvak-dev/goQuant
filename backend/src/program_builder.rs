use crate::error::UpgradeError;
use sha2::{Digest, Sha256};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::path::PathBuf;
use std::process::Command;

pub struct ProgramBuilder {
    build_dir: PathBuf,
    rpc_client: Option<RpcClient>,
}

impl ProgramBuilder {
    pub async fn new() -> Result<Self, UpgradeError> {
        let build_dir = std::env::temp_dir().join("goquant_builds");
        std::fs::create_dir_all(&build_dir)
            .map_err(|e| UpgradeError::InternalError(format!("Failed to create build dir: {}", e)))?;

        let rpc_url = std::env::var("SOLANA_RPC_URL")
            .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
        let rpc_client = Some(RpcClient::new(rpc_url));

        Ok(Self { build_dir, rpc_client })
    }

    /// Build Anchor program and return binary
    pub async fn build_program(&self, source_path: &str) -> Result<Vec<u8>, UpgradeError> {
        tracing::info!("Building program from: {}", source_path);

        // Change to source directory
        let source_dir = PathBuf::from(source_path);
        
        // Run anchor build
        let output = Command::new("anchor")
            .args(&["build"])
            .current_dir(&source_dir)
            .output()
            .map_err(|e| UpgradeError::InternalError(format!("Build failed: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(UpgradeError::InternalError(format!("Build error: {}", error)));
        }

        // Read compiled binary
        // Anchor builds to target/deploy/<program_name>.so
        let binary_path = source_dir
            .join("target")
            .join("deploy")
            .join("upgrade_manager.so");

        let binary = std::fs::read(&binary_path)
            .map_err(|e| UpgradeError::InternalError(format!("Failed to read binary: {}", e)))?;

        tracing::info!("Program built successfully: {} bytes", binary.len());

        Ok(binary)
    }

    /// Create buffer account and upload program
    pub async fn create_buffer(&self, program_binary: &[u8]) -> Result<Pubkey, UpgradeError> {
        tracing::info!("Creating buffer account for program ({} bytes)", program_binary.len());

        // In production, this would:
        // 1. Create buffer account
        // 2. Upload program binary in chunks
        // 3. Set buffer authority
        // 4. Return buffer pubkey

        // For now, return a placeholder
        // In real implementation, use solana program deploy or manual buffer creation
        Ok(Pubkey::new_unique())
    }

    /// Verify program hash matches expected
    pub async fn verify_program_hash(
        &self,
        program_binary: &[u8],
        expected_hash: &[u8; 32],
    ) -> Result<bool, UpgradeError> {
        let calculated_hash = self.calculate_program_hash(program_binary).await?;
        Ok(calculated_hash == *expected_hash)
    }

    /// Calculate SHA256 hash of program binary
    pub async fn calculate_program_hash(&self, program_binary: &[u8]) -> Result<[u8; 32], UpgradeError> {
        let mut hasher = Sha256::new();
        hasher.update(program_binary);
        let hash = hasher.finalize();
        
        let mut result = [0u8; 32];
        result.copy_from_slice(&hash);
        Ok(result)
    }

    /// Verify program on-chain matches expected hash
    pub async fn verify_onchain_program(
        &self,
        program_id: &Pubkey,
        expected_hash: &[u8; 32],
    ) -> Result<bool, UpgradeError> {
        let client = self.rpc_client.as_ref()
            .ok_or_else(|| UpgradeError::InternalError("RPC client not initialized".to_string()))?;

        // Fetch program account
        let account = client.get_account(program_id)
            .map_err(|e| UpgradeError::SolanaError(format!("Failed to fetch program: {}", e)))?;

        // Extract program data (skip account header)
        // Program data starts after 45 bytes (account header)
        if account.data.len() < 45 {
            return Err(UpgradeError::InternalError("Invalid program account".to_string()));
        }

        let program_data = &account.data[45..];
        
        // Calculate hash of on-chain program
        let onchain_hash = self.calculate_program_hash(program_data).await?;
        
        Ok(onchain_hash == *expected_hash)
    }

    /// Get program data account for upgradeable program
    pub async fn get_program_data_account(
        &self,
        program_id: &Pubkey,
    ) -> Result<Pubkey, UpgradeError> {
        // For upgradeable programs, program data account is derived from program ID
        // Program data = find_program_address([program_id, "programdata"])
        
        use solana_sdk::signature::Signer;
        
        // In production, use find_program_address
        // For now, return placeholder
        Ok(Pubkey::new_unique())
    }
}
