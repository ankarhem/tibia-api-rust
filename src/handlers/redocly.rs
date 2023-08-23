use axum::{
    extract::State,
    response::{Html, IntoResponse, Redirect},
    Json,
};
use utoipa::openapi::OpenApi;
use utoipa_redoc::Redoc;

pub async fn redirect_redocly() -> Redirect {
    Redirect::temporary("/api-docs")
}

pub async fn serve_openapi(State(openapi_docs): State<OpenApi>) -> impl IntoResponse {
    Json(openapi_docs)
}

pub async fn serve_redocly() -> impl IntoResponse {
    Html(Redoc::new("/openapi.json").to_html())
}
