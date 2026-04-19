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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::response::Response;

    #[tokio::test]
    async fn test_ws_handler() {
        let state = RoomManager::new();
        let response = ws_handler(
            WebSocketUpgrade::from_request(Request::builder().uri("/test").body(Body::empty()).unwrap()).await.unwrap(),
            State(state.clone()),
            Path("test".into())
        ).await.into_response();

        assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
    }

    #[tokio::test]
    async fn test_handle_socket() {
        let state = RoomManager::new();
        let (client, server) = tokio_tungstenite::tungstenite::protocol::WebSocket::pair().unwrap();
        let room = "test".to_string();

        let handle_task = tokio::spawn(handle_socket(server, state.clone(), room.clone()));

        let msg = Message::Chat {
            room: room.clone(),
            user: "user1".into(),
            body: "Hello, world!".into(),
            timestamp: 1234567890,
        };

        state.broadcast(&room, msg.clone());

        if let Ok(ws::Message::Text(text)) = client.recv() {
            let received_msg: Message = serde_json::from_str(&text).unwrap();
            assert!(matches!(received_msg, Message::Chat { .. }));
        } else {
            panic!("Did not receive expected message");
        }

        handle_task.abort();
    }

}