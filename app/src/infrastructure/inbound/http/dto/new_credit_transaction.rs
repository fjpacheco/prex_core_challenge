use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{
    domain::model::{
        dto::credit_transaction::CreditTransactionRequest, entity::balance::Balance,
        value::client_id::ClientId,
    },
    infrastructure::inbound::http::error::ApiError,
};

#[allow(unused_imports)]
use crate::domain::model::entity::client::Client;

/// The body of an [Client] credit request.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct NewCreditTransactionHttpRequestBody {
    client_id: String,
    amount: Decimal,
}

impl NewCreditTransactionHttpRequestBody {
    /// Converts the HTTP request body into a domain request.
    pub fn try_into_domain(self) -> Result<CreditTransactionRequest, ApiError> {
        let client_id = ClientId::try_from(self.client_id)?;
        let credit_transaction_request = CreditTransactionRequest::new(client_id, self.amount)?;
        Ok(credit_transaction_request)
    }
}

#[derive(Debug, Serialize)]
pub struct NewCreditTransactionHttpResponseBody {
    id: String,
    balance: String,
}

impl From<Balance> for NewCreditTransactionHttpResponseBody {
    fn from(client_balance: Balance) -> Self {
        Self {
            id: client_balance.client_id().to_string(),
            balance: client_balance.balance().to_string(),
        }
    }
}
