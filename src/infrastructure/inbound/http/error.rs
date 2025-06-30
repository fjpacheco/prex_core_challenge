use crate::domain::model::error::ClientError;
use actix_web::{
    HttpResponse, ResponseError,
    http::{StatusCode, header::ContentType},
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiError {
    #[serde(skip)]
    status_code: u16,
    error_code: String,
    error_message: String,
}

impl ApiError {
    pub fn new(status_code: u16, error_code: String, error_message: String) -> Self {
        Self {
            status_code,
            error_code,
            error_message,
        }
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error_message)
    }
}

impl actix_web::error::ResponseError for ClientError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .json(ApiError::new(
                self.status_code().as_u16(),
                self.code().to_string(),
                self.to_string(),
            ))
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            ClientError::Duplicate { .. } => StatusCode::CONFLICT,
            ClientError::NotFoundById { .. } => StatusCode::NOT_FOUND,
            ClientError::NegativeAmount => StatusCode::BAD_REQUEST,
            ClientError::ZeroAmount => StatusCode::BAD_REQUEST,
            ClientError::NotFoundByDocument { .. } => StatusCode::NOT_FOUND,
            ClientError::FieldEmpty { .. } => StatusCode::BAD_REQUEST,
            ClientError::FieldInvalid { .. } => StatusCode::BAD_REQUEST,
            ClientError::FieldMaxLength { .. } => StatusCode::BAD_REQUEST,
            ClientError::PositiveAmount => StatusCode::BAD_REQUEST,
            ClientError::BalancesEmpty => StatusCode::NOT_FOUND,
            ClientError::Unknown(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl actix_web::error::ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(
            StatusCode::from_u16(self.status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
        )
        .insert_header(ContentType::json())
        .json(self)
    }
}

impl From<ClientError> for ApiError {
    fn from(error: ClientError) -> Self {
        let status_code = error.status_code();
        Self::new(
            status_code.as_u16(),
            error.code().to_string(),
            error.to_string(),
        )
    }
}
