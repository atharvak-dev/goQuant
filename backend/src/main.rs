use axum::{
    extract::{Path, State, WebSocketUpgrade},
    http::StatusCode,
    response::{Json, Response},
    routing::{get, post},
    Router,
};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::{info, Level};
use tracing_subscriber;

mod database;
mod error;
mod migration;
mod monitoring;
mod multisig;
mod proposal;
mod program_builder;
mod rollback;
mod security;
mod squads;
mod timelock;
mod websocket;

use error::UpgradeError;
use database::Database;
use proposal::ProposalManager;
use multisig::MultisigCoordinator;
use timelock::TimelockManager;
use program_builder::ProgramBuilder;
use migration::MigrationManager;
use rollback::RollbackHandler;
use monitoring::MonitoringService;
use security::SecurityAuditor;

#[derive(Clone)]
pub struct AppState {
    pub proposal_manager: Arc<ProposalManager>,
    pub multisig_coordinator: Arc<MultisigCoordinator>,
    pub timelock_manager: Arc<TimelockManager>,
    pub program_builder: Arc<ProgramBuilder>,
    pub migration_manager: Arc<MigrationManager>,
    pub rollback_handler: Arc<RollbackHandler>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting GoQuant Upgrade Service...");

    // Initialize database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/goquant_upgrades".to_string());
    let database = Arc::new(Database::new(&database_url).await?);

    // Initialize services
    let multisig_coordinator = Arc::new(MultisigCoordinator::new().await?);
    let timelock_manager = Arc::new(TimelockManager::new().await?);
    let program_builder = Arc::new(ProgramBuilder::new().await?);
    let migration_manager = Arc::new(MigrationManager::new().await?);
    let rollback_handler = Arc::new(RollbackHandler::new().await?);

    let proposal_manager = Arc::new(
        ProposalManager::new(
            multisig_coordinator.clone(),
            timelock_manager.clone(),
            program_builder.clone(),
        )
        .await?,
    );

    let app_state = AppState {
        proposal_manager,
        multisig_coordinator,
        timelock_manager,
        program_builder,
        migration_manager,
        rollback_handler,
    };

    // Initialize notification service
    let notification_service = websocket::NotificationService::new();
    let notification_sender = notification_service.get_sender();
    
    // Initialize monitoring service
    let monitoring_service = Arc::new(MonitoringService::new());
    
    // Initialize security auditor
    let security_auditor = Arc::new(SecurityAuditor);

    // Build router
    let app = Router::new()
        .route("/upgrade/propose", post(propose_upgrade))
        .route("/upgrade/:id/approve", post(approve_upgrade))
        .route("/upgrade/:id/execute", post(execute_upgrade))
        .route("/upgrade/:id/cancel", post(cancel_upgrade))
        .route("/upgrade/proposals", get(list_proposals))
        .route("/upgrade/:id/status", get(get_proposal_status))
        .route("/migration/start", post(start_migration))
        .route("/migration/progress", get(get_migration_progress))
        .route("/monitoring/metrics", get(get_metrics))
        .route("/monitoring/alerts", get(get_alerts))
        .route("/monitoring/health", get(get_health))
        .route("/ws", get(websocket_handler))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("Server listening on http://0.0.0.0:3000");

    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Deserialize)]
struct ProposeUpgradeRequest {
    new_program_buffer: String,
    description: String,
}

#[derive(Serialize)]
struct ProposeUpgradeResponse {
    proposal_id: String,
    timelock_until: i64,
}

async fn propose_upgrade(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(req): Json<ProposeUpgradeRequest>,
) -> Result<Json<ProposeUpgradeResponse>, UpgradeError> {
    let buffer_pubkey = req.new_program_buffer.parse()
        .map_err(|_| UpgradeError::InvalidPubkey)?;

    let proposal_id = state.proposal_manager
        .propose_upgrade(buffer_pubkey, req.description)
        .await?;

    let timelock_until = state.timelock_manager
        .get_timelock_end(&proposal_id)
        .await?;

    Ok(Json(ProposeUpgradeResponse {
        proposal_id,
        timelock_until,
    }))
}

async fn approve_upgrade(
    axum::extract::State(state): axum::extract::State<AppState>,
    Path(proposal_id): Path<String>,
) -> Result<Json<serde_json::Value>, UpgradeError> {
    state.multisig_coordinator
        .approve_proposal(&proposal_id)
        .await?;

    Ok(Json(serde_json::json!({
        "status": "approved",
        "proposal_id": proposal_id
    })))
}

async fn execute_upgrade(
    axum::extract::State(state): axum::extract::State<AppState>,
    Path(proposal_id): Path<String>,
) -> Result<Json<serde_json::Value>, UpgradeError> {
    state.proposal_manager
        .execute_upgrade(&proposal_id)
        .await?;

    Ok(Json(serde_json::json!({
        "status": "executed",
        "proposal_id": proposal_id
    })))
}

async fn cancel_upgrade(
    axum::extract::State(state): axum::extract::State<AppState>,
    Path(proposal_id): Path<String>,
) -> Result<Json<serde_json::Value>, UpgradeError> {
    state.proposal_manager
        .cancel_upgrade(&proposal_id)
        .await?;

    Ok(Json(serde_json::json!({
        "status": "cancelled",
        "proposal_id": proposal_id
    })))
}

async fn list_proposals(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<serde_json::Value>, UpgradeError> {
    let proposals = state.proposal_manager
        .list_proposals()
        .await?;

    Ok(Json(serde_json::json!(proposals)))
}

async fn get_proposal_status(
    axum::extract::State(state): axum::extract::State<AppState>,
    Path(proposal_id): Path<String>,
) -> Result<Json<serde_json::Value>, UpgradeError> {
    let status = state.proposal_manager
        .get_proposal_status(&proposal_id)
        .await?;

    Ok(Json(status))
}

async fn start_migration(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<serde_json::Value>, UpgradeError> {
    let migration_id = state.migration_manager
        .start_migration()
        .await?;

    Ok(Json(serde_json::json!({
        "migration_id": migration_id,
        "status": "started"
    })))
}

async fn get_migration_progress(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<serde_json::Value>, UpgradeError> {
    let progress = state.migration_manager
        .get_progress()
        .await?;

    Ok(Json(progress))
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
) -> Response {
    // In real implementation, get notification sender from state
    let notification_service = websocket::NotificationService::new();
    let notification_sender = notification_service.get_sender();
    let receiver = notification_sender.subscribe();

    ws.on_upgrade(|socket| websocket::handle_websocket(socket, receiver))
}

async fn get_metrics() -> Json<serde_json::Value> {
    let monitoring = MonitoringService::new();
    let dashboard = monitoring.get_dashboard_data().await;
    Json(dashboard)
}

async fn get_alerts() -> Json<serde_json::Value> {
    let monitoring = MonitoringService::new();
    let alerts = monitoring.get_alerts(50).await;
    Json(serde_json::json!(alerts))
}

async fn get_health() -> Json<serde_json::Value> {
    let monitoring = MonitoringService::new();
    let health = monitoring.check_health("system").await;
    Json(serde_json::json!({
        "status": format!("{:?}", health),
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
    }))
}

