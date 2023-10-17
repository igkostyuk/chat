use super::{UserCode, UserEmail, UserId, UserName};
use chrono::{DateTime, Utc};
use secrecy::Secret;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct User {
    pub user_id: UserId,
    pub name: UserName,
    pub email: UserEmail,
    pub code: UserCode,
    pub created_at: DateTime<Utc>,
}

#[derive(serde::Deserialize)]
pub struct NewUser {
    pub name: UserName,
    pub email: UserEmail,
    pub password: Secret<String>,
    pub code: UserCode,
}
