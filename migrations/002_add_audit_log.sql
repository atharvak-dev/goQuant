-- Audit Log Table for comprehensive tracking

CREATE TABLE IF NOT EXISTS audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type VARCHAR(50) NOT NULL,
    proposal_id VARCHAR(255),
    actor VARCHAR(44) NOT NULL, -- Who performed the action
    action VARCHAR(100) NOT NULL,
    details JSONB,
    timestamp TIMESTAMP NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_audit_log_event_type ON audit_log(event_type);
CREATE INDEX IF NOT EXISTS idx_audit_log_proposal ON audit_log(proposal_id);
CREATE INDEX IF NOT EXISTS idx_audit_log_timestamp ON audit_log(timestamp);

-- Event types:
-- 'proposal_created'
-- 'proposal_approved'
-- 'proposal_executed'
-- 'proposal_cancelled'
-- 'migration_started'
-- 'migration_completed'
-- 'rollback_initiated'
-- 'upgrade_verified'
-- 'timelock_expired'

