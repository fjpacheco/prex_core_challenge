use serde::{Deserialize, Serialize};

use crate::{
    domain::model::{
        dto::create_client::CreateClientRequest,
        entity::client::Client,
        value::{
            birth_date::BirthDate, client_name::ClientName, country::Country, document::Document,
        },
    },
    infrastructure::inbound::http::error::ApiError,
};

/// The body of an [Client] creation request.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CreateClientHttpRequestBody {
    name: String,
    birth_date: String,
    document: String,
    country: String,
}

impl CreateClientHttpRequestBody {
    /// Converts the HTTP request body into a domain request.
    pub fn try_into_domain(self) -> Result<CreateClientRequest, ApiError> {
        let name = ClientName::new(&self.name)?;
        let document = Document::new(&self.document)?;
        let country = Country::new(&self.country)?;
        let birth_date = BirthDate::new(&self.birth_date)?;
        Ok(CreateClientRequest::new(
            name, birth_date, document, country,
        ))
    }
}

#[derive(Debug, Serialize)]
pub struct CreateClientHttpResponseBody {
    id: String,
}

impl From<Client> for CreateClientHttpResponseBody {
    fn from(client: Client) -> Self {
        Self {
            id: client.id().to_string(),
        }
    }
}
