use crate::message::Message;
use rusqlite::{Connection};

fn save_message(message: &Message, conn: &Connection) {

    let query = "INSERT INTO messages (room, user, body, timestamp) VALUES ($1, $2, $3, $4)";
    let params = match message {
        Message::Chat { room, user, body, timestamp } => (room, user, body, timestamp),
        _ => return, // Only save chat messages
    };

    conn.execute(query, rusqlite::params![params.0, params.1, params.2, params.3]).unwrap();
}

fn get_history(room: &str, conn: &Connection) -> Vec<Message> {
    let query = "SELECT room, user, body, timestamp FROM messages WHERE room = $1 ORDER BY timestamp DESC LIMIT 50";

    let mut stmt = conn.prepare(query).unwrap();    
    let message_iter = stmt.query_map(rusqlite::params![room], |row| {
        Ok(Message::Chat {
            room: row.get(0)?,
            user: row.get(1)?,
            body: row.get(2)?,
            timestamp: row.get(3)?,
        })
    }).unwrap();
    message_iter.collect::<Result<Vec<Message>, rusqlite::Error>>().unwrap()
}

fn initialize(conn: &Connection) {
    conn.execute("CREATE TABLE IF NOT EXISTS messages (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        room TEXT NOT NULL,
        user TEXT NOT NULL,
        body TEXT NOT NULL,
        timestamp INTEGER NOT NULL
    )", []).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_db() {
        let conn = Connection::open_in_memory().unwrap();
        initialize(&conn);

        let msg = Message::Chat {
            room: "test".into(),
            user: "user1".into(),
            body: "Hello, world!".into(),
            timestamp: 1234567890,
        };

        save_message(&msg, &conn);
        let history = get_history("test", &conn);
        assert_eq!(history.len(), 1);
        assert!(matches!(history[0], Message::Chat { .. }));
    }

    #[test]
    fn test_history_limit() {
        let conn = Connection::open_in_memory().unwrap();
        initialize(&conn);

        for i in 0..60 {
            let msg = Message::Chat {
                room: "test".into(),
                user: format!("user{}", i),
                body: format!("Message {}", i),
                timestamp: 1234567890 + i,
            };
            save_message(&msg, &conn);
        }

        let history = get_history("test", &conn);
        assert_eq!(history.len(), 50); // Should only return the last 50 messages
    }

    #[test]
    fn test_different_rooms() {
        let conn = Connection::open_in_memory().unwrap();
        initialize(&conn);

        let msg1 = Message::Chat {
            room: "room1".into(),
            user: "user1".into(),
            body: "Hello, room1!".into(),
            timestamp: 1234567890,
        };
        let msg2 = Message::Chat {
            room: "room2".into(),
            user: "user2".into(),
            body: "Hello, room2!".into(),
            timestamp: 1234567891,
        };

        save_message(&msg1, &conn);
        save_message(&msg2, &conn);

        let history1 = get_history("room1", &conn);
        let history2 = get_history("room2", &conn);
        assert_eq!(history1.len(), 1);
        assert_eq!(history2.len(), 1);
        assert!(matches!(history1[0], Message::Chat { .. }));   
        assert!(matches!(history2[0], Message::Chat { .. }));

    }


    #[test]
    fn test_non_chat_message() {
        let conn = Connection::open_in_memory().unwrap();
        initialize(&conn);

        let msg = Message::Join { room: "test".into(), user: "user1".into() };
        save_message(&msg, &conn);
        let history = get_history("test", &conn);
        assert!(history.is_empty()); 
        
    }
}