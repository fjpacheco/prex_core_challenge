use anyhow::Context;
use std::fmt::{Display, Formatter};
use uuid::Uuid;

use crate::domain::model::error::ClientError;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// A valid client id.
pub struct ClientId(Uuid);

impl Default for ClientId {
    fn default() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Display for ClientId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.to_string())
    }
}

impl TryFrom<String> for ClientId {
    type Error = ClientError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let id = Uuid::parse_str(&value).with_context(|| ClientError::FieldInvalid {
            field_name: "client_id".to_string(),
            value: value.to_string(),
        })?;
        Ok(ClientId(id))
    }
}
