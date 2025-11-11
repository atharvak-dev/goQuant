-- GoQuant Upgrade Management System Database Schema

-- Upgrade Proposals Table
CREATE TABLE IF NOT EXISTS upgrade_proposals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    proposal_id VARCHAR(255) UNIQUE NOT NULL,
    proposer VARCHAR(44) NOT NULL, -- Solana pubkey
    program VARCHAR(44) NOT NULL, -- Program to upgrade
    new_buffer VARCHAR(44) NOT NULL, -- New program buffer
    description TEXT NOT NULL,
    proposed_at TIMESTAMP NOT NULL DEFAULT NOW(),
    timelock_until TIMESTAMP NOT NULL,
    approval_threshold INTEGER NOT NULL,
    status VARCHAR(20) NOT NULL CHECK (status IN ('proposed', 'approved', 'timelock_active', 'executed', 'cancelled')),
    executed_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Approval History Table
CREATE TABLE IF NOT EXISTS approval_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    proposal_id VARCHAR(255) NOT NULL REFERENCES upgrade_proposals(proposal_id),
    approver VARCHAR(44) NOT NULL, -- Multisig member pubkey
    approved_at TIMESTAMP NOT NULL DEFAULT NOW(),
    signature VARCHAR(88), -- Transaction signature
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Timelock Tracking Table
CREATE TABLE IF NOT EXISTS timelock_tracking (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    proposal_id VARCHAR(255) UNIQUE NOT NULL REFERENCES upgrade_proposals(proposal_id),
    timelock_start TIMESTAMP NOT NULL,
    timelock_end TIMESTAMP NOT NULL,
    duration_seconds BIGINT NOT NULL,
    status VARCHAR(20) NOT NULL CHECK (status IN ('active', 'expired', 'cancelled')),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Migration Progress Table
CREATE TABLE IF NOT EXISTS migration_progress (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    migration_id VARCHAR(255) UNIQUE NOT NULL,
    proposal_id VARCHAR(255) REFERENCES upgrade_proposals(proposal_id),
    total_accounts INTEGER NOT NULL DEFAULT 0,
    migrated_accounts INTEGER NOT NULL DEFAULT 0,
    failed_accounts INTEGER NOT NULL DEFAULT 0,
    status VARCHAR(20) NOT NULL CHECK (status IN ('not_started', 'in_progress', 'completed', 'failed')),
    started_at TIMESTAMP NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Account Migrations Table
CREATE TABLE IF NOT EXISTS account_migrations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    migration_id VARCHAR(255) NOT NULL REFERENCES migration_progress(migration_id),
    account_pubkey VARCHAR(44) NOT NULL,
    old_version INTEGER NOT NULL,
    new_version INTEGER NOT NULL,
    migrated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    status VARCHAR(20) NOT NULL CHECK (status IN ('pending', 'migrated', 'failed', 'verified')),
    error_message TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Upgrade History Table
CREATE TABLE IF NOT EXISTS upgrade_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    proposal_id VARCHAR(255) NOT NULL REFERENCES upgrade_proposals(proposal_id),
    program VARCHAR(44) NOT NULL,
    old_program_hash VARCHAR(64),
    new_program_hash VARCHAR(64),
    executed_at TIMESTAMP NOT NULL,
    success BOOLEAN NOT NULL,
    error_message TEXT,
    rollback_required BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Rollback Events Table
CREATE TABLE IF NOT EXISTS rollback_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    proposal_id VARCHAR(255) NOT NULL REFERENCES upgrade_proposals(proposal_id),
    old_program_id VARCHAR(44) NOT NULL,
    rollback_reason TEXT NOT NULL,
    rollback_at TIMESTAMP NOT NULL DEFAULT NOW(),
    positions_closed INTEGER DEFAULT 0,
    funds_returned BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Multisig Configuration Table
CREATE TABLE IF NOT EXISTS multisig_config (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    multisig_address VARCHAR(44) UNIQUE NOT NULL,
    members TEXT[] NOT NULL, -- Array of pubkeys
    threshold INTEGER NOT NULL,
    upgrade_authority VARCHAR(44) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Program Versions Table
CREATE TABLE IF NOT EXISTS program_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    program_id VARCHAR(44) NOT NULL,
    version INTEGER NOT NULL,
    program_hash VARCHAR(64) NOT NULL,
    deployed_at TIMESTAMP NOT NULL DEFAULT NOW(),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_proposals_status ON upgrade_proposals(status);
CREATE INDEX IF NOT EXISTS idx_proposals_timelock ON upgrade_proposals(timelock_until);
CREATE INDEX IF NOT EXISTS idx_approvals_proposal ON approval_history(proposal_id);
CREATE INDEX IF NOT EXISTS idx_migrations_status ON migration_progress(status);
CREATE INDEX IF NOT EXISTS idx_account_migrations_migration ON account_migrations(migration_id);
CREATE INDEX IF NOT EXISTS idx_upgrade_history_program ON upgrade_history(program);
CREATE INDEX IF NOT EXISTS idx_program_versions_active ON program_versions(program_id, is_active);

-- Triggers for updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_upgrade_proposals_updated_at BEFORE UPDATE ON upgrade_proposals
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_timelock_tracking_updated_at BEFORE UPDATE ON timelock_tracking
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_migration_progress_updated_at BEFORE UPDATE ON migration_progress
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_multisig_config_updated_at BEFORE UPDATE ON multisig_config
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

