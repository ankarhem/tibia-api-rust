use axum::response::{Html, IntoResponse, Redirect};

pub async fn redirect_redocly() -> Redirect {
    Redirect::temporary("/api-docs")
}
