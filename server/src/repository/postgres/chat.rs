use anyhow::Context;
use async_trait::async_trait;
use shared::domain::{Message, MessageContent, Room, RoomCode, RoomId, RoomName, User, UserId};
use sqlx::PgPool;

use crate::service::ChatRepository;

use super::model::{MessageRow, RoomRow, UserRow};

#[derive(Clone)]
pub struct ChatAdapter {
    pool: PgPool,
}

impl ChatAdapter {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ChatRepository for ChatAdapter {
    async fn get_users(&self, room_id: &RoomId) -> Result<Vec<User>, anyhow::Error> {
        let rows = sqlx::query_as!(
            UserRow,
            r#"
                SELECT user_id, username, email, u.code, created_at 
                FROM members 
                JOIN users AS u using(user_id)
                WHERE room_id = $1
            "#,
            room_id.as_ref(),
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(User::try_from).collect()
    }

    async fn get_membership(
        &self,
        room_id: &RoomId,
        user_id: &UserId,
    ) -> Result<Option<String>, anyhow::Error> {
        let result = sqlx::query!(
            r#"
                SELECT code
                FROM members
                WHERE room_id = $1 AND user_id = $2;
            "#,
            room_id.as_ref(),
            user_id.as_ref(),
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed get user by email from database.")?
        .map(|row| row.code);

        Ok(result)
    }
}

impl ChatAdapter {
    pub async fn create_message(
        &self,
        sender_id: UserId,
        room_id: RoomId,
        content: MessageContent,
    ) -> Result<Message, anyhow::Error> {
        let result = sqlx::query_as!(
            MessageRow,
            r#"
                INSERT INTO messages (room_id, user_id, content)
                VALUES ($1, $2, $3)
                RETURNING message_id,room_id, content, user_id, created_at
            "#,
            room_id.as_ref(),
            sender_id.as_ref(),
            content.as_ref(),
        )
        .fetch_one(&self.pool)
        .await?;

        result.try_into()
    }

    pub async fn create_room(
        &self,
        user_id: UserId,
        room_name: RoomName,
        room_code: RoomCode,
    ) -> Result<Room, anyhow::Error> {
        let mut transaction = self.pool.begin().await?;

        let room = sqlx::query_as!(
            RoomRow,
            r#"
                INSERT INTO rooms (room_name, code)
                VALUES ($1, $2)
                RETURNING room_id, room_name, code
            "#,
            room_name.as_ref(),
            room_code.as_ref(),
        )
        .fetch_one(&mut *transaction)
        .await?;

        sqlx::query!(
            r#"
                INSERT INTO members (user_id, room_id)
                VALUES ($1, $2)
            "#,
            user_id.as_ref(),
            room.room_id,
        )
        .fetch_one(&mut *transaction)
        .await?;

        transaction.commit().await?;

        room.try_into()
    }

    pub async fn get_messages_by_room_id(
        &self,
        room_id: RoomId,
    ) -> Result<Vec<Message>, anyhow::Error> {
        let messages = sqlx::query_as!(
            MessageRow,
            r#"
                SELECT message_id, room_id, content, user_id, created_at
                FROM messages
                WHERE room_id = $1
            "#,
            room_id.as_ref(),
        )
        .fetch_all(&self.pool)
        .await?;

        messages.into_iter().map(Message::try_from).collect()
    }

    pub async fn get_user_rooms(&self, user_id: UserId) -> Result<Vec<Room>, anyhow::Error> {
        let messages = sqlx::query_as!(
            RoomRow,
            r#"
                SELECT room_id, room_name, code FROM rooms WHERE room_id IN (
                    SELECT room_id FROM members WHERE user_id = $1
                ) 
            "#,
            user_id.as_ref(),
        )
        .fetch_all(&self.pool)
        .await?;

        messages.into_iter().map(Room::try_from).collect()
    }
}
