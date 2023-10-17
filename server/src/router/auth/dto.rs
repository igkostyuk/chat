use chrono::{DateTime, Utc};
use secrecy::Secret;
use shared::domain::{NewUser, User};

use crate::service;

#[derive(serde::Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: Secret<String>,
}

impl From<LoginRequest> for service::Credentials {
    fn from(r: LoginRequest) -> Self {
        Self {
            email: r.email,
            password: r.password,
        }
    }
}

#[derive(serde::Serialize)]
pub struct TokensResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(serde::Deserialize)]
pub struct SignupRequest {
    pub name: String,
    pub email: String,
    pub password: Secret<String>,
    pub code: String,
}

impl TryFrom<SignupRequest> for NewUser {
    type Error = anyhow::Error;

    fn try_from(req: SignupRequest) -> Result<Self, Self::Error> {
        let SignupRequest {
            name,
            email,
            password,
            code,
        } = req;

        Ok(Self {
            name: name.parse()?,
            email: email.parse()?,
            password,
            code: code.parse()?,
        })
    }
}

#[derive(serde::Serialize)]
pub struct SignupResponse {
    pub user: UserResponse,
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(serde::Serialize)]
pub struct UserResponse {
    pub user_id: uuid::Uuid,
    pub name: String,
    pub email: String,
    pub code: String,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        let User {
            user_id,
            name,
            email,
            code,
            created_at,
        } = user;
        Self {
            user_id: *user_id.as_ref(),
            name: name.as_ref().to_owned(),
            email: email.as_ref().to_owned(),
            code: code.as_ref().to_owned(),
            created_at,
        }
    }
}
