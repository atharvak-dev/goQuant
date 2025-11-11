# Smart Contract Documentation

## Overview

The GoQuant Upgrade Manager is an Anchor program that provides secure, governance-controlled program upgrades with timelock protection and state migration capabilities.

## Program ID

```
UpgrMgr11111111111111111111111111111111
```

## Account Structures

### UpgradeProposal

Stores upgrade proposal metadata and tracks approval status.

```rust
#[account]
pub struct UpgradeProposal {
    pub id: [u8; 8],                    // Unique proposal identifier
    pub proposer: Pubkey,               // Who proposed the upgrade
    pub program: Pubkey,                // Program to be upgraded
    pub new_buffer: Pubkey,             // New program buffer account
    pub description: String,            // Upgrade description
    pub proposed_at: i64,               // Proposal timestamp
    pub timelock_until: i64,            // When timelock expires
    pub approvals: Vec<Pubkey>,         // List of approvers
    pub approval_threshold: u8,         // Required approvals
    pub status: UpgradeStatus,          // Current status
    pub executed_at: Option<i64>,       // Execution timestamp
    pub bump: u8,                       // PDA bump
}
```

**PDA Seeds**: `["proposal", program.key(), new_buffer.key()]`

### MultisigConfig

Configuration for multisig governance.

```rust
#[account]
pub struct MultisigConfig {
    pub members: Vec<Pubkey>,           // Multisig member pubkeys
    pub threshold: u8,                  // Approval threshold
    pub upgrade_authority: Pubkey,      // Upgrade authority
    pub bump: u8,                       // PDA bump
}
```

**PDA Seeds**: `["multisig_config"]`

### ProgramUpgradeState

Global upgrade state and configuration.

```rust
#[account]
pub struct ProgramUpgradeState {
    pub authority: Pubkey,              // Upgrade authority
    pub upgrade_buffer: Pubkey,         // Current upgrade buffer
    pub timelock_duration: i64,         // Timelock duration in seconds
    pub pending_upgrade: Option<PendingUpgrade>, // Current pending upgrade
    pub bump: u8,                       // PDA bump
}
```

**PDA Seeds**: `["program_upgrade_state"]`

### AccountVersion

Tracks account migration status.

```rust
#[account]
pub struct AccountVersion {
    pub version: u32,                   // Account version number
    pub migrated: bool,                 // Migration status
    pub migrated_at: Option<i64>,       // Migration timestamp
    pub bump: u8,                       // PDA bump
}
```

**PDA Seeds**: `["account_version", account.key()]`

## Enums

### UpgradeStatus

```rust
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum UpgradeStatus {
    Proposed,        // Initial proposal state
    Approved,        // Has approvals but not threshold
    TimelockActive,  // Threshold met, timelock active
    Executed,        // Upgrade executed
    Cancelled,       // Proposal cancelled
}
```

## Instructions

### initialize

Initializes the upgrade manager with multisig configuration.

```rust
pub fn initialize(
    ctx: Context<Initialize>,
    members: Vec<Pubkey>,
    threshold: u8,
    timelock_duration: i64,
) -> Result<()>
```

**Accounts:**
- `authority` (signer, mut): Upgrade authority
- `multisig_config` (init): Multisig configuration account
- `program_upgrade_state` (init): Program upgrade state account
- `system_program`: System program
- `rent`: Rent sysvar

**Validation:**
- Authority must sign
- Members list must not be empty
- Threshold must be <= members count
- Timelock duration must be >= 48 hours

### propose_upgrade

Creates a new upgrade proposal.

```rust
pub fn propose_upgrade(
    ctx: Context<ProposeUpgrade>,
    new_program_buffer: Pubkey,
    description: String,
) -> Result<()>
```

**Accounts:**
- `proposer` (signer, mut): Proposal creator
- `multisig_config`: Multisig configuration
- `program_upgrade_state`: Program upgrade state
- `program`: Program to upgrade
- `proposal` (init): New proposal account
- `new_program_buffer`: New program buffer
- `system_program`: System program

**Validation:**
- Proposer must be multisig member
- Buffer account must exist
- Description must not be empty

### approve_upgrade

Approves an upgrade proposal.

```rust
pub fn approve_upgrade(
    ctx: Context<ApproveUpgrade>,
    proposal_id: Pubkey,
) -> Result<()>
```

**Accounts:**
- `approver` (signer, mut): Multisig member approving
- `multisig_config`: Multisig configuration
- `proposal` (mut): Proposal to approve
- `program_upgrade_state`: Program upgrade state

**Validation:**
- Approver must be multisig member
- Proposal must be in valid status
- Approver must not have already approved
- Updates status to TimelockActive when threshold met

### execute_upgrade

Executes an approved upgrade after timelock expires.

```rust
pub fn execute_upgrade(
    ctx: Context<ExecuteUpgrade>,
    proposal_id: Pubkey,
) -> Result<()>
```

**Accounts:**
- `executor` (signer, mut): Executor (any account)
- `proposal` (mut): Proposal to execute
- `program_upgrade_state`: Program upgrade state

**Validation:**
- Timelock must have expired
- Sufficient approvals must exist
- Proposal must be in TimelockActive status
- Marks proposal as executed

### cancel_upgrade

Cancels an upgrade proposal (emergency only).

```rust
pub fn cancel_upgrade(
    ctx: Context<CancelUpgrade>,
    proposal_id: Pubkey,
) -> Result<()>
```

**Accounts:**
- `canceller` (signer, mut): Multisig member cancelling
- `multisig_config`: Multisig configuration
- `proposal` (mut): Proposal to cancel

**Validation:**
- Canceller must be multisig member
- Proposal must not be executed
- Sets status to Cancelled

### migrate_account

Migrates account state from old to new program version.

```rust
pub fn migrate_account(
    ctx: Context<MigrateAccount>,
    old_account: Pubkey,
) -> Result<()>
```

**Accounts:**
- `migrator` (signer, mut): Account performing migration
- `account_version` (mut): Account version tracking
- `old_account`: Account to migrate from
- `system_program`: System program

**Validation:**
- Account must not already be migrated
- Updates version and migration status

## Events

### InitializedEvent

Emitted when upgrade manager is initialized.

```rust
#[event]
pub struct InitializedEvent {
    pub authority: Pubkey,
    pub members: Vec<Pubkey>,
    pub threshold: u8,
    pub timelock_duration: i64,
}
```

### ProposalCreatedEvent

Emitted when new proposal is created.

```rust
#[event]
pub struct ProposalCreatedEvent {
    pub proposal_id: Pubkey,
    pub proposer: Pubkey,
    pub new_buffer: Pubkey,
    pub timelock_until: i64,
}
```

### ProposalApprovedEvent

Emitted when proposal receives approval.

```rust
#[event]
pub struct ProposalApprovedEvent {
    pub proposal_id: Pubkey,
    pub approver: Pubkey,
    pub approvals: usize,
    pub threshold: u8,
}
```

### UpgradeExecutedEvent

Emitted when upgrade is executed.

```rust
#[event]
pub struct UpgradeExecutedEvent {
    pub proposal_id: Pubkey,
    pub program: Pubkey,
    pub executed_at: i64,
}
```

### ProposalCancelledEvent

Emitted when proposal is cancelled.

```rust
#[event]
pub struct ProposalCancelledEvent {
    pub proposal_id: Pubkey,
    pub canceller: Pubkey,
}
```

### AccountMigratedEvent

Emitted when account is migrated.

```rust
#[event]
pub struct AccountMigratedEvent {
    pub account: Pubkey,
    pub new_version: u32,
    pub migrated_at: i64,
}
```

## Error Codes

```rust
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
```

## Security Considerations

1. **Multisig Protection**: All critical operations require multisig approval
2. **Timelock Enforcement**: 48-hour minimum delay before execution
3. **Authority Validation**: All operations verify caller permissions
4. **Replay Protection**: Proposals use unique PDAs
5. **State Validation**: Strict status transitions enforced

## Usage Examples

### Initialize System

```typescript
await program.methods
  .initialize(members, threshold, timelockDuration)
  .accounts({
    authority: authority.publicKey,
    multisigConfig,
    programUpgradeState,
    systemProgram: SystemProgram.programId,
    rent: SYSVAR_RENT_PUBKEY,
  })
  .rpc();
```

### Create Proposal

```typescript
await program.methods
  .proposeUpgrade(newProgramBuffer, "Upgrade description")
  .accounts({
    proposer: proposer.publicKey,
    multisigConfig,
    programUpgradeState,
    program: programToUpgrade,
    proposal,
    newProgramBuffer,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

### Approve Proposal

```typescript
await program.methods
  .approveUpgrade(proposalId)
  .accounts({
    approver: approver.publicKey,
    multisigConfig,
    proposal,
    programUpgradeState,
  })
  .rpc();
```

## Integration Notes

- Designed to work with Squads Protocol for multisig execution
- Compatible with Solana BPF upgradeable loader
- Supports both immediate and lazy migration strategies
- Provides comprehensive event logging for off-chain monitoring