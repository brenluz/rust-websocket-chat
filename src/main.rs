mod message;
mod db;

use axum::{Router, routing::get};

#[tokio::main]
async fn main() {
    let app: Router = Router::new()
    .route("/", get(|| async { "WebSocket Chat Server" }));

    let listener: tokio::net::TcpListener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app.into_make_service()).await.unwrap();
}
