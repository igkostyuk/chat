use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::service;

#[derive(serde::Serialize)]
pub struct ErrorResponse {
    pub message: String,
}

impl IntoResponse for service::Error {
    fn into_response(self) -> Response {
        let status: StatusCode = match self {
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::ValidationError(_) => StatusCode::BAD_REQUEST,
            Self::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InvalidCredentials(_) => StatusCode::FORBIDDEN,
            Self::ConflictError(_) => StatusCode::CONFLICT,
        };
        let resp = ErrorResponse {
            message: self.to_string(),
        };
        (status, Json(resp)).into_response()
    }
}
