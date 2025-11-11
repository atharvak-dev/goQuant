use crate::error::UpgradeError;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct TimelockManager {
    timelocks: Arc<Mutex<HashMap<String, i64>>>,
}

impl TimelockManager {
    pub async fn new() -> Result<Self, UpgradeError> {
        Ok(Self {
            timelocks: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub async fn set_timelock(&self, proposal_id: String, duration_seconds: i64) -> Result<(), UpgradeError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let timelock_end = now + duration_seconds;

        let mut timelocks = self.timelocks.lock().await;
        timelocks.insert(proposal_id, timelock_end);

        Ok(())
    }

    pub async fn get_timelock_end(&self, proposal_id: &str) -> Result<i64, UpgradeError> {
        let timelocks = self.timelocks.lock().await;
        timelocks
            .get(proposal_id)
            .copied()
            .ok_or_else(|| UpgradeError::ProposalNotFound(proposal_id.to_string()))
    }

    pub async fn is_timelock_expired(&self, proposal_id: &str) -> Result<bool, UpgradeError> {
        let timelock_end = self.get_timelock_end(proposal_id).await?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        Ok(now >= timelock_end)
    }

    pub async fn get_remaining_time(&self, proposal_id: &str) -> Result<i64, UpgradeError> {
        let timelock_end = self.get_timelock_end(proposal_id).await?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let remaining = timelock_end - now;
        Ok(remaining.max(0))
    }

    pub async fn monitor_timelocks(&self) {
        // Background task to monitor timelocks and send alerts
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;

            let timelocks = self.timelocks.lock().await;
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;

            for (proposal_id, timelock_end) in timelocks.iter() {
                let remaining = timelock_end - now;
                if remaining > 0 && remaining < 3600 {
                    // Less than 1 hour remaining
                    tracing::info!(
                        "Timelock expiring soon: {} ({} seconds remaining)",
                        proposal_id,
                        remaining
                    );
                }
            }
        }
    }
}

