use std::fmt::{Display, Formatter};

use crate::domain::model::error::ClientError;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// A valid client id.
pub struct ClientId(usize);

impl ClientId {
    pub fn new(id: &str) -> Result<Self, ClientError> {
        let id_trimmed = id.trim();
        if id_trimmed.is_empty() {
            return Err(ClientError::FieldInvalid {
                field_name: "client_id".to_string(),
                value: id.to_string(),
            });
        }
        let id_parsed = id_trimmed.parse::<usize>();
        match id_parsed {
            Ok(id) => Ok(Self(id)),
            Err(_) => Err(ClientError::FieldInvalid {
                field_name: "client_id".to_string(),
                value: id.to_string(),
            }),
        }
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
        ClientId::new(&value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_01_given_default_client_id_then_it_should_be_valid_integer() {
        let client_id = ClientId::new("1").unwrap();
        assert_eq!(client_id.to_string(), "1");
    }

    #[test]
    fn test_02_given_valid_integer_string_when_try_from_then_it_should_create_client_id() {
        let client_id = ClientId::try_from("1".to_string());
        assert!(client_id.is_ok());
        assert_eq!(client_id.unwrap().to_string(), "1");
    }

    #[test]
    fn test_03_given_invalid_string_when_try_from_then_it_should_fail() {
        let invalid = "not-a-number".to_string();
        let client_id = ClientId::try_from(invalid);
        assert!(client_id.is_err());
    }

    #[test]
    fn test_04_given_valid_integer_string_when_try_from_then_it_should_create_client_id() {
        let client_id = ClientId::try_from("123".to_string());
        assert!(client_id.is_ok());
        assert_eq!(client_id.unwrap().to_string(), "123");
    }

    #[test]
    fn test_05_given_string_with_spaces_when_try_from_then_it_should_fail() {
        let client_id = ClientId::try_from(" 1 ".to_string());
        assert!(client_id.is_ok());
        assert_eq!(client_id.unwrap().to_string(), "1");
        let client_id = ClientId::try_from("1 ".to_string());
        assert!(client_id.is_ok());
        assert_eq!(client_id.unwrap().to_string(), "1");
        let client_id = ClientId::try_from(" 1".to_string());
        assert!(client_id.is_ok());
        assert_eq!(client_id.unwrap().to_string(), "1");
    }

    #[test]
    fn test_06_given_empty_string_when_try_from_then_it_should_fail() {
        let client_id = ClientId::try_from("".to_string());
        assert!(client_id.is_err());
        let client_id = ClientId::try_from("   ".to_string());
        assert!(client_id.is_err());
    }
}
