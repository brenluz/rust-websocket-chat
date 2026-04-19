//! Database functions for saving and retrieving chat messages.
//!
//! Uses SQLite via sqlx for async persistence. The database file is
//! created automatically on first run.

use crate::message::Message;

/// Type alias for the SQLite connection pool.
pub type Db = sqlx::SqlitePool;

const HISTORY_LIMIT: usize = 50;

/// Saves a chat message to the database.
/// 
/// Only `Chat` messages are persisted — `Join` and `System` variants
/// are silently ignored since they are transient events.
pub async fn save_message(message: &Message, pool: &Db) {
    match message {
        Message::Chat { room, user, body, timestamp } => {
            sqlx::query("INSERT INTO messages (room, user, body, timestamp) VALUES (?, ?, ?, ?)")
                .bind(room)
                .bind(user)
                .bind(body)
                .bind(timestamp)
                .execute(pool).await.unwrap();
        },
        _ => return, 
    }
}

/// Retrieves the last 50 chat messages for a given room, ordered by timestamp descending (most recent first).
/// Only messages of type `Chat` are returned, since `Join` and `System` messages are not stored in the database.
/// If there are no messages for the room, an empty vector is returned.
/// The function handles database errors gracefully by returning an empty vector if the query fails for any reason.
pub async fn get_history(room: &str, pool: &Db) -> Vec<Message> {
    match sqlx::query_as::<_, (String, String, String, i64)>("SELECT room, user, body, timestamp FROM messages WHERE room = ? ORDER BY timestamp DESC LIMIT ?")
        .bind(room)
        .bind(HISTORY_LIMIT as i64)
        .fetch_all(pool).await {
            Ok(rows) => rows.into_iter().map(|(room, user, body, timestamp)| {
                Message::Chat { room, user, body, timestamp }
            }).collect(),
            Err(_) => vec![],
        }
}

/// Initializes the database by creating the `messages` table if it doesn't already exist. This function should be called when the server starts to ensure the database is ready to use.
pub async fn initialize(pool: &Db) {
    sqlx::query("CREATE TABLE IF NOT EXISTS messages (id INTEGER PRIMARY KEY, room TEXT, user TEXT, body TEXT, timestamp INTEGER)")
        .execute(pool).await.unwrap();
}

/// Opens a connection to the SQLite database.
/// If the database file doesn't exist, it will be created automatically.
pub async fn open_db() -> Db {
    let pool = sqlx::SqlitePool::connect("sqlite://chat.db?mode=rwc").await.unwrap();
    initialize(&pool).await;
    pool
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_test_db() -> Db {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        initialize(&pool).await;
        pool
    }

    #[tokio::test]
    async fn test_db() {
        let pool = setup_test_db().await;
        let msg = Message::Chat {
            room: "test".into(),
            user: "user1".into(),
            body: "Hello, world!".into(),
            timestamp: 1234567890,
        };
        save_message(&msg, &pool).await;
        let history = get_history("test", &pool).await;
        assert_eq!(history.len(), 1);
        assert!(matches!(history[0], Message::Chat { .. }));
    }

    #[tokio::test]
    async fn test_history_limit() {
        let pool = setup_test_db().await;

        for i in 0..60 {
            let msg = Message::Chat {
                room: "test".into(),
                user: format!("user{}", i),
                body: format!("Message {}", i),
                timestamp: 1234567890 + i,
            };
            save_message(&msg, &pool).await;
        }

        let history = get_history("test", &pool).await;
        assert_eq!(history.len(), 50); // Should only return the last 50 messages
    }

    #[tokio::test]
    async fn test_different_rooms() {
        let pool = setup_test_db().await;

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

        save_message(&msg1, &pool).await;
        save_message(&msg2, &pool).await;

        let history1 = get_history("room1", &pool).await;
        let history2 = get_history("room2", &pool).await;
        assert_eq!(history1.len(), 1);
        assert_eq!(history2.len(), 1);
        assert!(matches!(history1[0], Message::Chat { .. }));   
        assert!(matches!(history2[0], Message::Chat { .. }));

    }


    #[tokio::test]
    async fn test_non_chat_message() {
        let pool = setup_test_db().await;
        let msg = Message::Join { room: "test".into(), user: "user1".into() };
        save_message(&msg, &pool).await;
        let history = get_history("test", &pool).await;
        assert!(history.is_empty()); 
        
    }
}
