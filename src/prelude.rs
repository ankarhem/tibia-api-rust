use axum::{response::IntoResponse, Json};
use reqwest::StatusCode;
use serde_json::json;

pub const COMMUNITY_URL: &str = "https://www.tibia.com/community/";

pub fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}

#[derive(thiserror::Error)]
pub enum PublicError {
    #[error(transparent)]
    FetchError(#[from] reqwest::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for PublicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl IntoResponse for PublicError {
    fn into_response(self) -> axum::response::Response {
        match self {
            PublicError::FetchError(e) => match e.status() {
                Some(StatusCode::NOT_FOUND) => StatusCode::NOT_FOUND.into_response(),
                Some(status) => {
                    let body = json!({
                        "message": "The tibia website failed to process the underlying request",
                        "details": {
                            "status": status.as_u16(),
                        }
                    });
                    (StatusCode::SERVICE_UNAVAILABLE, Json(body)).into_response()
                }
                _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            },
            PublicError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

pub trait Sanitizable {
    fn sanitize(self) -> Self;
}

impl Sanitizable for String {
    fn sanitize(self) -> Self {
        self.trim()
            .replace("\\n", "")
            .replace("\\\"", "'")
            .replace("\\u00A0", " ")
            .replace("\\u0026", "&")
            .replace("\\u0026#39;", "'")
            .replace("&nbsp;", " ")
            .replace("&amp;", "&")
    }
}
