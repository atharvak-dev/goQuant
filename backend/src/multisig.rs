use crate::error::UpgradeError;
use crate::squads::SquadsClient;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultisigProposal {
    pub id: String,
    pub instruction: Vec<u8>,
    pub description: String,
    pub timelock: i64,
    pub approvals: Vec<String>,
    pub threshold: u8,
    pub status: MultisigStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MultisigStatus {
    Pending,
    Approved,
    Executed,
    Rejected,
}

pub struct MultisigCoordinator {
    proposals: Arc<Mutex<Vec<MultisigProposal>>>,
    members: Vec<String>,
    threshold: u8,
    squads_client: Option<Arc<SquadsClient>>,
    multisig_vault: Option<Pubkey>,
}

impl MultisigCoordinator {
    pub async fn new() -> Result<Self, UpgradeError> {
        // Initialize with optional Squads Protocol integration
        let rpc_url = std::env::var("SOLANA_RPC_URL")
            .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
        
        let multisig_vault_str = std::env::var("MULTISIG_VAULT").ok();
        let multisig_vault = multisig_vault_str
            .as_ref()
            .and_then(|s| Pubkey::from_str(s).ok());
        
        let squads_client = multisig_vault.map(|vault| {
            Arc::new(SquadsClient::new(rpc_url, vault, 3).unwrap())
        });
        
        Ok(Self {
            proposals: Arc::new(Mutex::new(Vec::new())),
            members: vec![
                "member1".to_string(),
                "member2".to_string(),
                "member3".to_string(),
                "member4".to_string(),
                "member5".to_string(),
            ],
            threshold: 3,
            squads_client,
            multisig_vault,
        })
    }

    pub async fn propose_transaction(
        &self,
        params: crate::proposal::ProposalParams,
    ) -> Result<String, UpgradeError> {
        let proposal_id = uuid::Uuid::new_v4().to_string();

        let proposal = MultisigProposal {
            id: proposal_id.clone(),
            instruction: params.instruction,
            description: params.description,
            timelock: params.timelock,
            approvals: vec![],
            threshold: self.threshold,
            status: MultisigStatus::Pending,
        };

        let mut proposals = self.proposals.lock().await;
        proposals.push(proposal);

        tracing::info!("Multisig proposal created: {}", proposal_id);

        Ok(proposal_id)
    }

    pub async fn approve_proposal(&self, proposal_id: &str) -> Result<(), UpgradeError> {
        let mut proposals = self.proposals.lock().await;
        let proposal = proposals
            .iter_mut()
            .find(|p| p.id == proposal_id)
            .ok_or_else(|| UpgradeError::ProposalNotFound(proposal_id.to_string()))?;

        // In real implementation, verify signer is a multisig member
        let approver = "member1".to_string(); // Get from context

        if proposal.approvals.contains(&approver) {
            return Err(UpgradeError::InternalError("Already approved".to_string()));
        }

        proposal.approvals.push(approver.clone());

        // Check if threshold met
        if proposal.approvals.len() >= proposal.threshold as usize {
            proposal.status = MultisigStatus::Approved;
            tracing::info!("Proposal approved! Threshold met: {}", proposal_id);
        }

        Ok(())
    }

    pub async fn execute_transaction(&self, proposal_id: &str) -> Result<(), UpgradeError> {
        let mut proposals = self.proposals.lock().await;
        let proposal = proposals
            .iter_mut()
            .find(|p| p.id == proposal_id)
            .ok_or_else(|| UpgradeError::ProposalNotFound(proposal_id.to_string()))?;

        if proposal.status != MultisigStatus::Approved {
            return Err(UpgradeError::InternalError(
                "Proposal not approved".to_string(),
            ));
        }

        // Execute via Squads Protocol if available
        if let Some(squads) = &self.squads_client {
            if let Some(vault) = self.multisig_vault {
                // Build upgrade instruction
                // In production, this would use actual program/buffer addresses
                let program_id = Pubkey::default(); // Placeholder
                let buffer = Pubkey::default(); // Placeholder
                let upgrade_authority = vault;
                let program_data = Pubkey::default(); // Placeholder
                
                let upgrade_ix = squads.build_upgrade_instruction(
                    &program_id,
                    &buffer,
                    &upgrade_authority,
                    &program_data,
                )?;
                
                // Execute via Squads
                let tx_sig = squads.execute_transaction(&vault).await?;
                tracing::info!("Squads transaction executed: {}", tx_sig);
            }
        }

        proposal.status = MultisigStatus::Executed;
        tracing::info!("Transaction executed: {}", proposal_id);

        Ok(())
    }

    pub async fn get_proposal(&self, proposal_id: &str) -> Result<MultisigProposal, UpgradeError> {
        let proposals = self.proposals.lock().await;
        proposals
            .iter()
            .find(|p| p.id == proposal_id)
            .cloned()
            .ok_or_else(|| UpgradeError::ProposalNotFound(proposal_id.to_string()))
    }

    pub async fn get_members(&self) -> Vec<String> {
        self.members.clone()
    }

    pub fn get_threshold(&self) -> u8 {
        self.threshold
    }
}

