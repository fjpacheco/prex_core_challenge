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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_01_given_default_client_id_then_it_should_be_valid_uuid() {
        let client_id = ClientId::default();
        let uuid = Uuid::parse_str(&client_id.to_string());
        assert!(uuid.is_ok());
    }

    #[test]
    fn test_02_given_valid_uuid_string_when_try_from_then_it_should_create_client_id() {
        let uuid = Uuid::new_v4();
        let uuid_str = uuid.to_string();
        let client_id = ClientId::try_from(uuid_str.clone());
        assert!(client_id.is_ok());
        assert_eq!(client_id.unwrap().to_string(), uuid_str);
    }

    #[test]
    fn test_03_given_invalid_uuid_string_when_try_from_then_it_should_fail() {
        let invalid_uuid = "not-a-uuid".to_string();
        let client_id = ClientId::try_from(invalid_uuid);
        assert!(client_id.is_err());
    }

    #[test]
    fn test_04_given_valid_uuid_string_uppercase_when_try_from_then_it_should_create_client_id() {
        let uuid = Uuid::new_v4();
        let uuid_str = uuid.to_string().to_uppercase();
        let client_id = ClientId::try_from(uuid_str.clone());
        assert!(client_id.is_ok());
        assert_eq!(client_id.unwrap().to_string().to_uppercase(), uuid_str);
    }

    #[test]
    fn test_05_given_uuid_string_with_spaces_when_try_from_then_it_should_fail() {
        let uuid = format!(" {} ", Uuid::new_v4());
        let client_id = ClientId::try_from(uuid);
        assert!(client_id.is_err());
    }
}
