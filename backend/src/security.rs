use crate::error::UpgradeError;
use sha2::{Digest, Sha256};
use solana_sdk::pubkey::Pubkey;

/// Security audit checks for upgrade proposals
pub struct SecurityAuditor;

impl SecurityAuditor {
    /// Audit an upgrade proposal before execution
    pub async fn audit_proposal(
        &self,
        program_hash: &[u8; 32],
        buffer_pubkey: &Pubkey,
        description: &str,
    ) -> Result<AuditResult, UpgradeError> {
        let mut issues = Vec::new();
        let mut warnings = Vec::new();

        // Check 1: Program hash verification
        if !self.verify_program_hash(program_hash).await? {
            issues.push("Program hash verification failed".to_string());
        }

        // Check 2: Description completeness
        if description.len() < 50 {
            warnings.push("Upgrade description is too short".to_string());
        }

        // Check 3: Buffer account verification
        if !self.verify_buffer_account(buffer_pubkey).await? {
            issues.push("Buffer account verification failed".to_string());
        }

        // Check 4: Check for known vulnerabilities
        if self.check_known_vulnerabilities(program_hash).await? {
            issues.push("Program matches known vulnerable pattern".to_string());
        }

        // Check 5: Code review requirement
        if !description.to_lowercase().contains("reviewed") {
            warnings.push("No mention of code review in description".to_string());
        }

        let passed = issues.is_empty();
        let severity = if !issues.is_empty() {
            AuditSeverity::Critical
        } else if !warnings.is_empty() {
            AuditSeverity::Warning
        } else {
            AuditSeverity::Pass
        };

        Ok(AuditResult {
            passed,
            severity,
            issues,
            warnings,
        })
    }

    async fn verify_program_hash(&self, _hash: &[u8; 32]) -> Result<bool, UpgradeError> {
        // In production, verify hash against:
        // 1. Expected hash from audit report
        // 2. Hash from trusted source
        // 3. Hash from CI/CD pipeline
        
        Ok(true) // Placeholder
    }

    async fn verify_buffer_account(&self, _buffer: &Pubkey) -> Result<bool, UpgradeError> {
        // In production, verify:
        // 1. Buffer account exists
        // 2. Buffer is properly initialized
        // 3. Buffer authority is correct
        
        Ok(true) // Placeholder
    }

    async fn check_known_vulnerabilities(&self, _hash: &[u8; 32]) -> Result<bool, UpgradeError> {
        // In production, check against database of known vulnerable hashes
        // or patterns
        
        Ok(false) // Placeholder
    }

    /// Verify multisig configuration is secure
    pub fn verify_multisig_config(
        &self,
        members: &[Pubkey],
        threshold: u8,
    ) -> Result<bool, UpgradeError> {
        // Security checks:
        // 1. Minimum threshold (e.g., at least 3 of 5)
        // 2. Not too many members (prevent key management issues)
        // 3. Members are distinct
        
        if members.len() < 3 {
            return Err(UpgradeError::InternalError(
                "Multisig must have at least 3 members".to_string(),
            ));
        }

        if members.len() > 20 {
            return Err(UpgradeError::InternalError(
                "Multisig should not exceed 20 members".to_string(),
            ));
        }

        if threshold < 2 {
            return Err(UpgradeError::InternalError(
                "Threshold must be at least 2".to_string(),
            ));
        }

        if threshold > members.len() as u8 {
            return Err(UpgradeError::InternalError(
                "Threshold cannot exceed number of members".to_string(),
            ));
        }

        // Check for duplicate members
        let mut seen = std::collections::HashSet::new();
        for member in members {
            if !seen.insert(member) {
                return Err(UpgradeError::InternalError(
                    "Duplicate multisig members not allowed".to_string(),
                ));
            }
        }

        // Require at least 50% threshold for security
        let min_threshold = (members.len() as f64 * 0.5).ceil() as u8;
        if threshold < min_threshold {
            return Err(UpgradeError::InternalError(
                format!("Threshold must be at least {} for security", min_threshold),
            ));
        }

        Ok(true)
    }

    /// Verify timelock duration is adequate
    pub fn verify_timelock(&self, timelock_seconds: i64) -> Result<bool, UpgradeError> {
        const MIN_TIMELOCK: i64 = 48 * 60 * 60; // 48 hours minimum

        if timelock_seconds < MIN_TIMELOCK {
            return Err(UpgradeError::InternalError(
                format!("Timelock must be at least {} seconds (48 hours)", MIN_TIMELOCK),
            ));
        }

        Ok(true)
    }

    /// Calculate program hash for verification
    pub fn calculate_program_hash(program_binary: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(program_binary);
        let hash = hasher.finalize();
        
        let mut result = [0u8; 32];
        result.copy_from_slice(&hash);
        result
    }
}

#[derive(Debug, Clone)]
pub struct AuditResult {
    pub passed: bool,
    pub severity: AuditSeverity,
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AuditSeverity {
    Pass,
    Warning,
    Critical,
}

impl AuditResult {
    pub fn can_proceed(&self) -> bool {
        self.passed && self.severity != AuditSeverity::Critical
    }
}

