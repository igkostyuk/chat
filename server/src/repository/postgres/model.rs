use chrono::{DateTime, Utc};
use shared::domain::{Message, Room, User};
use uuid::Uuid;

pub struct UserRow {
    pub user_id: uuid::Uuid,
    pub username: String,
    pub email: String,
    pub code: String,
    pub created_at: DateTime<Utc>,
}

impl TryFrom<UserRow> for User {
    type Error = anyhow::Error;

    fn try_from(u: UserRow) -> Result<Self, Self::Error> {
        Ok(Self {
            user_id: u.user_id.into(),
            name: u.username.try_into()?,
            email: u.email.try_into()?,
            code: u.code.try_into()?,
            created_at: u.created_at,
        })
    }
}

pub struct RoomRow {
    pub room_id: Uuid,
    pub room_name: String,
    pub code: String,
}

impl TryFrom<RoomRow> for Room {
    type Error = anyhow::Error;

    fn try_from(r: RoomRow) -> Result<Self, Self::Error> {
        let RoomRow {
            room_id,
            room_name,
            code,
        } = r;

        Ok(Self {
            id: room_id.into(),
            name: room_name.try_into()?,
            code: code.try_into()?,
        })
    }
}

pub struct MessageRow {
    pub message_id: Uuid,
    pub user_id: Uuid,
    pub room_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

impl TryFrom<MessageRow> for Message {
    type Error = anyhow::Error;

    fn try_from(m: MessageRow) -> Result<Self, Self::Error> {
        let MessageRow {
            message_id,
            user_id,
            room_id,
            content,
            created_at,
        } = m;

        Ok(Self {
            id: message_id.into(),
            user_id: user_id.into(),
            room_id: room_id.into(),
            content: content.try_into()?,
            created_at,
        })
    }
}
