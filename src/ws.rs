use axum::extract::{Path, ws, State, WebSocketUpgrade};
use crate::room::RoomManager;
use futures::{sink::SinkExt, stream::StreamExt};
use crate::message::Message;

pub async fn ws_handler(
    ws: WebSocketUpgrade, 
    State(state): State<RoomManager>, 
    Path(room): Path<String>
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, room))
}

async fn handle_socket(socket: ws::WebSocket, state: RoomManager, room: String) {
    let (mut sender, mut receiver_ws) = socket.split();
    let mut rx = state.subscribe(&room);

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
                state.broadcast(&room, msg);
            }
        }
    });

    tokio::select! {
        _ = &mut write_task => read_task.abort(),
        _ = &mut read_task => write_task.abort(),
    }
}