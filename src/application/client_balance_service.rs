/*!
   Module `client_balance_service` provides the canonical implementation of the [ClientBalanceService] port. All
   client-balance-domain logic is defined here with use cases.
*/

use anyhow::Context;

use crate::domain::{
    model::{
        dto::{
            create_client::CreateClientRequest, credit_transaction::CreditTransactionRequest,
            debit_transaction::DebitTransactionRequest, get_balance::GetClientRequest,
        },
        entity::{balance::Balance, client::Client},
        error::ClientError,
        value::{client_id::ClientId, document::Document},
    },
    port::{
        inbound::client_balance_service::ClientBalanceService,
        outbound::{
            balance_exporter::BalanceExporter, client_balance_repository::ClientBalanceRepository,
        },
    },
};

/// Canonical implementation of the [ClientBalanceService] port, through which the client balance domain API is consumed.
#[derive(Debug, Clone)]
pub struct Service<C, E>
where
    C: ClientBalanceRepository,
    E: BalanceExporter,
{
    client_repository: C,
    balance_exporter: E,
}

impl<C, E> Service<C, E>
where
    C: ClientBalanceRepository,
    E: BalanceExporter,
{
    pub fn new(client_repository: C, balance_exporter: E) -> Self {
        Self {
            client_repository,
            balance_exporter,
        }
    }

    async fn validate_client_exists(&self, client_id: &ClientId) -> Result<(), ClientError> {
        if !self.client_repository.client_id_exists(client_id).await? {
            return Err(ClientError::NotFoundById {
                id_document: client_id.clone(),
            });
        }

        Ok(())
    }

    async fn validate_client_exists_by_document(
        &self,
        document: &Document,
    ) -> Result<(), ClientError> {
        let result = self
            .client_repository
            .get_client_by_document(document)
            .await;
        if result.is_err() {
            return Ok(());
        }
        Err(ClientError::Duplicate {
            document: document.to_string(),
        })
    }
}

impl<C, E> ClientBalanceService for Service<C, E>
where
    C: ClientBalanceRepository,
    E: BalanceExporter,
{
    async fn create_client(&self, req: &CreateClientRequest) -> Result<Client, ClientError> {
        self.validate_client_exists_by_document(req.document())
            .await?;

        let client = self.client_repository.create_client(req).await?;

        if let Err(e) = self
            .client_repository
            .init_client_balance(client.id())
            .await
        {
            tracing::warn!("Error initializing client balance: {:?}", e);
            tracing::warn!(
                "Deleting client {:?} because it cannot exist without a balance",
                client.id()
            );
            self.client_repository.delete_client(client.id()).await?;
        }

        Ok(client)
    }

    async fn credit_balance(&self, req: &CreditTransactionRequest) -> Result<Balance, ClientError> {
        self.validate_client_exists(req.client_id()).await?;

        let balance = self.client_repository.credit_balance(req).await?;
        Ok(balance)
    }

    async fn debit_balance(&self, req: &DebitTransactionRequest) -> Result<Balance, ClientError> {
        self.validate_client_exists(req.client_id()).await?;

        let balance = self.client_repository.debit_balance(req).await?;
        Ok(balance)
    }

    async fn get_balance_by_client_id(
        &self,
        req: &GetClientRequest,
    ) -> Result<Balance, ClientError> {
        self.validate_client_exists(req.client_id()).await?;

        let balance: Balance = self.client_repository.get_balance_by_client_id(req).await?;
        Ok(balance)
    }

    async fn get_client_by_id(&self, req: &GetClientRequest) -> Result<Client, ClientError> {
        self.validate_client_exists(req.client_id()).await?;

        let client: Client = self.client_repository.get_client(req).await?;
        Ok(client)
    }

    async fn store_balances(&self) -> Result<(), ClientError> {
        if self.client_repository.are_balances_empty().await? {
            return Err(ClientError::BalancesEmpty);
        }

        let old_balance_clients = self
            .client_repository
            .reset_all_balances_to_zero()
            .await
            .with_context(|| "Error resetting all balances to zero")?;

        if let Err(e) = self
            .balance_exporter
            .export_balances(&old_balance_clients)
            .await
            .with_context(|| "Error exporting balances")
        {
            // If the merge fails, we need handle a way to recover the old balances! Maybe we can use a retry mechanism to merge the balances again,
            // or we can use a event bus to notify the system that recovery is needed and the system will be able to recover the balances.
            // Temporarily we are merging the old balances again!
            tracing::warn!("Error exporting balances, merging old balances again...");
            self.client_repository
                .merge_old_balances(old_balance_clients)
                .await
                .with_context(|| "Error merging old balances")?;
            return Err(ClientError::Unknown(e));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    use rust_decimal::Decimal;

    use crate::domain::{
        model::value::{
            birth_date::BirthDate, client_name::ClientName, country::Country, document::Document,
        },
        port::outbound::{
            balance_exporter::MockBalanceExporter,
            client_balance_repository::MockClientBalanceRepository,
        },
    };

    use super::*;

    fn setup_mocks() -> (MockClientBalanceRepository, MockBalanceExporter) {
        let balance_exporter = MockBalanceExporter::default();
        let (arc_mutex_clients, arc_mutex_client_balances) = (
            Arc::new(Mutex::new(HashMap::new())),
            Arc::new(Mutex::new(HashMap::new())),
        );
        let mut client_balance_repository = MockClientBalanceRepository::default();
        let (arc_mutex_clients_1, arc_mutex_client_balances_1) =
            (arc_mutex_clients.clone(), arc_mutex_client_balances.clone());
        client_balance_repository
            .expect_create_client()
            .returning(move |req| {
                let client_id = ClientId::default();
                let client = Client::new(
                    client_id.clone(),
                    req.name().clone(),
                    req.birth_date().clone(),
                    req.document().clone(),
                    req.country().clone(),
                );

                arc_mutex_clients_1
                    .lock()
                    .unwrap()
                    .insert(client_id.clone(), client.clone());
                Box::pin(async move { Ok(client) })
            });

        client_balance_repository
            .expect_init_client_balance()
            .returning(move |client_id| {
                let client_balance = Balance::new(client_id.clone(), Decimal::from(0));
                arc_mutex_client_balances_1
                    .lock()
                    .unwrap()
                    .insert(client_id.clone(), client_balance.clone());
                Box::pin(async move { Ok(client_balance) })
            });
        let arc_mutex_clients_2 = arc_mutex_clients.clone();
        client_balance_repository
            .expect_client_id_exists()
            .returning(move |client_id| {
                let result = arc_mutex_clients_2.lock().unwrap().contains_key(client_id);
                Box::pin(async move { Ok(result) })
            });

        let arc_mutex_clients_3 = arc_mutex_clients.clone();
        client_balance_repository
            .expect_get_client_by_document()
            .returning(move |document| {
                let document_clone = document.clone();
                let result = arc_mutex_clients_3
                    .lock()
                    .unwrap()
                    .values()
                    .find(|client| client.document() == document)
                    .cloned();
                if let Some(client) = result {
                    Box::pin(async move { Ok(client.clone()) })
                } else {
                    Box::pin(async move {
                        Err(ClientError::NotFoundByDocument {
                            document: document_clone.clone(),
                        })
                    })
                }
            });

        let arc_mutex_client_balances_3 = arc_mutex_client_balances.clone();
        client_balance_repository
            .expect_get_balance_by_client_id()
            .returning(move |req| {
                let client_id_clone = req.client_id().clone();
                let result = arc_mutex_client_balances_3
                    .lock()
                    .unwrap()
                    .get(req.client_id())
                    .cloned();
                if let Some(balance) = result {
                    Box::pin(async move { Ok(balance.clone()) })
                } else {
                    Box::pin(async move {
                        Err(ClientError::NotFoundById {
                            id_document: client_id_clone.clone(),
                        })
                    })
                }
            });

        let arc_mutex_client_balances_4 = arc_mutex_client_balances.clone();
        client_balance_repository
            .expect_credit_balance()
            .returning(move |req| {
                let client_id_clone = req.client_id().clone();
                if let Some(balance) = arc_mutex_client_balances_4
                    .lock()
                    .unwrap()
                    .get_mut(req.client_id())
                {
                    let new_balance = balance.balance() + req.amount();
                    balance.set_balance(new_balance);
                    let client_balance = balance.clone();
                    Box::pin(async move { Ok(client_balance) })
                } else {
                    Box::pin(async move {
                        Err(ClientError::NotFoundById {
                            id_document: client_id_clone.clone(),
                        })
                    })
                }
            });

        let arc_mutex_client_balances_5 = arc_mutex_client_balances.clone();
        client_balance_repository
            .expect_debit_balance()
            .returning(move |req| {
                let client_id_clone = req.client_id().clone();
                if let Some(balance) = arc_mutex_client_balances_5
                    .lock()
                    .unwrap()
                    .get_mut(req.client_id())
                {
                    let new_balance = balance.balance() + req.amount();
                    balance.set_balance(new_balance);
                    let client_balance = balance.clone();
                    Box::pin(async move { Ok(client_balance) })
                } else {
                    Box::pin(async move {
                        Err(ClientError::NotFoundById {
                            id_document: client_id_clone.clone(),
                        })
                    })
                }
            });

        (client_balance_repository, balance_exporter)
    }

    #[tokio::test]
    async fn test_01_given_a_client_when_creating_it_then_it_should_return_the_client_id_created() {
        // SETUP
        let (client_balance_repository, balance_exporter) = setup_mocks();
        let client_balance_service = Service::new(client_balance_repository, balance_exporter);

        // GIVEN
        let req_create = CreateClientRequest::new(
            ClientName::new("John Doe").unwrap(),
            BirthDate::new("1990-01-01").unwrap(),
            Document::new("1234567890").unwrap(),
            Country::new("US").unwrap(),
        );

        // WHEN
        let result_create = client_balance_service.create_client(&req_create).await;

        // ASSERT
        assert!(result_create.is_ok());
        assert!(!result_create.unwrap().id().to_string().is_empty());
    }

    #[tokio::test]
    async fn test_02_given_two_clients_with_the_same_document_when_creating_it_then_it_should_return_an_error()
     {
        // SETUP
        let (client_balance_repository, balance_exporter) = setup_mocks();
        let client_balance_service = Service::new(client_balance_repository, balance_exporter);

        // GIVEN
        let document = "1234567890";
        let req_create_1 = CreateClientRequest::new(
            ClientName::new("John Doe").unwrap(),
            BirthDate::new("1990-01-01").unwrap(),
            Document::new(document).unwrap(),
            Country::new("US").unwrap(),
        );
        let req_create_2 = CreateClientRequest::new(
            ClientName::new("John Doe").unwrap(),
            BirthDate::new("1990-01-01").unwrap(),
            Document::new(document).unwrap(),
            Country::new("US").unwrap(),
        );

        // WHEN
        let result_create_1 = client_balance_service.create_client(&req_create_1).await;
        let result_create_2 = client_balance_service.create_client(&req_create_2).await;

        // ASSERT
        assert!(result_create_1.is_ok());
        assert!(result_create_2.is_err());
        assert_eq!(
            result_create_2.err().unwrap(),
            ClientError::Duplicate {
                document: document.to_string(),
            }
        );
    }

    #[tokio::test]
    async fn test_03_given_a_client_created_when_getting_client_balance_then_it_should_return_the_client_balance_equal_to_zero()
     {
        // SETUP
        let (client_balance_repository, balance_exporter) = setup_mocks();
        let client_balance_service = Service::new(client_balance_repository, balance_exporter);

        // GIVEN
        let req_create = CreateClientRequest::new(
            ClientName::new("John Doe").unwrap(),
            BirthDate::new("1990-01-01").unwrap(),
            Document::new("1234567890").unwrap(),
            Country::new("US").unwrap(),
        );
        let result_create = client_balance_service.create_client(&req_create).await;
        let client_id = result_create.unwrap().id().clone();

        // WHEN
        let req_get = GetClientRequest::new(client_id.clone());
        let result_get = client_balance_service
            .get_balance_by_client_id(&req_get)
            .await;

        // ASSERT
        assert!(result_get.is_ok());
        let balance = result_get.unwrap();
        assert_eq!(balance.balance(), &Decimal::from(0));
    }

    #[tokio::test]
    async fn test_04_given_a_client_created_when_credit_balance_then_it_should_be_updated_with_the_new_balance()
     {
        // SETUP
        let (client_balance_repository, balance_exporter) = setup_mocks();
        let client_balance_service = Service::new(client_balance_repository, balance_exporter);

        // GIVEN
        let req = CreateClientRequest::new(
            ClientName::new("John Doe").unwrap(),
            BirthDate::new("1990-01-01").unwrap(),
            Document::new("1234567890").unwrap(),
            Country::new("US").unwrap(),
        );

        let result = client_balance_service.create_client(&req).await.unwrap();
        let client_id = result.id();
        let req_transaction =
            CreditTransactionRequest::new(client_id.clone(), Decimal::from(100)).unwrap();
        let req_get = GetClientRequest::new(client_id.clone());

        // WHEN
        let result_transaction = client_balance_service
            .credit_balance(&req_transaction)
            .await
            .unwrap();
        let result_get = client_balance_service
            .get_balance_by_client_id(&req_get)
            .await
            .unwrap();

        // ASSERT
        assert_eq!(result_transaction.balance(), &Decimal::from(100));
        assert_eq!(result_transaction.client_id(), client_id);
        assert_eq!(result_get.balance(), &Decimal::from(100));
        assert_eq!(result_get.client_id(), client_id);
    }

    #[tokio::test]
    async fn test_05_given_a_client_created_when_credit_and_debit_balance_then_it_should_be_updated_with_the_new_balance()
     {
        // SETUP
        let (client_balance_repository, balance_exporter) = setup_mocks();
        let client_balance_service = Service::new(client_balance_repository, balance_exporter);

        // GIVEN
        let req = CreateClientRequest::new(
            ClientName::new("John Doe").unwrap(),
            BirthDate::new("1990-01-01").unwrap(),
            Document::new("1234567890").unwrap(),
            Country::new("US").unwrap(),
        );

        let result = client_balance_service.create_client(&req).await.unwrap();
        let client_id = result.id();
        let req_transaction_1 =
            CreditTransactionRequest::new(client_id.clone(), Decimal::from(100)).unwrap();
        let req_transaction_2 =
            DebitTransactionRequest::new(client_id.clone(), Decimal::from(-33)).unwrap();
        let req_get = GetClientRequest::new(client_id.clone());

        // WHEN
        let result_transaction_1 = client_balance_service
            .credit_balance(&req_transaction_1)
            .await
            .unwrap();
        let result_transaction_2 = client_balance_service
            .debit_balance(&req_transaction_2)
            .await
            .unwrap();
        let result_get = client_balance_service
            .get_balance_by_client_id(&req_get)
            .await
            .unwrap();

        // ASSERT
        assert_eq!(result_transaction_1.balance(), &Decimal::from(100));
        assert_eq!(result_transaction_1.client_id(), client_id);
        assert_eq!(result_transaction_2.balance(), &Decimal::from(67));
        assert_eq!(result_transaction_2.client_id(), client_id);
        assert_eq!(result_get.balance(), &Decimal::from(67));
        assert_eq!(result_get.client_id(), client_id);
    }
}
