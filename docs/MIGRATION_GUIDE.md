# Migration Guide

## Overview

This guide explains how to plan and execute account state migrations when upgrading Solana programs.

## Migration Strategy

### Versioning Approach

Each account maintains a version number:
- Old accounts: version N
- New accounts: version N+1
- Migration tracks which accounts have been migrated

### Migration Types

1. **In-Place Migration**: Update existing account structure
2. **New Account Migration**: Create new account, copy data
3. **Lazy Migration**: Migrate on first access
4. **Batch Migration**: Process multiple accounts in parallel

## Planning a Migration

### Step 1: Analyze Account Changes

Identify:
- Which accounts need migration
- What data structure changes
- Compatibility requirements
- Data transformation logic

### Step 2: Design Migration Logic

```rust
// Example migration function
pub fn migrate_account(
    old_account: &OldAccountData,
) -> Result<NewAccountData, MigrationError> {
    // Transform data from old to new format
    Ok(NewAccountData {
        // ... transformed fields
    })
}
```

### Step 3: Test Migration

- Test with sample data
- Verify data integrity
- Test edge cases
- Performance testing

### Step 4: Deploy Migration Program

- Deploy new program version
- Include migration instruction
- Test on devnet first

## Migration Execution

### Automatic Migration

The system can automatically migrate accounts:
1. Identify accounts needing migration
2. Batch process migrations
3. Verify each migration
4. Track progress

### Manual Migration

For critical accounts, manual migration:
1. Call `migrate_account` instruction
2. Verify migration success
3. Update version tracking

### Migration Progress Tracking

Monitor via:
- Database migration_progress table
- API endpoint: `GET /migration/progress`
- WebSocket notifications

## Data Transformation Patterns

### Pattern 1: Field Addition

```rust
// Old structure
struct OldAccount {
    field1: u64,
    field2: String,
}

// New structure
struct NewAccount {
    field1: u64,
    field2: String,
    field3: u64, // New field with default
}
```

### Pattern 2: Field Removal

```rust
// Remove deprecated fields
// Ensure no data loss for active fields
```

### Pattern 3: Type Conversion

```rust
// Convert between types
// Example: u32 -> u64
```

### Pattern 4: Structure Reorganization

```rust
// Reorganize nested structures
// Flatten or nest as needed
```

## Verification

### Data Integrity Checks

1. **Field Validation**: Verify all fields migrated correctly
2. **Type Checking**: Ensure type conversions valid
3. **Range Validation**: Check value ranges
4. **Relationship Validation**: Verify account relationships

### Migration Verification

```rust
pub fn verify_migration(
    old_account: &OldAccountData,
    new_account: &NewAccountData,
) -> Result<bool, VerificationError> {
    // Compare critical fields
    // Verify transformations
    // Check constraints
    Ok(true)
}
```

## Error Handling

### Migration Failures

- Log failed migrations
- Retry mechanism
- Manual intervention option
- Rollback capability

### Partial Migrations

- Track which accounts migrated
- Resume from last checkpoint
- Handle partial state

## Best Practices

1. **Backward Compatibility**: Support both old and new formats during transition
2. **Idempotency**: Migration can be safely retried
3. **Atomicity**: Migrate account completely or not at all
4. **Performance**: Batch process for efficiency
5. **Monitoring**: Track progress and errors
6. **Testing**: Thoroughly test before production

## Example Migration

### Scenario: Adding a new field to user account

```rust
// 1. Old account structure
#[account]
pub struct UserAccount {
    pub owner: Pubkey,
    pub balance: u64,
}

// 2. New account structure
#[account]
pub struct UserAccount {
    pub owner: Pubkey,
    pub balance: u64,
    pub last_active: i64, // New field
}

// 3. Migration function
pub fn migrate_user_account(
    ctx: Context<MigrateAccount>,
) -> Result<()> {
    let old_account = &ctx.accounts.old_account;
    let new_account = &mut ctx.accounts.new_account;
    
    // Copy existing fields
    new_account.owner = old_account.owner;
    new_account.balance = old_account.balance;
    
    // Set default for new field
    new_account.last_active = Clock::get()?.unix_timestamp;
    
    // Mark as migrated
    ctx.accounts.account_version.migrated = true;
    
    Ok(())
}
```

## Migration Checklist

- [ ] Analyze account changes
- [ ] Design migration logic
- [ ] Write migration tests
- [ ] Deploy to devnet
- [ ] Test migration on devnet
- [ ] Deploy to mainnet
- [ ] Monitor migration progress
- [ ] Verify all accounts migrated
- [ ] Remove old account support (after grace period)

