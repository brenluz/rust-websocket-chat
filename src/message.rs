//! Defines the `Message` enum which represents different types of messages that can be sent over the WebSocket connection.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]

/// Represents a message that can be sent over the WebSocket connection.
/// 
/// All variants are serialized to JSON with a `type` field for easy
/// consumption in JavaScript clients.
pub enum Message {
    /// A chat message sent by a user in a room.
    Chat {
        /// The name of the room this message belongs to.
        room: String,
        /// The username of the sender.
        user: String,
        /// The message content.
        body: String,
        /// Unix timestamp of when the message was sent.
        timestamp: i64,
    },
    /// Sent when a user joins a room.
    Join {
        room: String,
        user: String,
    },
    /// A server-generated system message.
    System {
        body: String,
        timestamp: i64,
    },

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] 
    fn test_chat_message() {
        let msg = Message::Chat {
            room: "test".into(),
            user: "user1".into(),
            body: "Hello, world!".into(),
            timestamp: 1234567890,
        };

        let json = serde_json::to_string(&msg).unwrap();
        println!("{}", json);

        let deserialized: Message = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, Message::Chat { .. }));

    }

    #[test]
    fn test_join_message() {
        let msg = Message::Join {
            room: "test".into(),
            user: "user1".into(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        println!("{}", json);
        let deserialized: Message = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, Message::Join { .. }));
    }

    #[test]
    fn test_system_message() {
        let msg = Message::System {
            body: "Server will restart in 5 minutes".into(),
            timestamp: 1234567890,
        };

        let json = serde_json::to_string(&msg).unwrap();
        println!("{}", json);
        let deserialized: Message = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, Message::System { .. }));
    }
}