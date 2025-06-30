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
