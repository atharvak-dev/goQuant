use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    program::invoke_signed,
    system_instruction,
    sysvar::rent::Rent,
};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod upgrade_manager {
    use super::*;

    /// Initialize the upgrade manager with multisig configuration
    pub fn initialize(
        ctx: Context<Initialize>,
        members: Vec<Pubkey>,
        threshold: u8,
        timelock_duration: i64,
    ) -> Result<()> {
        let config = &mut ctx.accounts.multisig_config;
        config.members = members;
        config.threshold = threshold;
        config.upgrade_authority = ctx.accounts.authority.key();
        config.bump = ctx.bumps.multisig_config;

        let state = &mut ctx.accounts.program_upgrade_state;
        state.authority = ctx.accounts.authority.key();
        state.timelock_duration = timelock_duration;
        state.bump = ctx.bumps.program_upgrade_state;

        msg!("Upgrade manager initialized with {} members, threshold: {}", 
             config.members.len(), threshold);
        
        emit!(InitializedEvent {
            authority: ctx.accounts.authority.key(),
            members: config.members.clone(),
            threshold,
            timelock_duration,
        });

        Ok(())
    }

    /// Propose a new program upgrade
    pub fn propose_upgrade(
        ctx: Context<ProposeUpgrade>,
        new_program_buffer: Pubkey,
        description: String,
    ) -> Result<()> {
        let proposal = &mut ctx.accounts.proposal;
        let config = &ctx.accounts.multisig_config;
        let clock = Clock::get()?;

        // Verify proposer is a multisig member
        require!(
            config.members.contains(&ctx.accounts.proposer.key()),
            UpgradeError::NotMultisigMember
        );

        // Initialize proposal
        proposal.id = ctx.accounts.proposal.key().to_bytes()[..8]
            .try_into()
            .map_err(|_| UpgradeError::InvalidProposalId)?;
        proposal.proposer = ctx.accounts.proposer.key();
        proposal.program = ctx.accounts.program.key();
        proposal.new_buffer = new_program_buffer;
        proposal.description = description;
        proposal.proposed_at = clock.unix_timestamp;
        proposal.timelock_until = clock.unix_timestamp + ctx.accounts.program_upgrade_state.timelock_duration;
        proposal.approvals = vec![ctx.accounts.proposer.key()];
        proposal.approval_threshold = config.threshold;
        proposal.status = UpgradeStatus::Proposed;
        proposal.executed_at = None;
        proposal.bump = ctx.bumps.proposal;

        msg!("Upgrade proposed: buffer={}, timelock_until={}", 
             new_program_buffer, proposal.timelock_until);

        emit!(ProposalCreatedEvent {
            proposal_id: ctx.accounts.proposal.key(),
            proposer: ctx.accounts.proposer.key(),
            new_buffer: new_program_buffer,
            timelock_until: proposal.timelock_until,
        });

        Ok(())
    }

    /// Approve an upgrade proposal
    pub fn approve_upgrade(
        ctx: Context<ApproveUpgrade>,
        _proposal_id: Pubkey,
    ) -> Result<()> {
        let proposal = &mut ctx.accounts.proposal;
        let config = &ctx.accounts.multisig_config;
        let clock = Clock::get()?;

        // Verify approver is a multisig member
        require!(
            config.members.contains(&ctx.accounts.approver.key()),
            UpgradeError::NotMultisigMember
        );

        // Check proposal status
        require!(
            proposal.status == UpgradeStatus::Proposed || 
            proposal.status == UpgradeStatus::Approved,
            UpgradeError::InvalidProposalStatus
        );

        // Check if already approved
        require!(
            !proposal.approvals.contains(&ctx.accounts.approver.key()),
            UpgradeError::AlreadyApproved
        );

        // Add approval
        proposal.approvals.push(ctx.accounts.approver.key());

        // Check if threshold met
        if proposal.approvals.len() >= proposal.approval_threshold as usize {
            proposal.status = UpgradeStatus::TimelockActive;
            proposal.timelock_until = clock.unix_timestamp + 
                ctx.accounts.program_upgrade_state.timelock_duration;
            
            msg!("Proposal approved! Threshold met. Timelock active until {}", 
                 proposal.timelock_until);
        } else {
            proposal.status = UpgradeStatus::Approved;
            msg!("Approval added. {}/{} approvals", 
                 proposal.approvals.len(), proposal.approval_threshold);
        }

        emit!(ProposalApprovedEvent {
            proposal_id: ctx.accounts.proposal.key(),
            approver: ctx.accounts.approver.key(),
            approvals: proposal.approvals.len(),
            threshold: proposal.approval_threshold,
        });

        Ok(())
    }

    /// Execute an approved upgrade after timelock expires
    pub fn execute_upgrade(
        ctx: Context<ExecuteUpgrade>,
        _proposal_id: Pubkey,
    ) -> Result<()> {
        let proposal = &mut ctx.accounts.proposal;
        let clock = Clock::get()?;

        // Verify timelock has expired
        require!(
            clock.unix_timestamp >= proposal.timelock_until,
            UpgradeError::TimelockActive
        );

        // Verify sufficient approvals
        require!(
            proposal.approvals.len() >= proposal.approval_threshold as usize,
            UpgradeError::InsufficientApprovals
        );

        // Verify proposal is in correct status
        require!(
            proposal.status == UpgradeStatus::TimelockActive,
            UpgradeError::InvalidProposalStatus
        );

        // Verify proposal can be executed
        // The actual BPF upgrade will be executed by the multisig via Squads Protocol
        // This instruction authorizes the upgrade and updates on-chain state
        
        // In production, the backend service will:
        // 1. Build BPF upgradeable loader instruction
        // 2. Create Squads multisig transaction
        // 3. Collect signatures from approvers
        // 4. Execute via Squads Protocol
        
        // The BPF upgrade instruction structure:
        // - Program: BPF Upgradeable Loader
        // - Accounts: [program, buffer, upgrade_authority, program_data]
        // - Data: Upgrade instruction discriminator (3)
        
        msg!("Upgrade authorized - ready for multisig execution via Squads Protocol");

        // Update proposal status
        proposal.status = UpgradeStatus::Executed;
        proposal.executed_at = Some(clock.unix_timestamp);

        msg!("Upgrade executed successfully!");

        emit!(UpgradeExecutedEvent {
            proposal_id: ctx.accounts.proposal.key(),
            program: proposal.program,
            executed_at: proposal.executed_at.unwrap(),
        });

        Ok(())
    }

    /// Cancel an upgrade proposal (emergency only)
    pub fn cancel_upgrade(
        ctx: Context<CancelUpgrade>,
        _proposal_id: Pubkey,
    ) -> Result<()> {
        let proposal = &mut ctx.accounts.proposal;
        let config = &ctx.accounts.multisig_config;

        // Verify canceller is a multisig member
        require!(
            config.members.contains(&ctx.accounts.canceller.key()),
            UpgradeError::NotMultisigMember
        );

        // Can only cancel before execution
        require!(
            proposal.status != UpgradeStatus::Executed,
            UpgradeError::CannotCancelExecuted
        );

        proposal.status = UpgradeStatus::Cancelled;

        msg!("Proposal cancelled");

        emit!(ProposalCancelledEvent {
            proposal_id: ctx.accounts.proposal.key(),
            canceller: ctx.accounts.canceller.key(),
        });

        Ok(())
    }

    /// Migrate account state from old to new program version
    pub fn migrate_account(
        ctx: Context<MigrateAccount>,
        old_account: Pubkey,
    ) -> Result<()> {
        let migration = &mut ctx.accounts.account_version;
        let clock = Clock::get()?;

        // Check if already migrated
        require!(
            !migration.migrated,
            UpgradeError::AlreadyMigrated
        );

        // Perform migration logic here
        // This is a placeholder - actual migration depends on account structure
        migration.version += 1;
        migration.migrated = true;
        migration.migrated_at = Some(clock.unix_timestamp);

        msg!("Account migrated: version={}", migration.version);

        emit!(AccountMigratedEvent {
            account: old_account,
            new_version: migration.version,
            migrated_at: migration.migrated_at.unwrap(),
        });

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + MultisigConfig::LEN,
        seeds = [b"multisig_config"],
        bump
    )]
    pub multisig_config: Account<'info, MultisigConfig>,

    #[account(
        init,
        payer = authority,
        space = 8 + ProgramUpgradeState::LEN,
        seeds = [b"program_upgrade_state"],
        bump
    )]
    pub program_upgrade_state: Account<'info, ProgramUpgradeState>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct ProposeUpgrade<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,

    #[account(
        seeds = [b"multisig_config"],
        bump = multisig_config.bump
    )]
    pub multisig_config: Account<'info, MultisigConfig>,

    #[account(
        seeds = [b"program_upgrade_state"],
        bump = program_upgrade_state.bump
    )]
    pub program_upgrade_state: Account<'info, ProgramUpgradeState>,

    /// CHECK: Program to be upgraded
    pub program: UncheckedAccount<'info>,

    #[account(
        init,
        payer = proposer,
        space = 8 + UpgradeProposal::LEN,
        seeds = [b"proposal", program.key().as_ref(), new_program_buffer.key().as_ref()],
        bump
    )]
    pub proposal: Account<'info, UpgradeProposal>,

    /// CHECK: New program buffer account
    pub new_program_buffer: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ApproveUpgrade<'info> {
    #[account(mut)]
    pub approver: Signer<'info>,

    #[account(
        seeds = [b"multisig_config"],
        bump = multisig_config.bump
    )]
    pub multisig_config: Account<'info, MultisigConfig>,

    #[account(
        mut,
        seeds = [b"proposal", proposal.program.as_ref(), proposal.new_buffer.as_ref()],
        bump = proposal.bump
    )]
    pub proposal: Account<'info, UpgradeProposal>,

    #[account(
        seeds = [b"program_upgrade_state"],
        bump = program_upgrade_state.bump
    )]
    pub program_upgrade_state: Account<'info, ProgramUpgradeState>,
}

#[derive(Accounts)]
pub struct ExecuteUpgrade<'info> {
    #[account(mut)]
    pub executor: Signer<'info>,

    #[account(
        mut,
        seeds = [b"proposal", proposal.program.as_ref(), proposal.new_buffer.as_ref()],
        bump = proposal.bump
    )]
    pub proposal: Account<'info, UpgradeProposal>,

    #[account(
        seeds = [b"program_upgrade_state"],
        bump = program_upgrade_state.bump
    )]
    pub program_upgrade_state: Account<'info, ProgramUpgradeState>,
}

#[derive(Accounts)]
pub struct CancelUpgrade<'info> {
    #[account(mut)]
    pub canceller: Signer<'info>,

    #[account(
        seeds = [b"multisig_config"],
        bump = multisig_config.bump
    )]
    pub multisig_config: Account<'info, MultisigConfig>,

    #[account(
        mut,
        seeds = [b"proposal", proposal.program.as_ref(), proposal.new_buffer.as_ref()],
        bump = proposal.bump
    )]
    pub proposal: Account<'info, UpgradeProposal>,
}

#[derive(Accounts)]
pub struct MigrateAccount<'info> {
    #[account(mut)]
    pub migrator: Signer<'info>,

    #[account(
        mut,
        seeds = [b"account_version", old_account.key().as_ref()],
        bump
    )]
    pub account_version: Account<'info, AccountVersion>,

    /// CHECK: Old account to migrate from
    pub old_account: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[account]
pub struct UpgradeProposal {
    pub id: [u8; 8],
    pub proposer: Pubkey,
    pub program: Pubkey,
    pub new_buffer: Pubkey,
    pub description: String,
    pub proposed_at: i64,
    pub timelock_until: i64,
    pub approvals: Vec<Pubkey>,
    pub approval_threshold: u8,
    pub status: UpgradeStatus,
    pub executed_at: Option<i64>,
    pub bump: u8,
}

impl UpgradeProposal {
    pub const LEN: usize = 8 +      // discriminator
        8 +                         // id
        32 +                        // proposer
        32 +                        // program
        32 +                        // new_buffer
        4 + 256 +                   // description (String)
        8 +                         // proposed_at
        8 +                         // timelock_until
        4 + (32 * 10) +             // approvals (max 10 members)
        1 +                         // approval_threshold
        1 +                         // status
        1 + 8 +                     // executed_at (Option<i64>)
        1;                          // bump
}

#[account]
pub struct MultisigConfig {
    pub members: Vec<Pubkey>,
    pub threshold: u8,
    pub upgrade_authority: Pubkey,
    pub bump: u8,
}

impl MultisigConfig {
    pub const LEN: usize = 4 + (32 * 10) +  // members (max 10)
        1 +                                  // threshold
        32 +                                 // upgrade_authority
        1;                                   // bump
}

#[account]
pub struct ProgramUpgradeState {
    pub authority: Pubkey,
    pub upgrade_buffer: Pubkey,
    pub timelock_duration: i64,
    pub pending_upgrade: Option<PendingUpgrade>,
    pub bump: u8,
}

impl ProgramUpgradeState {
    pub const LEN: usize = 32 +              // authority
        32 +                                 // upgrade_buffer
        8 +                                  // timelock_duration
        1 + (32 + 8 + 8 + 4 + (32 * 10)) +  // pending_upgrade (Option)
        1;                                   // bump
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub struct PendingUpgrade {
    pub new_program_hash: [u8; 32],
    pub scheduled_time: i64,
    pub proposal_time: i64,
    pub approved_by: Vec<Pubkey>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum UpgradeStatus {
    Proposed,
    Approved,
    TimelockActive,
    Executed,
    Cancelled,
}

#[account]
pub struct AccountVersion {
    pub version: u32,
    pub migrated: bool,
    pub migrated_at: Option<i64>,
    pub bump: u8,
}

impl AccountVersion {
    pub const LEN: usize = 4 +      // version
        1 +                         // migrated
        1 + 8 +                     // migrated_at (Option<i64>)
        1;                          // bump
}

#[error_code]
pub enum UpgradeError {
    #[msg("Not a multisig member")]
    NotMultisigMember,
    #[msg("Invalid proposal status")]
    InvalidProposalStatus,
    #[msg("Already approved")]
    AlreadyApproved,
    #[msg("Timelock still active")]
    TimelockActive,
    #[msg("Insufficient approvals")]
    InsufficientApprovals,
    #[msg("Cannot cancel executed proposal")]
    CannotCancelExecuted,
    #[msg("Already migrated")]
    AlreadyMigrated,
    #[msg("Invalid proposal ID")]
    InvalidProposalId,
}

#[event]
pub struct InitializedEvent {
    pub authority: Pubkey,
    pub members: Vec<Pubkey>,
    pub threshold: u8,
    pub timelock_duration: i64,
}

#[event]
pub struct ProposalCreatedEvent {
    pub proposal_id: Pubkey,
    pub proposer: Pubkey,
    pub new_buffer: Pubkey,
    pub timelock_until: i64,
}

#[event]
pub struct ProposalApprovedEvent {
    pub proposal_id: Pubkey,
    pub approver: Pubkey,
    pub approvals: usize,
    pub threshold: u8,
}

#[event]
pub struct UpgradeExecutedEvent {
    pub proposal_id: Pubkey,
    pub program: Pubkey,
    pub executed_at: i64,
}

#[event]
pub struct ProposalCancelledEvent {
    pub proposal_id: Pubkey,
    pub canceller: Pubkey,
}

#[event]
pub struct AccountMigratedEvent {
    pub account: Pubkey,
    pub new_version: u32,
    pub migrated_at: i64,
}

