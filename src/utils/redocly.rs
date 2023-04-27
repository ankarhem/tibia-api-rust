use axum::response::{Html, IntoResponse, Redirect};

pub async fn redirect_redocly() -> Redirect {
    Redirect::temporary("/api-docs")
}

pub async fn redocly_index() -> impl IntoResponse {
    let html = REDOCLY_HTML
        .replace("{title}", "Tibia API")
        .replace("{spec_url}", "/api-docs/openapi.json");
    Html::from(html)
}

const REDOCLY_HTML: &'static str = r#"
<!DOCTYPE html>
<html>
  <head>
    <title>{title}</title>
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
    <!--
    Redoc element with link to your OpenAPI definition
    -->
    <redoc spec-url="{spec_url}"></redoc>
    <!--
    Link to Redoc JavaScript on CDN for rendering standalone element
    -->
    <script src="https://cdn.redoc.ly/redoc/latest/bundles/redoc.standalone.js"></script>
  </body>
</html>
"#;
