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
