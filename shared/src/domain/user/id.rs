use std::str::FromStr;

use crate::domain;

#[derive(
    Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash, Clone, Copy, Default,
)]
pub struct UserId(uuid::Uuid);

impl FromStr for UserId {
    type Err = domain::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(uuid::Uuid::from_str(s).map_err(|e| {
            domain::Error::ValidationError(e.to_string())
        })?))
    }
}

impl From<uuid::Uuid> for UserId {
    fn from(v: uuid::Uuid) -> Self {
        Self(v)
    }
}

impl AsRef<uuid::Uuid> for UserId {
    fn as_ref(&self) -> &uuid::Uuid {
        &self.0
    }
}
