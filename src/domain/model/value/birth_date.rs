use std::fmt::{Display, Formatter};

use chrono::NaiveDate;

use crate::domain::model::error::ClientError;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// A valid birth date.
pub struct BirthDate(NaiveDate);
impl BirthDate {
    pub fn new(name: &str) -> Result<Self, ClientError> {
        let name = name.trim();
        if name.is_empty() {
            Err(ClientError::FieldEmpty {
                field_name: "birth_date".to_string(),
            })
        } else {
            let birth_date = NaiveDate::parse_from_str(name, "%Y-%m-%d").map_err(|_| {
                ClientError::FieldInvalid {
                    field_name: "birth_date".to_string(),
                    value: name.to_string(),
                }
            })?;
            Ok(BirthDate(birth_date))
        }
    }
}

impl Display for BirthDate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.format("%Y-%m-%d").to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_01_given_a_valid_birth_date_when_creating_it_then_it_should_be_created() {
        let date_str = "1990-01-01";
        let birth_date = BirthDate::new(date_str).unwrap();
        assert_eq!(birth_date.to_string(), date_str);
    }

    #[test]
    fn test_02_given_an_empty_birth_date_when_creating_it_then_it_should_fail() {
        let date_str = "";
        let result = BirthDate::new(date_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_03_given_an_invalid_format_birth_date_when_creating_it_then_it_should_fail() {
        let date_str = "01-01-1990"; // formato incorrecto
        let result = BirthDate::new(date_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_04_given_a_non_date_string_when_creating_it_then_it_should_fail() {
        let date_str = "not-a-date";
        let result = BirthDate::new(date_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_05_given_a_birth_date_with_spaces_when_creating_it_then_it_should_be_trimmed_and_accepted()
     {
        let date_str = "   1990-01-01   ";
        let birth_date = BirthDate::new(date_str).unwrap();
        assert_eq!(birth_date.to_string(), "1990-01-01");
    }

    #[test]
    fn test_06_given_a_birth_date_out_of_range_when_creating_it_then_it_should_be_accepted() {
        let date_str = "1800-01-01";
        let birth_date = BirthDate::new(date_str);
        // chrono acepta fechas fuera de un rango "normal", así que solo validamos que no falle
        assert!(birth_date.is_ok());
    }
}
