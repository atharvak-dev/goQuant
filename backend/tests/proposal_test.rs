use goquant_upgrade_service::*;
use tokio_test;

#[tokio::test]
async fn test_proposal_creation() {
    let multisig = std::sync::Arc::new(
        multisig::MultisigCoordinator::new().await.unwrap()
    );
    let timelock = std::sync::Arc::new(
        timelock::TimelockManager::new().await.unwrap()
    );
    let builder = std::sync::Arc::new(
        program_builder::ProgramBuilder::new().await.unwrap()
    );

    let proposal_manager = proposal::ProposalManager::new(
        multisig, timelock, builder
    ).await.unwrap();

    let buffer_pubkey = "Buffer11111111111111111111111111111111"
        .parse()
        .unwrap();
    
    let proposal_id = proposal_manager
        .propose_upgrade(buffer_pubkey, "Test upgrade".to_string())
        .await
        .unwrap();

    assert!(!proposal_id.is_empty());

    let proposals = proposal_manager.list_proposals().await.unwrap();
    assert_eq!(proposals.len(), 1);
    assert_eq!(proposals[0].id, proposal_id);
}

#[tokio::test]
async fn test_proposal_approval_flow() {
    let multisig = std::sync::Arc::new(
        multisig::MultisigCoordinator::new().await.unwrap()
    );
    let timelock = std::sync::Arc::new(
        timelock::TimelockManager::new().await.unwrap()
    );
    let builder = std::sync::Arc::new(
        program_builder::ProgramBuilder::new().await.unwrap()
    );

    let proposal_manager = proposal::ProposalManager::new(
        multisig.clone(), timelock, builder
    ).await.unwrap();

    // Create proposal
    let buffer_pubkey = "Buffer11111111111111111111111111111111"
        .parse()
        .unwrap();
    
    let proposal_id = proposal_manager
        .propose_upgrade(buffer_pubkey, "Test upgrade".to_string())
        .await
        .unwrap();

    // Approve proposal
    multisig.approve_proposal(&proposal_id).await.unwrap();

    let status = proposal_manager
        .get_proposal_status(&proposal_id)
        .await
        .unwrap();

    assert_eq!(status["approvals"], 1);
}

#[tokio::test]
async fn test_timelock_enforcement() {
    let multisig = std::sync::Arc::new(
        multisig::MultisigCoordinator::new().await.unwrap()
    );
    let timelock = std::sync::Arc::new(
        timelock::TimelockManager::new().await.unwrap()
    );
    let builder = std::sync::Arc::new(
        program_builder::ProgramBuilder::new().await.unwrap()
    );

    let proposal_manager = proposal::ProposalManager::new(
        multisig, timelock.clone(), builder
    ).await.unwrap();

    let proposal_id = "test-proposal".to_string();
    
    // Set timelock
    timelock.set_timelock(proposal_id.clone(), 3600).await.unwrap(); // 1 hour

    // Should not be expired immediately
    let expired = timelock.is_timelock_expired(&proposal_id).await.unwrap();
    assert!(!expired);

    let remaining = timelock.get_remaining_time(&proposal_id).await.unwrap();
    assert!(remaining > 0);
}

#[tokio::test]
async fn test_proposal_cancellation() {
    let multisig = std::sync::Arc::new(
        multisig::MultisigCoordinator::new().await.unwrap()
    );
    let timelock = std::sync::Arc::new(
        timelock::TimelockManager::new().await.unwrap()
    );
    let builder = std::sync::Arc::new(
        program_builder::ProgramBuilder::new().await.unwrap()
    );

    let proposal_manager = proposal::ProposalManager::new(
        multisig, timelock, builder
    ).await.unwrap();

    // Create proposal
    let buffer_pubkey = "Buffer11111111111111111111111111111111"
        .parse()
        .unwrap();
    
    let proposal_id = proposal_manager
        .propose_upgrade(buffer_pubkey, "Test upgrade".to_string())
        .await
        .unwrap();

    // Cancel proposal
    proposal_manager.cancel_upgrade(&proposal_id).await.unwrap();

    let proposals = proposal_manager.list_proposals().await.unwrap();
    let proposal = proposals.iter().find(|p| p.id == proposal_id).unwrap();
    assert_eq!(proposal.status, proposal::ProposalStatus::Cancelled);
}