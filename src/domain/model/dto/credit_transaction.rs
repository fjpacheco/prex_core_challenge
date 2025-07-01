use derive_more::From;
use rust_decimal::Decimal;

use crate::domain::model::{error::ClientError, value::client_id::ClientId};

#[allow(unused_imports)]
use crate::domain::model::entity::client::Client;

/// The fields required by the domain to credit balance to a [Client].
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, From)]
pub struct CreditTransactionRequest {
    client_id: ClientId,
    /// The amount to credit to the [Client] balance. Always positive.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::value::client_id::ClientId;
    use rust_decimal::Decimal;

    #[test]
    fn test_01_given_positive_amount_when_creating_credit_transaction_then_should_be_ok() {
        let client_id = ClientId::default();
        let req = CreditTransactionRequest::new(client_id.clone(), Decimal::from(100));
        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(req.client_id(), &client_id);
        assert_eq!(req.amount(), &Decimal::from(100));
    }

    #[test]
    fn test_02_given_negative_amount_when_creating_credit_transaction_then_should_fail() {
        let client_id = ClientId::default();
        let req = CreditTransactionRequest::new(client_id, Decimal::from(-100));
        assert!(req.is_err());
        assert_eq!(req.err().unwrap(), ClientError::NegativeAmount);
    }

    #[test]
    fn test_03_given_zero_amount_when_creating_credit_transaction_then_should_fail() {
        let client_id = ClientId::default();
        let req = CreditTransactionRequest::new(client_id, Decimal::ZERO);
        assert!(req.is_err());
        assert_eq!(req.err().unwrap(), ClientError::ZeroAmount);
    }
}
