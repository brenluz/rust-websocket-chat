use crate::message::Message;
pub type Db = sqlx::SqlitePool;

const HISTORY_LIMIT: usize = 50;

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

pub async fn initialize(pool: &Db) {
    sqlx::query("CREATE TABLE IF NOT EXISTS messages (id INTEGER PRIMARY KEY, room TEXT, user TEXT, body TEXT, timestamp INTEGER)")
        .execute(pool).await.unwrap();
}

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
