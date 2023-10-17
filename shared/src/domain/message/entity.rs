use chrono::{DateTime, Utc};

use crate::domain::{RoomId, UserId};

use super::{MessageContent, MessageId};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub user_id: UserId,
    pub room_id: RoomId,
    pub content: MessageContent,
    pub created_at: DateTime<Utc>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct NewMessage {
    pub user_id: UserId,
    pub room_id: RoomId,
    pub content: MessageContent,
}
