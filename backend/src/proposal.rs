use crate::error::UpgradeError;
use crate::multisig::MultisigCoordinator;
use crate::program_builder::ProgramBuilder;
use crate::timelock::TimelockManager;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: String,
    pub proposer: String,
    pub program: String,
    pub new_buffer: String,
    pub description: String,
    pub proposed_at: i64,
    pub timelock_until: i64,
    pub approvals: Vec<String>,
    pub approval_threshold: u8,
    pub status: ProposalStatus,
    pub executed_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalStatus {
    Proposed,
    Approved,
    TimelockActive,
    Executed,
    Cancelled,
}

pub struct ProposalManager {
    multisig: Arc<MultisigCoordinator>,
    timelock_manager: Arc<TimelockManager>,
    program_builder: Arc<ProgramBuilder>,
    proposals: Arc<Mutex<Vec<Proposal>>>,
}

impl ProposalManager {
    pub async fn new(
        multisig: Arc<MultisigCoordinator>,
        timelock_manager: Arc<TimelockManager>,
        program_builder: Arc<ProgramBuilder>,
    ) -> Result<Self, UpgradeError> {
        Ok(Self {
            multisig,
            timelock_manager,
            program_builder,
            proposals: Arc::new(Mutex::new(Vec::new())),
        })
    }

    pub async fn propose_upgrade(
        &self,
        new_program_buffer: Pubkey,
        description: String,
    ) -> Result<String, UpgradeError> {
        let proposal_id = uuid::Uuid::new_v4().to_string();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let timelock_duration = 48 * 60 * 60; // 48 hours
        let timelock_until = now + timelock_duration;

        // Create proposal via multisig
        let multisig_proposal_id = self
            .multisig
            .propose_transaction(ProposalParams {
                instruction: self.build_upgrade_instruction(&new_program_buffer)?,
                description: description.clone(),
                timelock: timelock_duration,
            })
            .await?;

        // Create proposal record
        let proposal = Proposal {
            id: proposal_id.clone(),
            proposer: "multisig".to_string(), // In real implementation, get from context
            program: "program_id".to_string(), // In real implementation, get from config
            new_buffer: new_program_buffer.to_string(),
            description,
            proposed_at: now,
            timelock_until,
            approvals: vec![],
            approval_threshold: 3, // 3 of 5
            status: ProposalStatus::Proposed,
            executed_at: None,
        };

        let mut proposals = self.proposals.lock().await;
        proposals.push(proposal);

        // Notify community
        self.notify_community(&proposal_id).await?;

        Ok(proposal_id)
    }

    pub async fn execute_upgrade(&self, proposal_id: &str) -> Result<(), UpgradeError> {
        let mut proposals = self.proposals.lock().await;
        let proposal = proposals
            .iter_mut()
            .find(|p| p.id == proposal_id)
            .ok_or_else(|| UpgradeError::ProposalNotFound(proposal_id.to_string()))?;

        // Check status
        if proposal.status == ProposalStatus::Executed {
            return Err(UpgradeError::AlreadyExecuted);
        }

        if proposal.status == ProposalStatus::Cancelled {
            return Err(UpgradeError::AlreadyCancelled);
        }

        // Wait for timelock to expire
        self.wait_for_timelock(proposal_id).await?;

        // Verify approvals
        if proposal.approvals.len() < proposal.approval_threshold as usize {
            return Err(UpgradeError::InsufficientApprovals {
                current: proposal.approvals.len(),
                required: proposal.approval_threshold as usize,
            });
        }

        // Execute via multisig
        self.multisig.execute_transaction(proposal_id).await?;

        // Verify upgrade
        self.verify_upgrade().await?;

        // Update proposal
        proposal.status = ProposalStatus::Executed;
        proposal.executed_at = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64
        );

        // Announce completion
        self.announce_upgrade(proposal_id).await?;

        Ok(())
    }

    pub async fn cancel_upgrade(&self, proposal_id: &str) -> Result<(), UpgradeError> {
        let mut proposals = self.proposals.lock().await;
        let proposal = proposals
            .iter_mut()
            .find(|p| p.id == proposal_id)
            .ok_or_else(|| UpgradeError::ProposalNotFound(proposal_id.to_string()))?;

        if proposal.status == ProposalStatus::Executed {
            return Err(UpgradeError::AlreadyExecuted);
        }

        proposal.status = ProposalStatus::Cancelled;

        Ok(())
    }

    pub async fn list_proposals(&self) -> Result<Vec<Proposal>, UpgradeError> {
        let proposals = self.proposals.lock().await;
        Ok(proposals.clone())
    }

    pub async fn get_proposal_status(
        &self,
        proposal_id: &str,
    ) -> Result<serde_json::Value, UpgradeError> {
        let proposals = self.proposals.lock().await;
        let proposal = proposals
            .iter()
            .find(|p| p.id == proposal_id)
            .ok_or_else(|| UpgradeError::ProposalNotFound(proposal_id.to_string()))?;

        Ok(serde_json::json!({
            "id": proposal.id,
            "status": proposal.status,
            "approvals": proposal.approvals.len(),
            "threshold": proposal.approval_threshold,
            "timelock_until": proposal.timelock_until,
            "executed_at": proposal.executed_at,
        }))
    }

    async fn wait_for_timelock(&self, proposal_id: &str) -> Result<(), UpgradeError> {
        let timelock_end = self.timelock_manager.get_timelock_end(proposal_id).await?;
        let now = Utc::now().timestamp();

        if now < timelock_end {
            let remaining = timelock_end - now;
            return Err(UpgradeError::TimelockActive { remaining_seconds: remaining });
        }

        Ok(())
    }

    async fn verify_upgrade(&self) -> Result<(), UpgradeError> {
        // Verify new program is functioning correctly
        // This would include:
        // - Checking program hash
        // - Running health checks
        // - Verifying critical functions
        Ok(())
    }

    async fn announce_upgrade(&self, proposal_id: &str) -> Result<(), UpgradeError> {
        // Announce upgrade completion via notification service
        tracing::info!("Upgrade executed: {}", proposal_id);
        Ok(())
    }

    async fn notify_community(&self, proposal_id: &str) -> Result<(), UpgradeError> {
        // Notify community via multiple channels
        tracing::info!("Notifying community about proposal: {}", proposal_id);
        Ok(())
    }

    fn build_upgrade_instruction(
        &self,
        _new_program_buffer: &Pubkey,
    ) -> Result<Vec<u8>, UpgradeError> {
        // Build upgrade instruction
        // This would construct the actual Solana instruction
        Ok(vec![])
    }
}

#[derive(Debug)]
pub struct ProposalParams {
    pub instruction: Vec<u8>,
    pub description: String,
    pub timelock: i64,
}

