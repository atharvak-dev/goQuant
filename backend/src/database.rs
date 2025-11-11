use crate::error::UpgradeError;
use sqlx::{PgPool, Row};
use serde_json::Value;

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, UpgradeError> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self { pool })
    }

    pub async fn save_proposal(
        &self,
        proposal_id: &str,
        proposer: &str,
        program: &str,
        new_buffer: &str,
        description: &str,
        timelock_until: i64,
        approval_threshold: i32,
    ) -> Result<(), UpgradeError> {
        sqlx::query!(
            r#"
            INSERT INTO upgrade_proposals 
            (proposal_id, proposer, program, new_buffer, description, timelock_until, approval_threshold, status)
            VALUES ($1, $2, $3, $4, $5, to_timestamp($6), $7, 'proposed')
            "#,
            proposal_id,
            proposer,
            program,
            new_buffer,
            description,
            timelock_until,
            approval_threshold
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn add_approval(
        &self,
        proposal_id: &str,
        approver: &str,
        signature: Option<&str>,
    ) -> Result<(), UpgradeError> {
        sqlx::query!(
            r#"
            INSERT INTO approval_history (proposal_id, approver, signature)
            VALUES ($1, $2, $3)
            "#,
            proposal_id,
            approver,
            signature
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_proposal_status(
        &self,
        proposal_id: &str,
        status: &str,
        executed_at: Option<i64>,
    ) -> Result<(), UpgradeError> {
        if let Some(executed_at) = executed_at {
            sqlx::query!(
                r#"
                UPDATE upgrade_proposals 
                SET status = $1, executed_at = to_timestamp($2)
                WHERE proposal_id = $3
                "#,
                status,
                executed_at,
                proposal_id
            )
            .execute(&self.pool)
            .await?;
        } else {
            sqlx::query!(
                r#"
                UPDATE upgrade_proposals 
                SET status = $1
                WHERE proposal_id = $2
                "#,
                status,
                proposal_id
            )
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    pub async fn get_proposal(&self, proposal_id: &str) -> Result<Value, UpgradeError> {
        let row = sqlx::query!(
            r#"
            SELECT proposal_id, proposer, program, new_buffer, description,
                   EXTRACT(epoch FROM proposed_at) as proposed_at,
                   EXTRACT(epoch FROM timelock_until) as timelock_until,
                   approval_threshold, status,
                   EXTRACT(epoch FROM executed_at) as executed_at
            FROM upgrade_proposals
            WHERE proposal_id = $1
            "#,
            proposal_id
        )
        .fetch_one(&self.pool)
        .await?;

        let approvals = sqlx::query!(
            "SELECT approver FROM approval_history WHERE proposal_id = $1",
            proposal_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(serde_json::json!({
            "id": row.proposal_id,
            "proposer": row.proposer,
            "program": row.program,
            "new_buffer": row.new_buffer,
            "description": row.description,
            "proposed_at": row.proposed_at,
            "timelock_until": row.timelock_until,
            "approval_threshold": row.approval_threshold,
            "status": row.status,
            "executed_at": row.executed_at,
            "approvals": approvals.iter().map(|a| &a.approver).collect::<Vec<_>>(),
        }))
    }

    pub async fn list_proposals(&self) -> Result<Vec<Value>, UpgradeError> {
        let rows = sqlx::query!(
            r#"
            SELECT proposal_id, proposer, program, new_buffer, description,
                   EXTRACT(epoch FROM proposed_at) as proposed_at,
                   EXTRACT(epoch FROM timelock_until) as timelock_until,
                   approval_threshold, status,
                   EXTRACT(epoch FROM executed_at) as executed_at
            FROM upgrade_proposals
            ORDER BY proposed_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut proposals = Vec::new();
        for row in rows {
            let approvals = sqlx::query!(
                "SELECT approver FROM approval_history WHERE proposal_id = $1",
                row.proposal_id
            )
            .fetch_all(&self.pool)
            .await?;

            proposals.push(serde_json::json!({
                "id": row.proposal_id,
                "proposer": row.proposer,
                "program": row.program,
                "new_buffer": row.new_buffer,
                "description": row.description,
                "proposed_at": row.proposed_at,
                "timelock_until": row.timelock_until,
                "approval_threshold": row.approval_threshold,
                "status": row.status,
                "executed_at": row.executed_at,
                "approvals": approvals.iter().map(|a| &a.approver).collect::<Vec<_>>(),
            }));
        }

        Ok(proposals)
    }

    pub async fn save_migration_progress(
        &self,
        migration_id: &str,
        proposal_id: Option<&str>,
        total_accounts: i32,
        status: &str,
    ) -> Result<(), UpgradeError> {
        sqlx::query!(
            r#"
            INSERT INTO migration_progress 
            (migration_id, proposal_id, total_accounts, status)
            VALUES ($1, $2, $3, $4)
            "#,
            migration_id,
            proposal_id,
            total_accounts,
            status
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_migration_progress(
        &self,
        migration_id: &str,
        migrated_accounts: i32,
        failed_accounts: i32,
        status: &str,
    ) -> Result<(), UpgradeError> {
        sqlx::query!(
            r#"
            UPDATE migration_progress 
            SET migrated_accounts = $1, failed_accounts = $2, status = $3,
                completed_at = CASE WHEN $3 IN ('completed', 'failed') THEN NOW() ELSE completed_at END
            WHERE migration_id = $4
            "#,
            migrated_accounts,
            failed_accounts,
            status,
            migration_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn record_upgrade_history(
        &self,
        proposal_id: &str,
        program: &str,
        old_program_hash: Option<&str>,
        new_program_hash: &str,
        success: bool,
        error_message: Option<&str>,
    ) -> Result<(), UpgradeError> {
        sqlx::query!(
            r#"
            INSERT INTO upgrade_history 
            (proposal_id, program, old_program_hash, new_program_hash, executed_at, success, error_message)
            VALUES ($1, $2, $3, $4, NOW(), $5, $6)
            "#,
            proposal_id,
            program,
            old_program_hash,
            new_program_hash,
            success,
            error_message
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn record_rollback_event(
        &self,
        proposal_id: &str,
        old_program_id: &str,
        rollback_reason: &str,
        positions_closed: i32,
        funds_returned: bool,
    ) -> Result<(), UpgradeError> {
        sqlx::query!(
            r#"
            INSERT INTO rollback_events 
            (proposal_id, old_program_id, rollback_reason, positions_closed, funds_returned)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            proposal_id,
            old_program_id,
            rollback_reason,
            positions_closed,
            funds_returned
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}