use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use utoipa::ToSchema;

pub mod v1;

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiError {
    status: u16,
    message: String,
}

impl ApiError {
    pub fn new(status: u16, message: &str) -> Self {
        Self {
            status,
            message: message.to_string(),
        }
    }

    pub fn bad_request(message: &str) -> Self {
        Self::new(400, message)
    }

    pub fn not_found(message: &str) -> Self {
        Self::new(404, message)
    }

    pub fn internal_server_error(message: &str) -> Self {
        Self::new(500, message)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status_code = self.status.clone();
        let json = Json(self);
        (StatusCode::from_u16(status_code).unwrap(), json).into_response()
    }
}
