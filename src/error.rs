use std::num::ParseIntError;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use thiserror::Error;
use utoipa::ToSchema;

pub type Result<T> = core::result::Result<T, ServerError>;

#[derive(Error, Clone, Debug)]
pub enum ServerError {
    // -- Request errors.
    #[error("Could not connect to the Tibia website.")]
    RequestFail,
    #[error("Could not decode the response body.")]
    RequestDecodeBodyFail,

    // -- Scraping errors.
    #[error("The requested resource was not found.")]
    ScrapeIs404Page, // tibia returns 200 for 404 pages, so we need to check for this.
    #[error("Unable to parse the response body.")]
    ScrapeUnexpectedPageContent,
    #[error(transparent)]
    ScrapeParseIntError(#[from] ParseIntError),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        // Create a placeholder Axum reponse.
        let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();

        // Insert the Error into the reponse.
        response.extensions_mut().insert(self);

        response
    }
}

impl ServerError {
    pub fn into_client_error(&self) -> ClientError {
        #[allow(unreachable_patterns)]
        match self {
            // -- Request.
            Self::RequestFail | Self::RequestDecodeBodyFail => ClientError {
                status: StatusCode::SERVICE_UNAVAILABLE,
                code: ClientErrorCode::SERVICE_ERROR,
                message: Some("Could not connect to the Tibia website."),
            },

            // -- Scrape.
            Self::ScrapeIs404Page => ClientError {
                status: StatusCode::NOT_FOUND,
                code: ClientErrorCode::RESOURCE_NOT_FOUND,
                message: Some("The requested resource was not found."),
            },
            Self::ScrapeUnexpectedPageContent => ClientError {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                code: ClientErrorCode::INTERNAL_ERROR,
                message: None,
            },

            // -- Fallback.
            _ => ClientError {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                code: ClientErrorCode::INTERNAL_ERROR,
                message: None,
            },
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[allow(non_camel_case_types)]
pub enum ClientErrorCode {
    SERVICE_ERROR,
    RESOURCE_NOT_FOUND,
    INTERNAL_ERROR,
}

#[derive(Debug, Serialize, ToSchema)]
#[allow(non_camel_case_types)]
pub struct ClientError<'a> {
    #[serde(skip)]
    status: StatusCode,
    code: ClientErrorCode,
    message: Option<&'a str>,
}

impl IntoResponse for ClientError<'_> {
    fn into_response(self) -> Response {
        let json = Json(&self);
        (self.status, json).into_response()
    }
}
