use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Message {
    Chat {
        room: String,
        user: String,
        body: String,
        timestamp: i64,
    },
    Join {
        room: String,
        user: String,
    },
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
}