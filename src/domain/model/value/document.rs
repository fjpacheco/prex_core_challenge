use std::fmt::{Display, Formatter};

use crate::domain::model::{error::ClientError, value::MAX_LENGTH_DOCUMENT};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// A valid document.
pub struct Document(String);

impl Document {
    pub fn new(name: &str) -> Result<Self, ClientError> {
        let name = name.trim();
        if name.is_empty() {
            Err(ClientError::FieldEmpty {
                field_name: "document".to_string(),
            })
        } else if name.len() > MAX_LENGTH_DOCUMENT {
            Err(ClientError::FieldMaxLength {
                field_name: "document".to_string(),
                max_length: MAX_LENGTH_DOCUMENT,
            })
        } else {
            Ok(Document(name.to_string()))
        }
    }
}

impl Display for Document {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
