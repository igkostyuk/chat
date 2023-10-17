use std::str::FromStr;

use async_trait::async_trait;
use deadpool_redis::redis::AsyncCommands;
use deadpool_redis::Pool;
use shared::domain::UserId;
use uuid::Uuid;

use crate::service;

#[derive(Clone)]
pub struct TokenAdapter {
    pool: Pool,
}

impl TokenAdapter {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl service::TokenRepository for TokenAdapter {
    async fn create(&self, token_id: &Uuid, user_id: &UserId) -> anyhow::Result<()> {
        let mut conn = self.pool.get().await?;

        let key = format!("{}:{}", user_id.as_ref(), token_id);
        conn.set_ex(key, user_id.as_ref().to_string(), 3600).await?;
        Ok(())
    }
    async fn exist(&self, token_id: &Uuid, user_id: &UserId) -> anyhow::Result<Option<UserId>> {
        let mut conn = self.pool.get().await?;

        let key = format!("{}:{}", user_id.as_ref(), token_id);
        let res = conn.get::<String, String>(key).await?;
        Ok(Some(UserId::from_str(&res)?))
    }
    async fn delete(&self, token_id: &Uuid, user_id: &UserId) -> anyhow::Result<()> {
        let mut conn = self.pool.get().await?;

        let key = format!("{}:{}", user_id.as_ref(), token_id);
        conn.del(key).await?;
        Ok(())
    }
}
