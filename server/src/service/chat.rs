use async_trait::async_trait;

use crate::service::Error;
use shared::domain::{RoomId, User, UserId};

#[derive(Clone)]
pub struct ChatServiceImp<ChatRepo> {
    chat_repo: ChatRepo,
}

impl<ChatRepo> ChatServiceImp<ChatRepo>
where
    ChatRepo: ChatRepository,
{
    pub fn new(chat_repo: ChatRepo) -> Self {
        Self { chat_repo }
    }
}

#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait ChatRepository {
    async fn get_users(&self, room_id: &RoomId) -> Result<Vec<User>, anyhow::Error>;

    async fn get_membership(
        &self,
        room_id: &RoomId,
        user_id: &UserId,
    ) -> Result<Option<String>, anyhow::Error>;
}

#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait ChatService {
    async fn get_users(&self, room_id: &RoomId) -> Result<Vec<User>, Error>;
    async fn get_membership(
        &self,
        room_id: &RoomId,
        user_id: &UserId,
    ) -> Result<Option<String>, Error>;
}

#[async_trait]
impl<ChatRepo> ChatService for ChatServiceImp<ChatRepo>
where
    ChatRepo: ChatRepository + Send + Sync,
{
    async fn get_users(&self, chat_id: &RoomId) -> Result<Vec<User>, Error> {
        self.chat_repo
            .get_users(chat_id)
            .await
            .map_err(Error::UnexpectedError)
    }

    async fn get_membership(
        &self,
        room_id: &RoomId,
        user_id: &UserId,
    ) -> Result<Option<String>, Error> {
        self.chat_repo
            .get_membership(room_id, user_id)
            .await
            .map_err(Error::UnexpectedError)
    }
}
