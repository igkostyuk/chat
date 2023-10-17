use std::str::FromStr;

use crate::domain;

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
pub struct RoomId(uuid::Uuid);

impl FromStr for RoomId {
    type Err = domain::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(uuid::Uuid::from_str(s).map_err(|e| {
            domain::Error::ValidationError(e.to_string())
        })?))
    }
}

impl From<uuid::Uuid> for RoomId {
    fn from(v: uuid::Uuid) -> Self {
        Self(v)
    }
}

impl AsRef<uuid::Uuid> for RoomId {
    fn as_ref(&self) -> &uuid::Uuid {
        &self.0
    }
}
