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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_01_given_a_balance_when_creating_it_then_it_should_be_created() {
        let client_id = ClientId::new("1").unwrap();
        let balance = Balance::new(client_id.clone(), Decimal::from(100));
        assert_eq!(balance.balance(), &Decimal::from(100));
        assert_eq!(balance.client_id(), &client_id);
    }

    #[test]
    fn test_02_given_a_balance_when_setting_new_balance_then_it_should_update_and_return_old() {
        let mut balance = Balance::new(ClientId::new("1").unwrap(), Decimal::from(100));
        let old = balance.set_balance(Decimal::from(200));
        assert_eq!(old, Decimal::from(100));
        assert_eq!(balance.balance(), &Decimal::from(200));
    }

    #[test]
    fn test_03_given_a_balance_with_zero_then_it_should_be_created() {
        let balance = Balance::new(ClientId::new("1").unwrap(), Decimal::from(0));
        assert_eq!(balance.balance(), &Decimal::from(0));
    }

    #[test]
    fn test_04_given_a_balance_with_negative_value_then_it_should_be_created() {
        let balance = Balance::new(ClientId::new("1").unwrap(), Decimal::from(-50));
        assert_eq!(balance.balance(), &Decimal::from(-50));
    }

    #[test]
    fn test_05_given_a_balance_when_setting_new_balance_then_id_should_remain_unchanged() {
        let mut balance = Balance::new(ClientId::new("1").unwrap(), Decimal::from(100));
        let id_before = balance.client_id().clone();
        balance.set_balance(Decimal::from(300));
        assert_eq!(balance.client_id(), &id_before);
    }

    #[test]
    fn test_06_given_a_balance_when_increasing_then_balance_should_increase() {
        let mut balance = Balance::new(ClientId::new("1").unwrap(), Decimal::from(50));
        balance.set_balance(balance.balance() + Decimal::from(25));
        assert_eq!(balance.balance(), &Decimal::from(75));
    }

    #[test]
    fn test_07_given_a_balance_when_decreasing_then_balance_should_decrease() {
        let mut balance = Balance::new(ClientId::new("1").unwrap(), Decimal::from(50));
        balance.set_balance(balance.balance() - Decimal::from(20));
        assert_eq!(balance.balance(), &Decimal::from(30));
    }

    #[test]
    fn test_08_given_a_balance_when_changing_from_negative_to_positive_then_should_reflect() {
        let mut balance = Balance::new(ClientId::new("1").unwrap(), Decimal::from(-10));
        balance.set_balance(Decimal::from(20));
        assert_eq!(balance.balance(), &Decimal::from(20));
    }

    #[test]
    fn test_09_given_a_balance_when_changing_from_positive_to_negative_then_should_reflect() {
        let mut balance = Balance::new(ClientId::new("1").unwrap(), Decimal::from(30));
        balance.set_balance(Decimal::from(-5));
        assert_eq!(balance.balance(), &Decimal::from(-5));
    }
}
