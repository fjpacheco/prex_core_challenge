use std::fmt::{Display, Formatter};

use crate::domain::model::{error::ClientError, value::MAX_LENGTH_COUNTRY};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// A valid country.
pub struct Country(String);

impl Country {
    pub fn new(name: &str) -> Result<Self, ClientError> {
        let name = name.trim();
        if name.is_empty() {
            Err(ClientError::FieldEmpty {
                field_name: "country".to_string(),
            })
        } else if name.len() > MAX_LENGTH_COUNTRY {
            Err(ClientError::FieldMaxLength {
                field_name: "country".to_string(),
                max_length: MAX_LENGTH_COUNTRY,
            })
        } else {
            Ok(Country(name.to_string()))
        }
    }
}

impl Display for Country {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_01_given_a_valid_country_when_creating_it_then_it_should_be_created() {
        let country_name = "Argentina";
        let country = Country::new(country_name).unwrap();
        assert_eq!(country.to_string(), country_name);
    }

    #[test]
    fn test_02_given_an_empty_country_when_creating_it_then_it_should_fail() {
        let country_name = "";
        let result = Country::new(country_name);
        assert!(result.is_err());
    }

    #[test]
    fn test_03_given_a_country_with_only_spaces_when_creating_it_then_it_should_fail() {
        let country_name = "    ";
        let result = Country::new(country_name);
        assert!(result.is_err());
    }

    #[test]
    fn test_04_given_a_country_exceeding_max_length_when_creating_it_then_it_should_fail() {
        let country_name = "a".repeat(MAX_LENGTH_COUNTRY + 1);
        let result = Country::new(&country_name);
        assert!(result.is_err());
    }

    #[test]
    fn test_05_given_a_country_with_spaces_when_creating_it_then_it_should_be_trimmed_and_accepted()
    {
        let country_name = "   Argentina   ";
        let country = Country::new(country_name).unwrap();
        assert_eq!(country.to_string(), "Argentina");
    }

    #[test]
    fn test_06_given_a_country_with_special_characters_when_creating_it_then_it_should_be_accepted()
    {
        let country_name = "Côte d'Ivoire!@#";
        let country = Country::new(country_name).unwrap();
        assert_eq!(country.to_string(), country_name);
    }
}
