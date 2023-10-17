use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{Message, MessageContent, Room, User, UserId};

#[derive(Serialize, Deserialize)]
pub enum ClientEvent {
    Join(JoinRequest),
    SendMessage(MessageContent),
}

#[derive(Serialize, Deserialize)]
pub enum ServerEvent {
    ErrMessage(ServerError),
    Join(JoinResponse),
    UserJoin(UserJoinResponse),
    ReceivedMessage(Message),
}

#[derive(Serialize, Deserialize)]
pub struct JoinRequest {
    pub join_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct JoinResponse {
    pub user_id: UserId,
    // pub room: Room,
    // pub users: Vec<User>,
    // pub messages: Vec<Message>,
}

#[derive(Serialize, Deserialize)]
pub struct UserJoinResponse {
    pub user_id: UserId,
    // pub room: Room,
    // pub users: Vec<User>,
    // pub messages: Vec<Message>,
}

#[derive(Serialize, Deserialize)]
pub struct ServerError {
    pub message: String,
}
