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
}