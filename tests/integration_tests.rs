use axum::{Router, routing::get};
use tokio::net::TcpListener;
use websocket_chat_server::room::RoomManager;
use websocket_chat_server::ws::ws_handler;
use futures::{SinkExt, StreamExt};
use sqlx::SqlitePool;
use websocket_chat_server::state::AppState;

async fn spawn_test_server() -> String {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    websocket_chat_server::db::initialize(&pool).await;
    
    let state = AppState {
        room_manager: RoomManager::new(),
        db: pool,
    };

    let app = Router::new()
        .route("/ws/{room}", get(ws_handler))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
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

#[tokio::test]
async fn test_room_isolation() {
    let server_url = spawn_test_server().await;

    let (mut client1, _)= tokio_tungstenite::connect_async(
        format!("{}/ws/room1", server_url)).await.unwrap();
    let (mut client2, _)= tokio_tungstenite::connect_async(
        format!("{}/ws/room2", server_url)).await.unwrap();

    let msg = websocket_chat_server::message::Message::Chat {
        room: "room1".into(),
        user: "test_user".into(),
        body: "Hello, room1!".into(),
        timestamp: 1234567890,
    };
    let json_msg = serde_json::to_string(&msg).unwrap();
    client1.send(tokio_tungstenite::tungstenite::Message::Text(json_msg.into())).await.unwrap();

    let result = tokio::time::timeout(std::time::Duration::from_millis(100), client2.next()).await;

    assert!(result.is_err(), "Client in room2 should not receive messages from room1" );
}

#[tokio::test]
async fn test_history_on_connect() {
    let server_url = spawn_test_server().await;

    let (mut client1, _)= tokio_tungstenite::connect_async(
        format!("{}/ws/history_test", server_url)).await.unwrap();

    for i in 0..55 {
        let msg = websocket_chat_server::message::Message::Chat {
            room: "history_test".into(),
            user: format!("user{}", i),
            body: format!("Message {}", i),
            timestamp: 1234567890 + i,
        };
        let json_msg = serde_json::to_string(&msg).unwrap();
        client1.send(tokio_tungstenite::tungstenite::Message::Text(json_msg.into())).await.unwrap();
    }

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let (mut client2, _)= tokio_tungstenite::connect_async(
        format!("{}/ws/history_test", server_url)).await.unwrap();

    let mut received_messages = Vec::new();
    for _ in 0..50 {
        if let Some(Ok(tokio_tungstenite::tungstenite::Message::Text(text))) = client2.next().await {
            received_messages.push(text);
        }
    }

    assert_eq!(received_messages.len(), 50);
    assert!(received_messages[0].contains("Message 54"));
    assert!(received_messages[49].contains("Message 5"));
}

#[tokio::test]
async fn test_history_isolation() {
    let server_url = spawn_test_server().await;

    let (mut client1, _)= tokio_tungstenite::connect_async(
        format!("{}/ws/roomA", server_url)).await.unwrap();
    let (mut client2, _)= tokio_tungstenite::connect_async(
        format!("{}/ws/roomB", server_url)).await.unwrap();

    for i in 0..10 {
        let msg = websocket_chat_server::message::Message::Chat {
            room: "roomA".into(),
            user: format!("userA{}", i),
            body: format!("Message A{}", i),
            timestamp: 1234567890 + i,
        };
        let json_msg = serde_json::to_string(&msg).unwrap();
        client1.send(tokio_tungstenite::tungstenite::Message::Text(json_msg.into())).await.unwrap();
    }

    for i in 0..10 {
        let msg = websocket_chat_server::message::Message::Chat {
            room: "roomB".into(),
            user: format!("userB{}", i),
            body: format!("Message B{}", i),
            timestamp: 1234567890 + i,
        };
        let json_msg = serde_json::to_string(&msg).unwrap();
        client2.send(tokio_tungstenite::tungstenite::Message::Text(json_msg.into())).await.unwrap();
    }

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let (mut client3, _)= tokio_tungstenite::connect_async(
        format!("{}/ws/roomA", server_url)).await.unwrap();

    let mut received_messages_a = Vec::new();
    for _ in 0..10 {
        if let Some(Ok(tokio_tungstenite::tungstenite::Message::Text(text))) = client3.next().await {
            received_messages_a.push(text);
        }
    }

    assert_eq!(received_messages_a.len(), 10);
    assert!(received_messages_a[0].contains("Message A9"));
    assert!(received_messages_a[9].contains("Message A0"));

    let (mut client4, _)= tokio_tungstenite::connect_async(
        format!("{}/ws/roomB", server_url)).await.unwrap();

    let mut received_messages_b = Vec::new();
    for _ in 0..10 {
        if let Some(Ok(tokio_tungstenite::tungstenite::Message::Text(text))) = client4.next().await {
            received_messages_b.push(text);
        }
    }
    assert_eq!(received_messages_b.len(), 10);
    assert!(received_messages_b[0].contains("Message B9"));
    assert!(received_messages_b[9].contains("Message B0"));
}