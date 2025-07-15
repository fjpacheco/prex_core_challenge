use crate::domain::model::entity::client::Client;
use crate::domain::model::error::ClientError;
use crate::domain::model::{
    dto::{
        create_client::CreateClientRequest, credit_transaction::CreditTransactionRequest,
        debit_transaction::DebitTransactionRequest, get_balance::GetClientRequest,
    },
    entity::balance::Balance,
};

#[allow(unused_imports)]
use crate::domain::model::value::document::Document;

/// `ClientBalanceService` is the public API for the balance client domain.
pub trait ClientBalanceService: Send + Sync + 'static {
    /// Asynchronously create a new [Client]. Returns the created [Client].
    ///
    /// # Errors
    ///
    /// - [ClientError::Duplicate] if an [Client] with the same [Document] already exists.
    fn create_client(
        &self,
        req: &CreateClientRequest,
    ) -> impl Future<Output = Result<Client, ClientError>> + Send;

    /// Asynchronously get the [Client] by id. Returns the [Client].
    ///
    /// # Errors
    ///
    /// - [ClientError::NotFoundById] if the [Client] does not exist.
    fn get_client_by_id(
        &self,
        req: &GetClientRequest,
    ) -> impl Future<Output = Result<Client, ClientError>> + Send;

    /// Asynchronously credit the balance of a [Client]. Returns the updated [Balance].
    ///
    /// # Errors
    ///
    /// - [ClientError::NotFoundById] if the [Client] does not exist.
    /// - [ClientError::NegativeAmount] if the amount is negative.
    /// - [ClientError::ZeroAmount] if the amount is zero.
    fn credit_balance(
        &self,
        req: &CreditTransactionRequest,
    ) -> impl Future<Output = Result<Balance, ClientError>> + Send;

    /// Asynchronously debit the balance of a [Client]. Returns the updated [Balance].
    ///
    /// # Errors
    ///
    /// - [ClientError::NotFoundById] if the [Client] does not exist.
    /// - [ClientError::PositiveAmount] if the amount is positive.
    /// - [ClientError::ZeroAmount] if the amount is zero.
    fn debit_balance(
        &self,
        req: &DebitTransactionRequest,
    ) -> impl Future<Output = Result<Balance, ClientError>> + Send;

    /// Asynchronously get the balance of a [Client]. Returns the [Balance].
    ///
    /// # Errors
    ///
    /// - [ClientError::NotFoundById] if the [Client] does not exist.
    fn get_balance_by_client_id(
        &self,
        req: &GetClientRequest,
    ) -> impl Future<Output = Result<Balance, ClientError>> + Send;

    /// Asynchronously set the balances of all [Balance]s to zero and export the previous balances to the external system.
    ///
    /// # Errors
    ///
    /// - [ClientError::BalancesEmpty] if the balances are empty.
    /// - [ClientError::Unknown] if the balances cannot be exported.
    fn store_balances(&self) -> impl Future<Output = Result<(), ClientError>> + Send;
}
