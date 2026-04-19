//! WebSocket handler for the chat server.
//! 
//! This module defines the `ws_handler` function which is responsible for upgrading HTTP requests to WebSocket connections, managing the communication between clients and the server, and broadcasting messages to all clients subscribed to a room. It also handles sending the chat history to clients when they first connect.

use axum::extract::{Path, ws, State, WebSocketUpgrade};
use futures::{sink::SinkExt, stream::StreamExt};
use crate::db::{get_history, save_message};
use crate::message::Message;
use crate::state::AppState;

/// WebSocket handler that upgrades an HTTP request to a WebSocket connection and manages communication with the client.
/// 
/// Clients connect to a specific room by including the room name in the URL (e.g., `/ws/room1`). The handler sends the chat history for that room to the client upon connection and then listens for incoming messages from the client, broadcasting them to all other clients subscribed to the same room.
pub async fn ws_handler(
    upgrade: WebSocketUpgrade, 
    State(state): State<AppState>, 
    Path(room): Path<String>
) -> impl axum::response::IntoResponse {
    upgrade.on_upgrade(move |socket| handle_socket(socket, state, room))
}

/// Handles the WebSocket connection for a client. This function is responsible for sending the chat history to the client, listening for incoming messages, and broadcasting messages to other clients in the same room.
async fn handle_socket(socket: ws::WebSocket, state: AppState, room: String) {
    let (mut sender, mut receiver_ws) = socket.split();
    let mut rx = state.room_manager.subscribe(&room);
    
    // Send chat history to the client upon connection
    let history = get_history(&room, &state.db).await;
    for msg in history {
        let json = serde_json::to_string(&msg).unwrap();
        if sender.send(ws::Message::Text(json.into())).await.is_err() {
            return;
        }
    }

    // Spawn a task to handle incoming messages from the client and broadcast them to other clients in the same room
    let mut write_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let json = serde_json::to_string(&msg).unwrap();
            if sender.send(ws::Message::Text(json.into())).await.is_err() {
                break;
            }
        }
    });

    // Listen for incoming messages from the client and broadcast them to other clients in the same room
    let mut read_task = tokio::spawn(async move {
        while let Some(Ok(ws::Message::Text(text))) = receiver_ws.next().await {
            if let Ok(msg) = serde_json::from_str::<Message>(&text) {
                state.room_manager.broadcast(&room, msg.clone());
                save_message(&msg, &state.db).await;
            }
        }
    });

    // If either task finishes (e.g., client disconnects), we abort the other task to clean up resources
    tokio::select! {
        _ = &mut write_task => read_task.abort(),
        _ = &mut read_task => write_task.abort(),
    }
}