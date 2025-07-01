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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_01_given_a_valid_document_when_creating_it_then_it_should_be_created() {
        let doc = "1234567890";
        let document = Document::new(doc).unwrap();
        assert_eq!(document.to_string(), doc);
    }

    #[test]
    fn test_02_given_an_empty_document_when_creating_it_then_it_should_fail() {
        let doc = "";
        let result = Document::new(doc);
        assert!(result.is_err());
    }

    #[test]
    fn test_03_given_a_document_with_only_spaces_when_creating_it_then_it_should_fail() {
        let doc = "    ";
        let result = Document::new(doc);
        assert!(result.is_err());
    }

    #[test]
    fn test_04_given_a_document_exceeding_max_length_when_creating_it_then_it_should_fail() {
        let doc = "a".repeat(MAX_LENGTH_DOCUMENT + 1);
        let result = Document::new(&doc);
        assert!(result.is_err());
    }

    #[test]
    fn test_05_given_a_document_with_spaces_when_creating_it_then_it_should_be_trimmed_and_accepted()
     {
        let doc = "   1234567890   ";
        let document = Document::new(doc).unwrap();
        assert_eq!(document.to_string(), "1234567890");
    }
}
