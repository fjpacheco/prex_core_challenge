use rust_decimal::Decimal;

use crate::domain::model::value::client_id::ClientId;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Balance {
    id: ClientId,
    balance: Decimal,
}

impl Balance {
    pub fn new(id: ClientId, balance: Decimal) -> Self {
        Self { id, balance }
    }

    pub fn client_id(&self) -> &ClientId {
        &self.id
    }

    pub fn balance(&self) -> &Decimal {
        &self.balance
    }

    /// Sets the balance of the [Balance] and returns the old balance.
    pub fn set_balance(&mut self, balance: Decimal) -> Decimal {
        let old_balance = self.balance;
        self.balance = balance;
        old_balance
    }
}
