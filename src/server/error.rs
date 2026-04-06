use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("ZeroTier local service error: {0}")]
    ZtLocal(String),
    #[error("ZeroTier Central API error: {0}")]
    ZtCentral(String),
    #[error("Config error: {0}")]
    Config(String),
    #[error("Exit node error: {0}")]
    ExitNode(String),
    #[error("Not found")]
    NotFound,
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, msg) = match &self {
            ApiError::ZtLocal(m) => (StatusCode::BAD_GATEWAY, "ERR_ZT_LOCAL", m.as_str()),
            ApiError::ZtCentral(m) => (StatusCode::BAD_GATEWAY, "ERR_ZT_CENTRAL", m.as_str()),
            ApiError::Config(m) => (StatusCode::INTERNAL_SERVER_ERROR, "ERR_CONFIG", m.as_str()),
            ApiError::ExitNode(m) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "ERR_EXITNODE",
                m.as_str(),
            ),
            ApiError::NotFound => (StatusCode::NOT_FOUND, "ERR_NOT_FOUND", "Not found"),
            ApiError::InvalidInput(m) => {
                (StatusCode::UNPROCESSABLE_ENTITY, "ERR_INVALID", m.as_str())
            }
            ApiError::Internal(m) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "ERR_INTERNAL",
                m.as_str(),
            ),
        };
        let body = Json(json!({ "error": msg, "code": code }));
        (status, body).into_response()
    }
}

// Удобные конверсии
impl From<crate::config::ConfigError> for ApiError {
    fn from(e: crate::config::ConfigError) -> Self {
        ApiError::Config(e.to_string())
    }
}
