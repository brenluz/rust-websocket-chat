mod message;
mod db;
mod room;
mod ws;
use axum::{Router, routing::get};

#[tokio::main]
async fn main() {

    let room_manager = room::RoomManager::new();
    let app = Router::new()
    .route("/", get(|| async { "WebSocket Chat Server" }))
    .route("/ws/:room", get(ws::ws_handler))
    .with_state(room_manager);



    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app.into_make_service()).await.unwrap();
}
