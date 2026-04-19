use axum::extract::{Path, ws, State, WebSocketUpgrade};
use futures::{sink::SinkExt, stream::StreamExt};
use crate::db::{get_history, save_message};
use crate::message::Message;
use crate::state::AppState;

pub async fn ws_handler(
    upgrade: WebSocketUpgrade, 
    State(state): State<AppState>, 
    Path(room): Path<String>
) -> impl axum::response::IntoResponse {
    upgrade.on_upgrade(move |socket| handle_socket(socket, state, room))
}

async fn handle_socket(socket: ws::WebSocket, state: AppState, room: String) {
    let (mut sender, mut receiver_ws) = socket.split();
    let mut rx = state.room_manager.subscribe(&room);
    
    let history = get_history(&room, &state.db).await;
    for msg in history {
        let json = serde_json::to_string(&msg).unwrap();
        if sender.send(ws::Message::Text(json.into())).await.is_err() {
            return;
        }
    }

    let mut write_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let json = serde_json::to_string(&msg).unwrap();
            if sender.send(ws::Message::Text(json.into())).await.is_err() {
                break;
            }
        }
    });

    let mut read_task = tokio::spawn(async move {
        while let Some(Ok(ws::Message::Text(text))) = receiver_ws.next().await {
            if let Ok(msg) = serde_json::from_str::<Message>(&text) {
                state.room_manager.broadcast(&room, msg.clone());
                save_message(&msg, &state.db).await;
            }
        }
    });

    tokio::select! {
        _ = &mut write_task => read_task.abort(),
        _ = &mut read_task => write_task.abort(),
    }
}