use axum::{Router, routing::get};
use tokio::net::TcpListener;
use websocket_chat_server::room::RoomManager;
use websocket_chat_server::ws::ws_handler;
use futures::{SinkExt, StreamExt};

async fn spawn_test_server() -> String {
    let room_manager = RoomManager::new();
    let app = Router::new()
        .route("/ws/{room}", get(ws_handler))
        .with_state(room_manager);

    // port 0 means OS picks a free port automatically
    let listener = TcpListener::bind("0.0.0.0:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        axum::serve(listener, app.into_make_service()).await.unwrap();
    });

    format!("ws://127.0.0.1:{}", port)
}

#[tokio::test]
async fn test_client_receives_message() {
    let server_url = spawn_test_server().await;

    let (mut client1, _)= tokio_tungstenite::connect_async(
        format!("{}/ws/general", server_url)).await.unwrap();
    let (mut client2, _)= tokio_tungstenite::connect_async(
        format!("{}/ws/general", server_url)).await.unwrap();

    let msg = websocket_chat_server::message::Message::Chat {
        room: "general".into(),
        user: "test_user".into(),
        body: "Hello, world!".into(),
        timestamp: 1234567890,
    };
    let json_msg = serde_json::to_string(&msg).unwrap();
    client1.send(tokio_tungstenite::tungstenite::Message::Text(json_msg.into())).await.unwrap();

    let received = client2.next().await.unwrap().unwrap();
    println!("Received: {:?}", received);
    assert!(received.to_text().unwrap().contains("Hello, world!"));

}