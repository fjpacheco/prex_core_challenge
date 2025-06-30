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
