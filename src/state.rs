//! Application state shared across handlers

use crate::room::RoomManager;
use crate::db::Db;

/// Application state that is shared across all handlers. This includes the `RoomManager` for managing chat rooms and a database connection pool for storing chat history.
#[derive(Clone)]
pub struct AppState {
    pub room_manager: RoomManager,
    pub db: Db,
}