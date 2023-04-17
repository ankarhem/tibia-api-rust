use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    // our router
    let app = Router::new().route("/", get(root));
    // .route("/foo", get(get_foo).post(post_foo))
    // .route("/foo/bar", get(foo_bar));

    let server =
        axum::Server::bind(&"0.0.0.0:7032".parse().unwrap()).serve(app.into_make_service());
    let addr = server.local_addr();

    println!("Listening on {addr}");

    server.await.unwrap();
}

async fn root() -> &'static str {
    "Hi from Axum!"
}
