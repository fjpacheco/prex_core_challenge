use std::fmt::{Display, Formatter};

use crate::domain::model::{error::ClientError, value::MAX_LENGTH_NAME};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// A valid client name.
pub struct ClientName(String);

impl ClientName {
    pub fn new(name: &str) -> Result<Self, ClientError> {
        let name = name.trim();
        if name.is_empty() {
            Err(ClientError::FieldEmpty {
                field_name: "name".to_string(),
            })
        } else if name.len() > MAX_LENGTH_NAME {
            Err(ClientError::FieldMaxLength {
                field_name: "name".to_string(),
                max_length: MAX_LENGTH_NAME,
            })
        } else {
            Ok(ClientName(name.to_string()))
        }
    }
}

impl Display for ClientName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_01_given_a_client_name_when_creating_it_then_it_should_be_created() {
        // GIVEN
        let name_expected = "John Doe";

        // SETUP
        let client_name = ClientName::new(name_expected).unwrap();

        // THEN
        assert_eq!(client_name.to_string(), name_expected);
    }

    #[test]
    fn test_02_given_an_empty_client_name_when_creating_it_then_it_should_fail() {
        // GIVEN
        let name = "";

        // WHEN
        let result = ClientName::new(name);

        // THEN
        assert!(result.is_err());
    }

    #[test]
    fn test_03_given_a_client_name_with_only_spaces_when_creating_it_then_it_should_fail() {
        // GIVEN
        let name = "     ";

        // WHEN
        let result = ClientName::new(name);

        // THEN
        assert!(result.is_err());
    }

    #[test]
    fn test_04_given_a_client_name_exceeding_max_length_when_creating_it_then_it_should_fail() {
        // GIVEN
        let name = "a".repeat(MAX_LENGTH_NAME + 1);

        // WHEN
        let result = ClientName::new(&name);

        // THEN
        assert!(result.is_err());
    }

    #[test]
    fn test_05_given_a_client_name_with_spaces_when_creating_it_then_it_should_be_trimmed_and_accepted()
     {
        let name = "   John Doe   ";
        let client_name = ClientName::new(name).unwrap();
        assert_eq!(client_name.to_string(), "John Doe");
    }

    #[test]
    fn test_06_given_a_client_name_with_special_characters_when_creating_it_then_it_should_be_accepted()
     {
        let name = "Jöhn D'oe!@#";
        let client_name = ClientName::new(name).unwrap();
        assert_eq!(client_name.to_string(), name);
    }
}
