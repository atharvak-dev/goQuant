use thiserror::Error;

#[derive(Error, Debug)]
pub enum UpgradeError {
    #[error("Invalid public key")]
    InvalidPubkey,

    #[error("Proposal not found: {0}")]
    ProposalNotFound(String),

    #[error("Timelock still active: {remaining_seconds} seconds remaining")]
    TimelockActive { remaining_seconds: i64 },

    #[error("Insufficient approvals: {current}/{required}")]
    InsufficientApprovals { current: usize, required: usize },

    #[error("Not a multisig member")]
    NotMultisigMember,

    #[error("Proposal already executed")]
    AlreadyExecuted,

    #[error("Proposal already cancelled")]
    AlreadyCancelled,

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Solana client error: {0}")]
    SolanaError(String),

    #[error("Multisig error: {0}")]
    MultisigError(String),

    #[error("Migration error: {0}")]
    MigrationError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl axum::response::IntoResponse for UpgradeError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            UpgradeError::ProposalNotFound(_) => (axum::http::StatusCode::NOT_FOUND, self.to_string()),
            UpgradeError::TimelockActive { .. } => (axum::http::StatusCode::BAD_REQUEST, self.to_string()),
            UpgradeError::InsufficientApprovals { .. } => (axum::http::StatusCode::BAD_REQUEST, self.to_string()),
            UpgradeError::NotMultisigMember => (axum::http::StatusCode::FORBIDDEN, self.to_string()),
            UpgradeError::AlreadyExecuted => (axum::http::StatusCode::BAD_REQUEST, self.to_string()),
            UpgradeError::AlreadyCancelled => (axum::http::StatusCode::BAD_REQUEST, self.to_string()),
            _ => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = serde_json::json!({
            "error": error_message
        });

        (status, axum::Json(body)).into_response()
    }
}

