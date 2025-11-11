use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, warn};

pub type NotificationSender = broadcast::Sender<Notification>;

#[derive(Debug, Clone)]
pub struct Notification {
    pub notification_type: NotificationType,
    pub proposal_id: Option<String>,
    pub message: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone)]
pub enum NotificationType {
    ProposalCreated,
    ProposalApproved,
    TimelockExpired,
    UpgradeExecuted,
    MigrationProgress,
    RollbackInitiated,
}

impl NotificationType {
    fn as_str(&self) -> &'static str {
        match self {
            NotificationType::ProposalCreated => "proposal_created",
            NotificationType::ProposalApproved => "proposal_approved",
            NotificationType::TimelockExpired => "timelock_expired",
            NotificationType::UpgradeExecuted => "upgrade_executed",
            NotificationType::MigrationProgress => "migration_progress",
            NotificationType::RollbackInitiated => "rollback_initiated",
        }
    }
}

pub struct NotificationService {
    sender: NotificationSender,
}

impl NotificationService {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(100);
        Self { sender }
    }

    pub fn get_sender(&self) -> NotificationSender {
        self.sender.clone()
    }

    pub async fn notify(&self, notification: Notification) {
        let json = json!({
            "type": notification.notification_type.as_str(),
            "proposal_id": notification.proposal_id,
            "message": notification.message,
            "data": notification.data,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        });

        if let Err(e) = self.sender.send(notification.clone()) {
            warn!("Failed to send notification: {}", e);
        } else {
            info!("Notification sent: {}", json);
        }
    }

    pub async fn notify_proposal_created(&self, proposal_id: String, data: serde_json::Value) {
        self.notify(Notification {
            notification_type: NotificationType::ProposalCreated,
            proposal_id: Some(proposal_id),
            message: "New upgrade proposal created".to_string(),
            data,
        })
        .await;
    }

    pub async fn notify_proposal_approved(&self, proposal_id: String, approvals: usize, threshold: u8) {
        self.notify(Notification {
            notification_type: NotificationType::ProposalApproved,
            proposal_id: Some(proposal_id),
            message: format!("Proposal approved: {}/{}", approvals, threshold),
            data: json!({
                "approvals": approvals,
                "threshold": threshold,
            }),
        })
        .await;
    }

    pub async fn notify_timelock_expired(&self, proposal_id: String) {
        self.notify(Notification {
            notification_type: NotificationType::TimelockExpired,
            proposal_id: Some(proposal_id),
            message: "Timelock expired - upgrade can now be executed".to_string(),
            data: json!({}),
        })
        .await;
    }

    pub async fn notify_upgrade_executed(&self, proposal_id: String, program: String) {
        self.notify(Notification {
            notification_type: NotificationType::UpgradeExecuted,
            proposal_id: Some(proposal_id),
            message: "Upgrade executed successfully".to_string(),
            data: json!({
                "program": program,
            }),
        })
        .await;
    }

    pub async fn notify_migration_progress(
        &self,
        migration_id: String,
        progress: f64,
        migrated: usize,
        total: usize,
    ) {
        self.notify(Notification {
            notification_type: NotificationType::MigrationProgress,
            proposal_id: Some(migration_id),
            message: format!("Migration progress: {:.2}%", progress),
            data: json!({
                "progress_percent": progress,
                "migrated_accounts": migrated,
                "total_accounts": total,
            }),
        })
        .await;
    }
}

pub async fn handle_websocket(
    socket: WebSocket,
    mut receiver: broadcast::Receiver<Notification>,
) {
    let (mut sender, mut receiver_ws) = socket.split();

    // Spawn task to send notifications
    let mut send_task = tokio::spawn(async move {
        while let Ok(notification) = receiver.recv().await {
            let json = json!({
                "type": notification.notification_type.as_str(),
                "proposal_id": notification.proposal_id,
                "message": notification.message,
                "data": notification.data,
                "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            });

            if sender.send(Message::Text(json.to_string())).await.is_err() {
                break;
            }
        }
    });

    // Spawn task to receive messages (ping/pong)
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver_ws.next().await {
            if let Message::Close(_) = msg {
                break;
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    info!("WebSocket connection closed");
}

