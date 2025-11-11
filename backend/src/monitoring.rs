use crate::error::UpgradeError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    pub proposals_created: u64,
    pub proposals_executed: u64,
    pub proposals_cancelled: u64,
    pub migrations_completed: u64,
    pub rollbacks_initiated: u64,
    pub average_timelock_duration: f64,
    pub average_approval_time: f64,
}

pub struct MonitoringService {
    metrics: Arc<Mutex<Metrics>>,
    alerts: Arc<Mutex<Vec<Alert>>>,
    health_checks: Arc<Mutex<HashMap<String, HealthStatus>>>,
}

#[derive(Debug, Clone)]
pub struct Alert {
    pub level: AlertLevel,
    pub message: String,
    pub timestamp: i64,
    pub component: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

impl MonitoringService {
    pub fn new() -> Self {
        let service = Self {
            metrics: Arc::new(Mutex::new(Metrics {
                proposals_created: 0,
                proposals_executed: 0,
                proposals_cancelled: 0,
                migrations_completed: 0,
                rollbacks_initiated: 0,
                average_timelock_duration: 0.0,
                average_approval_time: 0.0,
            })),
            alerts: Arc::new(Mutex::new(Vec::new())),
            health_checks: Arc::new(Mutex::new(HashMap::new())),
        };

        // Start background monitoring tasks
        let metrics_clone = service.metrics.clone();
        let alerts_clone = service.alerts.clone();
        
        tokio::spawn(async move {
            Self::monitor_health(metrics_clone, alerts_clone).await;
        });

        service
    }

    pub async fn record_proposal_created(&self) {
        let mut metrics = self.metrics.lock().await;
        metrics.proposals_created += 1;
    }

    pub async fn record_proposal_executed(&self) {
        let mut metrics = self.metrics.lock().await;
        metrics.proposals_executed += 1;
    }

    pub async fn record_proposal_cancelled(&self) {
        let mut metrics = self.metrics.lock().await;
        metrics.proposals_cancelled += 1;
    }

    pub async fn record_migration_completed(&self) {
        let mut metrics = self.metrics.lock().await;
        metrics.migrations_completed += 1;
    }

    pub async fn record_rollback(&self) {
        let mut metrics = self.metrics.lock().await;
        metrics.rollbacks_initiated += 1;
        
        // Send critical alert
        self.send_alert(
            AlertLevel::Critical,
            "Rollback initiated".to_string(),
            "rollback_handler".to_string(),
        ).await;
    }

    pub async fn send_alert(&self, level: AlertLevel, message: String, component: String) {
        let alert = Alert {
            level: level.clone(),
            message: message.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            component,
        };

        let mut alerts = self.alerts.lock().await;
        alerts.push(alert.clone());

        // Log alert
        match level {
            AlertLevel::Info => tracing::info!("[{}] {}", alert.component, message),
            AlertLevel::Warning => tracing::warn!("[{}] {}", alert.component, message),
            AlertLevel::Critical => tracing::error!("[{}] {}", alert.component, message),
        }

        // In production, send to alerting service (PagerDuty, Slack, etc.)
        if level == AlertLevel::Critical {
            // Send critical alerts immediately
            Self::send_critical_alert(&alert).await;
        }
    }

    async fn send_critical_alert(alert: &Alert) {
        // In production, integrate with alerting service
        tracing::error!("CRITICAL ALERT: {} - {}", alert.component, alert.message);
    }

    pub async fn get_metrics(&self) -> Metrics {
        self.metrics.lock().await.clone()
    }

    pub async fn get_alerts(&self, limit: usize) -> Vec<Alert> {
        let alerts = self.alerts.lock().await;
        alerts.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    pub async fn check_health(&self, component: &str) -> HealthStatus {
        let health_checks = self.health_checks.lock().await;
        health_checks.get(component)
            .cloned()
            .unwrap_or(HealthStatus::Healthy)
    }

    pub async fn update_health(&self, component: String, status: HealthStatus) {
        let mut health_checks = self.health_checks.lock().await;
        health_checks.insert(component.clone(), status.clone());

        if status == HealthStatus::Unhealthy {
            self.send_alert(
                AlertLevel::Critical,
                format!("Component {} is unhealthy", component),
                component,
            ).await;
        }
    }

    async fn monitor_health(
        metrics: Arc<Mutex<Metrics>>,
        alerts: Arc<Mutex<Vec<Alert>>>,
    ) {
        let mut interval = interval(Duration::from_secs(60));

        loop {
            interval.tick().await;

            // Check various health indicators
            let metrics_guard = metrics.lock().await;
            
            // Example: Alert if too many proposals cancelled
            let cancellation_rate = if metrics_guard.proposals_created > 0 {
                metrics_guard.proposals_cancelled as f64 / metrics_guard.proposals_created as f64
            } else {
                0.0
            };

            if cancellation_rate > 0.5 {
                let mut alerts_guard = alerts.lock().await;
                alerts_guard.push(Alert {
                    level: AlertLevel::Warning,
                    message: format!("High cancellation rate: {:.2}%", cancellation_rate * 100.0),
                    timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
                    component: "monitoring".to_string(),
                });
            }
        }
    }

    pub async fn get_dashboard_data(&self) -> serde_json::Value {
        let metrics = self.get_metrics().await;
        let recent_alerts = self.get_alerts(10).await;
        let health_status = self.check_health("system").await;

        serde_json::json!({
            "metrics": metrics,
            "recent_alerts": recent_alerts,
            "health_status": format!("{:?}", health_status),
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        })
    }
}

