use derive_more::From;
use rust_decimal::Decimal;

use crate::domain::model::{error::ClientError, value::client_id::ClientId};

#[allow(unused_imports)]
use crate::domain::model::entity::client::Client;

/// The fields required by the domain to credit balance to a [Client].
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, From)]
pub struct CreditTransactionRequest {
    client_id: ClientId,
    amount: Decimal,
}

impl CreditTransactionRequest {
    pub fn new(client_id: ClientId, amount: Decimal) -> Result<Self, ClientError> {
        if amount < Decimal::ZERO {
            return Err(ClientError::NegativeAmount);
        }

        if amount == Decimal::ZERO {
            return Err(ClientError::ZeroAmount);
        }

        Ok(Self { client_id, amount })
    }

    pub fn client_id(&self) -> &ClientId {
        &self.client_id
    }

    pub fn amount(&self) -> &Decimal {
        &self.amount
    }
}
