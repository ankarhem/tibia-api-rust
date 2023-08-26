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
    Html(
        Redoc::new("/openapi.json")
            .custom_html(CUSTOM_HTML)
            .to_html(),
    )
}

const CUSTOM_HTML: &str = r#"
<!DOCTYPE html>
<html>
  <head>
    <title>Tibia API</title>
    <!-- needed for adaptive design -->
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <link rel="icon" href="/favicon.png">
    <link
      href="https://fonts.googleapis.com/css?family=Montserrat:300,400,700|Roboto:300,400,700"
      rel="stylesheet"
    />

    <!--
    Redoc doesn't change outer page styles
    -->
    <style>
      body {
        margin: 0;
        padding: 0;
      }
    </style>
  </head>
  <body>
    <div id="redoc-container"></div>
    <script src="https://cdn.redoc.ly/redoc/latest/bundles/redoc.standalone.js"></script>
    <script>
      Redoc.init(
        $spec,
        $config,
        document.getElementById("redoc-container")
      );
    </script>
  </body>
</html>
"#;
