use axum::{response::IntoResponse, Json};
use reqwest::StatusCode;
use utoipa::{schema, ToSchema};

pub use crate::clients::{Client, TibiaClient};

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
pub enum ServerError {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Middleware(#[from] reqwest_middleware::Error),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

#[derive(serde::Serialize, serde::Deserialize, ToSchema)]
pub struct PublicErrorBody {
    #[schema(example = "The tibia website failed to process the underlying request")]
    message: String,
}

impl std::fmt::Debug for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ServerError::Middleware(reqwest_middleware::Error::Reqwest(e))
            | ServerError::Reqwest(e) => match e.status() {
                Some(StatusCode::NOT_FOUND) => StatusCode::NOT_FOUND.into_response(),
                Some(_) => {
                    let body = PublicErrorBody {
                        message: "The tibia website failed to process the underlying request"
                            .into(),
                    };
                    (StatusCode::SERVICE_UNAVAILABLE, Json(body)).into_response()
                }
                _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            },
            ServerError::Middleware(_) | ServerError::Unexpected(_) => {
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
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
            .replace(' ', " ")
    }
}
