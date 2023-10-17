use super::{RoomCode, RoomId, RoomName};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Room {
    pub id: RoomId,
    pub name: RoomName,
    pub code: RoomCode,
}
