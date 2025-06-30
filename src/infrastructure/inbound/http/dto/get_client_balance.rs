use serde::{Deserialize, Serialize};

use crate::{
    domain::model::{
        dto::get_balance::GetClientRequest,
        entity::{balance::Balance, client::Client},
        value::client_id::ClientId,
    },
    infrastructure::inbound::http::error::ApiError,
};

/// The path to get the client balance.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct GetClientBalanceHttpRequestPath {
    user_id: String,
}
impl GetClientBalanceHttpRequestPath {
    /// Converts the HTTP request path into a domain request.
    pub fn try_into_domain(self) -> Result<GetClientRequest, ApiError> {
        let client_id = ClientId::try_from(self.user_id)?;
        Ok(GetClientRequest::new(client_id))
    }
}

#[derive(Debug, Serialize)]
pub struct GetClientBalanceHttpResponseBody {
    id: String,
    name: String,
    birth_date: String,
    document: String,
    country: String,
    balance: String,
}

impl From<(Client, Balance)> for GetClientBalanceHttpResponseBody {
    fn from((client, client_balance): (Client, Balance)) -> Self {
        Self {
            id: client_balance.client_id().to_string(),
            name: client.name().to_string(),
            birth_date: client.birth_date().to_string(),
            document: client.document().to_string(),
            country: client.country().to_string(),
            balance: client_balance.balance().to_string(),
        }
    }
}
