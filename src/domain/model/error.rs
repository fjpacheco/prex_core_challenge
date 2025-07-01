use thiserror::Error;

use crate::domain::model::value::{client_id::ClientId, document::Document};

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("client with document {document} already exists")]
    Duplicate { document: String },

    #[error("client not found by id {id_document}")]
    NotFoundById { id_document: ClientId },

    #[error("client not found by document {document}")]
    NotFoundByDocument { document: Document },

    #[error("client {field_name} cannot be empty")]
    FieldEmpty { field_name: String },

    #[error("client {field_name} is invalid: {value}")]
    FieldInvalid { field_name: String, value: String },

    #[error("client {field_name} is too long. Max length is {max_length}")]
    FieldMaxLength {
        field_name: String,
        max_length: usize,
    },

    #[error("client amount cannot be negative")]
    NegativeAmount,

    #[error("client amount cannot be positive")]
    PositiveAmount,

    #[error("client amount cannot be zero")]
    ZeroAmount,

    #[error("balances are empty")]
    BalancesEmpty,

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

impl PartialEq for ClientError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ClientError::Duplicate { document: d1 }, ClientError::Duplicate { document: d2 }) => {
                d1 == d2
            }
            (
                ClientError::NotFoundById { id_document: id1 },
                ClientError::NotFoundById { id_document: id2 },
            ) => id1 == id2,
            (
                ClientError::NotFoundByDocument { document: d1 },
                ClientError::NotFoundByDocument { document: d2 },
            ) => d1 == d2,
            (
                ClientError::FieldEmpty { field_name: f1 },
                ClientError::FieldEmpty { field_name: f2 },
            ) => f1 == f2,
            (
                ClientError::FieldInvalid {
                    field_name: f1,
                    value: v1,
                },
                ClientError::FieldInvalid {
                    field_name: f2,
                    value: v2,
                },
            ) => f1 == f2 && v1 == v2,
            (
                ClientError::FieldMaxLength {
                    field_name: f1,
                    max_length: m1,
                },
                ClientError::FieldMaxLength {
                    field_name: f2,
                    max_length: m2,
                },
            ) => f1 == f2 && m1 == m2,
            (ClientError::NegativeAmount, ClientError::NegativeAmount) => true,
            (ClientError::PositiveAmount, ClientError::PositiveAmount) => true,
            (ClientError::ZeroAmount, ClientError::ZeroAmount) => true,
            (ClientError::BalancesEmpty, ClientError::BalancesEmpty) => true,
            (ClientError::Unknown(_), ClientError::Unknown(_)) => true,
            _ => false,
        }
    }
}

impl ClientError {
    /// Code error personalized for the client domain!
    pub fn code(&self) -> String {
        match self {
            ClientError::Duplicate { .. } => "CLIENT_DUPLICATE".to_string(),
            ClientError::NotFoundById { .. } => "CLIENT_NOT_FOUND_BY_ID".to_string(),
            ClientError::NotFoundByDocument { .. } => "CLIENT_NOT_FOUND_BY_DOCUMENT".to_string(),
            ClientError::FieldEmpty { field_name } => {
                format!("CLIENT_{}_EMPTY", field_name.to_uppercase())
            }
            ClientError::FieldInvalid {
                field_name,
                value: _,
            } => format!("CLIENT_{}_INVALID", field_name.to_uppercase()),
            ClientError::FieldMaxLength {
                field_name,
                max_length: _,
            } => format!("CLIENT_{}_MAX_LENGTH", field_name.to_uppercase()),
            ClientError::NegativeAmount => "CLIENT_NEGATIVE_BALANCE".to_string(),
            ClientError::PositiveAmount => "CLIENT_POSITIVE_BALANCE".to_string(),
            ClientError::ZeroAmount => "CLIENT_ZERO_BALANCE".to_string(),
            ClientError::BalancesEmpty => "CLIENT_BALANCES_EMPTY".to_string(),
            ClientError::Unknown(_) => "CLIENT_UNKNOWN_ERROR".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::value::{client_id::ClientId, document::Document};
    use anyhow::anyhow;

    #[test]
    fn test_01_given_two_duplicate_errors_with_same_document_when_comparing_then_they_should_be_equal() {
        // GIVEN
        let doc = "123".to_string();
        let err1 = ClientError::Duplicate { document: doc.clone() };
        let err2 = ClientError::Duplicate { document: doc.clone() };
        // THEN
        assert_eq!(err1, err2);
    }

    #[test]
    fn test_02_given_two_duplicate_errors_with_different_document_when_comparing_then_they_should_not_be_equal() {
        // GIVEN
        let err1 = ClientError::Duplicate { document: "123".to_string() };
        let err2 = ClientError::Duplicate { document: "456".to_string() };
        // THEN
        assert_ne!(err1, err2);
    }

    #[test]
    fn test_03_given_two_not_found_by_id_errors_with_same_id_when_comparing_then_they_should_be_equal() {
        // GIVEN
        let id = ClientId::default();
        let err1 = ClientError::NotFoundById { id_document: id.clone() };
        let err2 = ClientError::NotFoundById { id_document: id.clone() };
        // THEN
        assert_eq!(err1, err2);
    }

    #[test]
    fn test_04_given_two_not_found_by_id_errors_with_different_id_when_comparing_then_they_should_not_be_equal() {
        // GIVEN
        let err1 = ClientError::NotFoundById { id_document: ClientId::default() };
        let err2 = ClientError::NotFoundById { id_document: ClientId::default() };
        // THEN
        assert_ne!(err1, err2);
    }

    #[test]
    fn test_05_given_two_field_empty_errors_with_same_field_when_comparing_then_they_should_be_equal() {
        // GIVEN
        let err1 = ClientError::FieldEmpty { field_name: "foo".to_string() };
        let err2 = ClientError::FieldEmpty { field_name: "foo".to_string() };
        // THEN
        assert_eq!(err1, err2);
    }

    #[test]
    fn test_06_given_two_field_empty_errors_with_different_field_when_comparing_then_they_should_not_be_equal() {
        // GIVEN
        let err1 = ClientError::FieldEmpty { field_name: "foo".to_string() };
        let err2 = ClientError::FieldEmpty { field_name: "bar".to_string() };
        // THEN
        assert_ne!(err1, err2);
    }

    #[test]
    fn test_07_given_simple_variants_when_comparing_then_they_should_be_equal_or_not() {
        // THEN
        assert_eq!(ClientError::NegativeAmount, ClientError::NegativeAmount);
        assert_eq!(ClientError::PositiveAmount, ClientError::PositiveAmount);
        assert_eq!(ClientError::ZeroAmount, ClientError::ZeroAmount);
        assert_eq!(ClientError::BalancesEmpty, ClientError::BalancesEmpty);
        assert_eq!(ClientError::Unknown(anyhow!("err1")), ClientError::Unknown(anyhow!("err2")));
        assert_ne!(ClientError::NegativeAmount, ClientError::PositiveAmount);
    }

    #[test]
    fn test_08_given_field_invalid_and_field_max_length_when_comparing_then_they_should_be_equal_or_not() {
        // GIVEN
        let err1 = ClientError::FieldInvalid { field_name: "foo".to_string(), value: "a".to_string() };
        let err2 = ClientError::FieldInvalid { field_name: "foo".to_string(), value: "a".to_string() };
        let err3 = ClientError::FieldInvalid { field_name: "foo".to_string(), value: "b".to_string() };
        let err4 = ClientError::FieldMaxLength { field_name: "foo".to_string(), max_length: 5 };
        let err5 = ClientError::FieldMaxLength { field_name: "foo".to_string(), max_length: 5 };
        let err6 = ClientError::FieldMaxLength { field_name: "foo".to_string(), max_length: 6 };
        // THEN
        assert_eq!(err1, err2);
        assert_ne!(err1, err3);
        assert_eq!(err4, err5);
        assert_ne!(err4, err6);
    }

    #[test]
    fn test_09_given_all_variants_when_calling_code_then_should_return_expected_code() {
        // GIVEN
        let doc = "123".to_string();
        let id = ClientId::default();
        let d = Document::new("A").unwrap();
        // THEN
        assert_eq!(ClientError::Duplicate { document: doc.clone() }.code(), "CLIENT_DUPLICATE");
        assert_eq!(ClientError::NotFoundById { id_document: id.clone() }.code(), "CLIENT_NOT_FOUND_BY_ID");
        assert_eq!(ClientError::NotFoundByDocument { document: d.clone() }.code(), "CLIENT_NOT_FOUND_BY_DOCUMENT");
        assert_eq!(ClientError::FieldEmpty { field_name: "foo".to_string() }.code(), "CLIENT_FOO_EMPTY");
        assert_eq!(ClientError::FieldInvalid { field_name: "foo".to_string(), value: "bar".to_string() }.code(), "CLIENT_FOO_INVALID");
        assert_eq!(ClientError::FieldMaxLength { field_name: "foo".to_string(), max_length: 5 }.code(), "CLIENT_FOO_MAX_LENGTH");
        assert_eq!(ClientError::NegativeAmount.code(), "CLIENT_NEGATIVE_BALANCE");
        assert_eq!(ClientError::PositiveAmount.code(), "CLIENT_POSITIVE_BALANCE");
        assert_eq!(ClientError::ZeroAmount.code(), "CLIENT_ZERO_BALANCE");
        assert_eq!(ClientError::BalancesEmpty.code(), "CLIENT_BALANCES_EMPTY");
        assert_eq!(ClientError::Unknown(anyhow!("err")).code(), "CLIENT_UNKNOWN_ERROR");
    }

    #[test]
    fn test_10_given_all_variants_when_display_then_should_return_expected_message() {
        // GIVEN
        let doc = "123".to_string();
        let id = ClientId::default();
        let d = Document::new("A").unwrap();
        // THEN
        assert_eq!(format!("{}", ClientError::Duplicate { document: doc.clone() }), format!("client with document {} already exists", doc));
        assert_eq!(format!("{}", ClientError::NotFoundById { id_document: id.clone() }), format!("client not found by id {}", id));
        assert_eq!(format!("{}", ClientError::NotFoundByDocument { document: d.clone() }), format!("client not found by document {}", d));
        assert_eq!(format!("{}", ClientError::FieldEmpty { field_name: "foo".to_string() }), "client foo cannot be empty");
        assert_eq!(format!("{}", ClientError::FieldInvalid { field_name: "foo".to_string(), value: "bar".to_string() }), "client foo is invalid: bar");
        assert_eq!(format!("{}", ClientError::FieldMaxLength { field_name: "foo".to_string(), max_length: 5 }), "client foo is too long. Max length is 5");
        assert_eq!(format!("{}", ClientError::NegativeAmount), "client amount cannot be negative");
        assert_eq!(format!("{}", ClientError::PositiveAmount), "client amount cannot be positive");
        assert_eq!(format!("{}", ClientError::ZeroAmount), "client amount cannot be zero");
        assert_eq!(format!("{}", ClientError::BalancesEmpty), "balances are empty");
        // Unknown error: solo chequear que contiene el string
        let unknown = format!("{}", ClientError::Unknown(anyhow!("err")));
        assert!(unknown.contains("err"));
    }
}
