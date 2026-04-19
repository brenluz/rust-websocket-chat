use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::message::Message;

const CHANNEL_CAPACITY: usize = 100;

#[derive(Clone)]
pub struct RoomManager {
    inner: Arc<Mutex<HashMap<String, broadcast::Sender<Message>>>>
}

impl RoomManager {

    pub fn new() -> Self {
        RoomManager {
            inner: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    pub fn subscribe(&self, room: &str) -> broadcast::Receiver<Message> {
        let mut rooms = self.inner.lock().unwrap();
        if let Some(sender) = rooms.get(room) {
            sender.subscribe()
        } else {
            let (sender, receiver) = broadcast::channel(CHANNEL_CAPACITY);
            rooms.insert(room.to_string(), sender);
            receiver
        }
    }

    pub fn broadcast(&self, room: &str, message: Message) {
        let rooms = self.inner.lock().unwrap();
        if let Some(sender) = rooms.get(room) {
            let _ = sender.send(message);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_room_manager() {
        let manager = RoomManager::new();
        let mut receiver1 = manager.subscribe("test");
        let mut receiver2 = manager.subscribe("test");

        manager.broadcast("test", Message::Chat {
            room: "test".into(),
            user: "user1".into(),
            body: "Hello, world!".into(),
            timestamp: 1234567890,
        });

        assert!(matches!(receiver1.try_recv(), Ok(Message::Chat { .. })));
        assert!(matches!(receiver2.try_recv(), Ok(Message::Chat { .. })));
    }

    #[test]
    fn test_room_isolation() {
        let manager = RoomManager::new();
        let mut receiver1 = manager.subscribe("room1");
        let mut receiver2 = manager.subscribe("room2");

        manager.broadcast("room1", Message::Chat {
            room: "room1".into(),
            user: "user1".into(),
            body: "Hello, room1!".into(),
            timestamp: 1234567890,
        });

        assert!(matches!(receiver1.try_recv(), Ok(Message::Chat { .. })));
        assert!(matches!(receiver2.try_recv(), Err(broadcast::error::TryRecvError::Empty)));
    }

    #[test]
    fn test_multiple_broadcasts() {
        let manager = RoomManager::new();
        let mut receiver = manager.subscribe("test");

        for i in 0..5 {
            manager.broadcast("test", Message::Chat {
                room: "test".into(),
                user: format!("user{}", i),
                body: format!("Message {}", i),
                timestamp: 1234567890 + i,
            });
        }

        for i in 0..5 {
            if let Ok(Message::Chat { user, body, .. }) = receiver.try_recv() {
                assert_eq!(user, format!("user{}", i));
                assert_eq!(body, format!("Message {}", i));
            } else {
                panic!("Expected a chat message");
            }
        }
    }
}