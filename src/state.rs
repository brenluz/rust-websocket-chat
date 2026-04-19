use crate::room::RoomManager;
use crate::db::Db;


#[derive(Clone)]
pub struct AppState {
    pub room_manager: RoomManager,
    pub db: Db,
}