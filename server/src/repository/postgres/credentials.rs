use super::model::UserRow;
use crate::service;
use anyhow::Context;
use async_trait::async_trait;
use secrecy::{ExposeSecret, Secret};
use shared::domain::{User, UserCode, UserEmail, UserName};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct CredentialsAdapter {
    pool: PgPool,
}

impl CredentialsAdapter {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl service::CredentialsRepository for CredentialsAdapter {
    async fn get_credential(
        &self,
        email: &str,
    ) -> Result<Option<(Uuid, Secret<String>)>, anyhow::Error> {
        let row = sqlx::query!(
            r#"
                SELECT user_id, hashed_password
                FROM users
                WHERE email = $1;
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed to get credential from database.")?
        .map(|row| (row.user_id, Secret::new(row.hashed_password)));
        Ok(row)
    }

    async fn signup(
        &self,
        user_name: &UserName,
        user_email: &UserEmail,
        user_password: Secret<String>,
        user_code: &UserCode,
    ) -> Result<User, anyhow::Error> {
        let result = sqlx::query_as!(
            UserRow,
            r#"
                INSERT INTO users (username, email, code, hashed_password)
                VALUES ( $1, $2, $3, $4 )
                RETURNING user_id, username, email, code, created_at
            "#,
            user_name.as_ref(),
            user_email.as_ref(),
            user_code.as_ref(),
            user_password.expose_secret()
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to signup user in database.")?;

        result.try_into()
    }

    #[tracing::instrument(
        name = "Getting user details by email from the database",
        skip(self, email)
    )]
    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, anyhow::Error> {
        let result = sqlx::query_as!(
            UserRow,
            r#"
                SELECT user_id, username, email, code, created_at
                FROM users
                WHERE email = $1;
            "#,
            email,
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed get user by email from database.")?;
        result.map(UserRow::try_into).transpose()
    }
}
