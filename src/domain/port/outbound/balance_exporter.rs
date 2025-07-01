use crate::domain::model::{entity::balance::Balance, error::ClientError};

/// `BalanceExporter` represents a service to export [Balance] data.
#[cfg_attr(test, mockall::automock)]
pub trait BalanceExporter: Send + Sync + 'static {
    /// Asynchronously given a list of [Balance]s, export them to the external system.
    ///
    /// # Errors
    ///
    /// - [ClientError::BalancesEmpty] if the balances are empty.
    /// - [ClientError::Unknown] if the balances cannot be exported.
    fn export_balances(
        &self,
        balances: &[Balance],
    ) -> impl Future<Output = Result<(), ClientError>> + Send;
}
