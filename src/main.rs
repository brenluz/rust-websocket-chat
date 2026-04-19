mod message;
mod db;
use axum::{Router, routing::get};

use websocket_chat_server::room::RoomManager;
use websocket_chat_server::ws::ws_handler;

#[tokio::main]
async fn main() {

    let room_manager = RoomManager::new();
    let app = Router::new()
    .route("/", get(|| async { "WebSocket Chat Server" }))
    .route("/ws/{room}", get(ws_handler))
    .with_state(room_manager);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app.into_make_service()).await.unwrap();
}
