use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{
    domain::model::{
        dto::debit_transaction::DebitTransactionRequest, entity::balance::Balance,
        value::client_id::ClientId,
    },
    infrastructure::inbound::http::error::ApiError,
};

#[allow(unused_imports)]
use crate::domain::model::entity::client::Client;

/// The body of an [Client] debit request.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct NewDebitTransactionHttpRequestBody {
    client_id: String,
    amount: Decimal,
}

impl NewDebitTransactionHttpRequestBody {
    /// Converts the HTTP request body into a domain request.
    pub fn try_into_domain(self) -> Result<DebitTransactionRequest, ApiError> {
        let client_id = ClientId::try_from(self.client_id)?;
        let debit_transaction_request = DebitTransactionRequest::new(client_id, self.amount)?;
        Ok(debit_transaction_request)
    }
}

#[derive(Debug, Serialize)]
pub struct NewDebitTransactionHttpResponseBody {
    id: String,
    balance: String,
}

impl From<Balance> for NewDebitTransactionHttpResponseBody {
    fn from(client_balance: Balance) -> Self {
        Self {
            id: client_balance.client_id().to_string(),
            balance: client_balance.balance().to_string(),
        }
    }
}
