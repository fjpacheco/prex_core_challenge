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
