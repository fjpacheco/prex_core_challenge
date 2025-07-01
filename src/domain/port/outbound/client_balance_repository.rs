use crate::domain::model::entity::balance::Balance;
use crate::domain::model::error::ClientError;
use crate::domain::model::value::client_id::ClientId;
use crate::domain::model::{
    dto::{
        create_client::CreateClientRequest, credit_transaction::CreditTransactionRequest,
        debit_transaction::DebitTransactionRequest, get_balance::GetClientRequest,
    },
    entity::client::Client,
};

#[allow(unused_imports)]
use crate::domain::model::value::document::Document;

/// `ClientRepository` represents a store of all [Client]s.
#[cfg_attr(test, mockall::automock)]
pub trait ClientBalanceRepository: Send + Sync + 'static {
    /// Asynchronously persist a new [Client]. Returns the created [Client].
    ///
    /// # Errors
    ///
    /// - [ClientError::Duplicate] if an [Client] with the same [Document] already exists.
    /// - [ClientError::Unknown] if the [Client] cannot be created.
    fn create_client(
        &self,
        req: &CreateClientRequest,
    ) -> impl Future<Output = Result<Client, ClientError>> + Send;

    /// Asynchronously check if a [ClientId] exists.
    ///
    /// # Errors
    ///
    /// - [ClientError::NotFoundById] if an [Client] with the given [ClientId] does not exist.
    /// - [ClientError::Unknown] if the [Client] cannot be found.
    fn client_id_exists(
        &self,
        client_id: &ClientId,
    ) -> impl Future<Output = Result<bool, ClientError>> + Send;

    /// Asynchronously get a [Client] by [Document].
    ///
    /// # Errors
    ///
    /// - [ClientError::NotFoundByDocument] if an [Client] with the given [Document] does not exist.
    /// - [ClientError::Unknown] if the [Client] cannot be found.
    fn get_client_by_document(
        &self,
        document: &Document,
    ) -> impl Future<Output = Result<Client, ClientError>> + Send;

    /// Asynchronously credit the balance of a [Client]. Returns the updated [Balance].
    ///
    /// # Errors
    ///
    /// - [ClientError::NotFoundById] if an [Client] with the given [ClientId] does not exist.
    /// - [ClientError::NegativeAmount] if the amount is negative.
    /// - [ClientError::ZeroAmount] if the amount is zero.
    /// - [ClientError::Unknown] if the [Client] cannot be credited.
    fn credit_balance(
        &self,
        req: &CreditTransactionRequest,
    ) -> impl Future<Output = Result<Balance, ClientError>> + Send;

    /// Asynchronously debit the balance of a [Client]. Returns the updated [Balance].
    ///
    /// # Errors
    ///
    /// - [ClientError::NotFoundById] if an [Client] with the given [ClientId] does not exist.
    /// - [ClientError::PositiveAmount] if the amount is positive.
    /// - [ClientError::ZeroAmount] if the amount is zero.
    /// - [ClientError::Unknown] if the [Client] cannot be debited.
    fn debit_balance(
        &self,
        req: &DebitTransactionRequest,
    ) -> impl Future<Output = Result<Balance, ClientError>> + Send;

    /// Asynchronously get the [Balance] of a [Client].
    ///
    /// # Errors
    ///
    /// - [ClientError::NotFoundById] if an [Client] with the given [ClientId] does not exist.
    /// - [ClientError::Unknown] if the [Client] cannot be found.
    fn get_balance_by_client_id(
        &self,
        req: &GetClientRequest,
    ) -> impl Future<Output = Result<Balance, ClientError>> + Send;

    /// Asynchronously get the [Client] by id. Returns the [Client].
    ///
    /// # Errors
    ///
    /// - [ClientError::NotFoundById] if an [Client] with the given [ClientId] does not exist.
    /// - [ClientError::Unknown] if the [Client] cannot be found.
    fn get_client(
        &self,
        req: &GetClientRequest,
    ) -> impl Future<Output = Result<Client, ClientError>> + Send;

    /// Asynchronously returns if balances are empty.
    ///
    /// # Errors
    ///
    /// - [ClientError::Unknown] if the balances cannot be checked.
    fn are_balances_empty(&self) -> impl Future<Output = Result<bool, ClientError>> + Send;

    /// Asynchronously resets balances of all [Client]s to zero and returns the previous [Balance]s with their old balances.
    ///
    /// # Errors
    ///
    /// - [ClientError::Unknown] if the balances cannot be reset.
    fn reset_all_balances_to_zero(
        &self,
    ) -> impl Future<Output = Result<Vec<Balance>, ClientError>> + Send;

    /// Asynchronously given a old list of [Balance]s, merge them with the actual balances of the [Client]s.
    ///
    /// # Errors
    ///
    /// - [ClientError::Unknown] if the balances cannot be merged.
    fn merge_old_balances(
        &self,
        old_balances: Vec<Balance>,
    ) -> impl Future<Output = Result<(), ClientError>> + Send;
}
