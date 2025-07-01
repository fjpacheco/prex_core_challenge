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
        match result {
            Ok(_) => Err(ClientError::Duplicate {
                document: document.to_string(),
            }),
            Err(e) => match e {
                ClientError::NotFoundByDocument { .. } => Ok(()),
                _ => Err(e),
            },
        }
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
        sync::{
            Arc, Mutex,
            atomic::{AtomicUsize, Ordering},
        },
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

    type ClientsHashMap = Arc<Mutex<HashMap<ClientId, Client>>>;
    type ClientBalancesHashMap = Arc<Mutex<HashMap<ClientId, Balance>>>;

    fn setup_general_mocks(
        client_balance_repository: Option<(
            MockClientBalanceRepository,
            ClientsHashMap,
            ClientBalancesHashMap,
        )>,
        balance_exporter: Option<MockBalanceExporter>,
    ) -> (MockClientBalanceRepository, MockBalanceExporter) {
        let mut balance_exporter = balance_exporter.unwrap_or_default();

        let (mut client_balance_repository, arc_mutex_clients, arc_mutex_client_balances) =
            client_balance_repository.unwrap_or_default();
        let (arc_mutex_clients_1, arc_mutex_client_balances_1) =
            (arc_mutex_clients.clone(), arc_mutex_client_balances.clone());
        let id_counter = AtomicUsize::new(0);
        client_balance_repository
            .expect_create_client()
            .returning(move |req| {
                let client_id =
                    ClientId::new(&id_counter.fetch_add(1, Ordering::Relaxed).to_string()).unwrap();
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
                arc_mutex_client_balances_1.lock().unwrap().insert(
                    client_id.clone(),
                    Balance::new(client_id.clone(), Decimal::from(0)),
                );
                Box::pin(async move { Ok(client) })
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

        client_balance_repository
            .expect_are_balances_empty()
            .returning(move || Box::pin(async move { Ok(false) }));

        let arc_mutex_clients_4 = arc_mutex_clients.clone();
        client_balance_repository
            .expect_get_client()
            .returning(move |req| {
                let client_id_clone = req.client_id().clone();
                let result = arc_mutex_clients_4
                    .lock()
                    .unwrap()
                    .get(req.client_id())
                    .cloned();
                if let Some(client) = result {
                    Box::pin(async move { Ok(client) })
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
            .expect_reset_all_balances_to_zero()
            .returning(move || {
                let mut map = arc_mutex_client_balances_4.lock().unwrap();
                let mut old_balances = Vec::new();
                map.iter_mut().for_each(|(_, balance)| {
                    let old_balance = balance.set_balance(Decimal::ZERO);
                    old_balances.push(Balance::new(balance.client_id().clone(), old_balance));
                });
                Box::pin(async move { Ok(old_balances) })
            });

        balance_exporter
            .expect_export_balances()
            .returning(move |_| Box::pin(async move { Ok(()) }));

        let arc_mutex_client_balances_6 = arc_mutex_client_balances.clone();
        client_balance_repository
            .expect_merge_old_balances()
            .returning(move |old_balances| {
                let mut map = arc_mutex_client_balances_6.lock().unwrap();
                old_balances.iter().for_each(|old_balance| {
                    let actual_balance = map.get_mut(old_balance.client_id()).unwrap();
                    let new_balance_recorded = actual_balance.balance() + old_balance.balance();
                    actual_balance.set_balance(new_balance_recorded);
                });
                Box::pin(async move { Ok(()) })
            });

        (client_balance_repository, balance_exporter)
    }

    #[tokio::test]
    async fn test_01_given_a_client_when_creating_it_then_it_should_return_the_client_id_created() {
        // SETUP
        let (client_balance_repository, balance_exporter) = setup_general_mocks(None, None);
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
        let (client_balance_repository, balance_exporter) = setup_general_mocks(None, None);
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
        let (client_balance_repository, balance_exporter) = setup_general_mocks(None, None);
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
    async fn test_04_given_a_client_created_when_getting_client_then_it_should_return_the_client_info()
     {
        // SETUP
        let (client_balance_repository, balance_exporter) = setup_general_mocks(None, None);
        let client_balance_service = Service::new(client_balance_repository, balance_exporter);

        // GIVEN
        let client_name = "John Doe";
        let birth_date = "1990-01-01";
        let document = "1234567890";
        let country = "US";
        let req_create = CreateClientRequest::new(
            ClientName::new(client_name).unwrap(),
            BirthDate::new(birth_date).unwrap(),
            Document::new(document).unwrap(),
            Country::new(country).unwrap(),
        );
        let result_create = client_balance_service.create_client(&req_create).await;
        let client_id = result_create.unwrap().id().clone();

        // WHEN
        let req_get = GetClientRequest::new(client_id.clone());
        let result_get = client_balance_service.get_client_by_id(&req_get).await;

        // ASSERT
        assert!(result_get.is_ok());
        let client = result_get.unwrap();
        assert_eq!(client.name(), &ClientName::new(client_name).unwrap());
        assert_eq!(client.birth_date(), &BirthDate::new(birth_date).unwrap());
        assert_eq!(client.document(), &Document::new(document).unwrap());
        assert_eq!(client.country(), &Country::new(country).unwrap());
    }

    #[tokio::test]
    async fn test_05_given_error_in_repository_on_create_client_when_creating_client_then_should_return_error()
     {
        // SETUP
        let mut client_balance_repository = MockClientBalanceRepository::default();
        client_balance_repository
            .expect_get_client_by_document()
            .returning(|_| {
                Box::pin(async {
                    Err(ClientError::NotFoundByDocument {
                        document: Document::new("1234567890").unwrap(),
                    })
                })
            });
        client_balance_repository
            .expect_create_client()
            .returning(|_| {
                Box::pin(async { Err(ClientError::Unknown(anyhow::anyhow!("repo fail"))) })
            });
        let (client_balance_repository, balance_exporter) = setup_general_mocks(
            Some((
                client_balance_repository,
                Arc::new(Mutex::new(HashMap::new())),
                Arc::new(Mutex::new(HashMap::new())),
            )),
            None,
        );
        let client_balance_service = Service::new(client_balance_repository, balance_exporter);

        // GIVEN
        let req_create = CreateClientRequest::new(
            ClientName::new("John Doe").unwrap(),
            BirthDate::new("1990-01-01").unwrap(),
            Document::new("1234567890").unwrap(),
            Country::new("US").unwrap(),
        );
        // WHEN
        let result = client_balance_service.create_client(&req_create).await;

        // THEN
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            ClientError::Unknown(anyhow::anyhow!("repo fail"))
        );
    }

    #[tokio::test]
    async fn test_06_given_error_in_repository_on_get_client_by_document_when_creating_client_then_should_return_error()
     {
        // SETUP
        let mut client_balance_repository = MockClientBalanceRepository::default();
        client_balance_repository
            .expect_get_client_by_document()
            .returning(|_| {
                Box::pin(async { Err(ClientError::Unknown(anyhow::anyhow!("ka boom!"))) })
            });
        let (client_balance_repository, balance_exporter) = setup_general_mocks(
            Some((
                client_balance_repository,
                Arc::new(Mutex::new(HashMap::new())),
                Arc::new(Mutex::new(HashMap::new())),
            )),
            None,
        );
        let client_balance_service = Service::new(client_balance_repository, balance_exporter);

        // GIVEN
        let req_create = CreateClientRequest::new(
            ClientName::new("John Doe").unwrap(),
            BirthDate::new("1990-01-01").unwrap(),
            Document::new("1234567890").unwrap(),
            Country::new("US").unwrap(),
        );
        // WHEN
        let result = client_balance_service.create_client(&req_create).await;

        // THEN
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            ClientError::Unknown(anyhow::anyhow!("ka boom!"))
        );
    }

    #[tokio::test]
    async fn test_07_given_a_client_created_when_credit_and_debit_balance_then_it_should_be_updated_with_the_new_balance()
     {
        // SETUP
        let (client_balance_repository, balance_exporter) = setup_general_mocks(None, None);
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

    #[tokio::test]
    async fn test_08_given_nonexistent_client_when_credit_balance_then_should_return_not_found() {
        // SETUP
        let (client_balance_repository, balance_exporter) = setup_general_mocks(None, None);
        let client_balance_service = Service::new(client_balance_repository, balance_exporter);

        // GIVEN
        let client_id = ClientId::new("1").unwrap();
        let req = CreditTransactionRequest::new(client_id.clone(), Decimal::from(100)).unwrap();

        // WHEN
        let result = client_balance_service.credit_balance(&req).await;

        // ASSERT
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            ClientError::NotFoundById {
                id_document: client_id.clone()
            }
        );
    }

    #[tokio::test]
    async fn test_09_given_error_in_repository_when_credit_balance_then_should_return_error() {
        // SETUP
        let mut client_balance_repository = MockClientBalanceRepository::default();
        client_balance_repository
            .expect_client_id_exists()
            .returning(|_| Box::pin(async { Ok(true) }));
        client_balance_repository
            .expect_credit_balance()
            .returning(|_| {
                Box::pin(async { Err(ClientError::Unknown(anyhow::anyhow!("ka boom!"))) })
            });
        let (client_balance_repository, balance_exporter) = setup_general_mocks(
            Some((
                client_balance_repository,
                Arc::new(Mutex::new(HashMap::new())),
                Arc::new(Mutex::new(HashMap::new())),
            )),
            None,
        );
        let client_balance_service = Service::new(client_balance_repository, balance_exporter);

        // GIVEN
        let client_id = ClientId::new("1").unwrap();
        let req = CreditTransactionRequest::new(client_id.clone(), Decimal::from(100)).unwrap();

        // WHEN
        let result = client_balance_service.credit_balance(&req).await;

        // ASSERT
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            ClientError::Unknown(anyhow::anyhow!("ka boom!"))
        );
    }

    #[tokio::test]
    async fn test_10_given_nonexistent_client_when_debit_balance_then_should_return_not_found() {
        // SETUP
        let (client_balance_repository, balance_exporter) = setup_general_mocks(None, None);
        let client_balance_service = Service::new(client_balance_repository, balance_exporter);

        // GIVEN
        let client_id = ClientId::new("1").unwrap();
        let req = DebitTransactionRequest::new(client_id.clone(), Decimal::from(-100)).unwrap();

        // WHEN
        let result = client_balance_service.debit_balance(&req).await;

        // ASSERT
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            ClientError::NotFoundById {
                id_document: client_id.clone()
            }
        );
    }

    #[tokio::test]
    async fn test_11_given_error_in_repository_when_debit_balance_then_should_return_error() {
        // SETUP
        let mut client_balance_repository = MockClientBalanceRepository::default();
        client_balance_repository
            .expect_client_id_exists()
            .returning(|_| Box::pin(async { Ok(true) }));
        client_balance_repository
            .expect_debit_balance()
            .returning(|_| {
                Box::pin(async { Err(ClientError::Unknown(anyhow::anyhow!("ka boom!"))) })
            });
        let (client_balance_repository, balance_exporter) = setup_general_mocks(
            Some((
                client_balance_repository,
                Arc::new(Mutex::new(HashMap::new())),
                Arc::new(Mutex::new(HashMap::new())),
            )),
            None,
        );
        let client_balance_service = Service::new(client_balance_repository, balance_exporter);

        // GIVEN
        let client_id = ClientId::new("1").unwrap();
        let req = DebitTransactionRequest::new(client_id.clone(), Decimal::from(-100)).unwrap();

        // WHEN
        let result = client_balance_service.debit_balance(&req).await;

        // ASSERT
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            ClientError::Unknown(anyhow::anyhow!("ka boom!"))
        );
    }

    #[tokio::test]
    async fn test_12_given_nonexistent_client_when_get_balance_then_should_return_not_found() {
        // SETUP
        let (client_balance_repository, balance_exporter) = setup_general_mocks(None, None);
        let client_balance_service = Service::new(client_balance_repository, balance_exporter);

        // GIVEN
        let client_id = ClientId::new("1").unwrap();
        let req = GetClientRequest::new(client_id.clone());

        // WHEN
        let result = client_balance_service.get_balance_by_client_id(&req).await;

        // ASSERT
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            ClientError::NotFoundById {
                id_document: client_id.clone()
            }
        );
    }

    #[tokio::test]
    async fn test_13_given_error_in_repository_when_get_balance_then_should_return_error() {
        // SETUP
        let mut client_balance_repository = MockClientBalanceRepository::default();
        client_balance_repository
            .expect_client_id_exists()
            .returning(|_| Box::pin(async { Ok(true) }));
        client_balance_repository
            .expect_get_balance_by_client_id()
            .returning(|_| {
                Box::pin(async { Err(ClientError::Unknown(anyhow::anyhow!("ka boom!"))) })
            });
        let (client_balance_repository, balance_exporter) = setup_general_mocks(
            Some((
                client_balance_repository,
                Arc::new(Mutex::new(HashMap::new())),
                Arc::new(Mutex::new(HashMap::new())),
            )),
            None,
        );
        let client_balance_service = Service::new(client_balance_repository, balance_exporter);

        // GIVEN
        let client_id = ClientId::new("1").unwrap();
        let req = GetClientRequest::new(client_id.clone());

        // WHEN
        let result = client_balance_service.get_balance_by_client_id(&req).await;

        // ASSERT
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            ClientError::Unknown(anyhow::anyhow!("ka boom!"))
        );
    }

    #[tokio::test]
    async fn test_14_given_nonexistent_client_when_get_client_then_should_return_not_found() {
        // SETUP
        let (client_balance_repository, balance_exporter) = setup_general_mocks(None, None);
        let client_balance_service = Service::new(client_balance_repository, balance_exporter);

        // GIVEN
        let client_id = ClientId::new("1").unwrap();
        let req = GetClientRequest::new(client_id.clone());

        // WHEN
        let result = client_balance_service.get_client_by_id(&req).await;

        // ASSERT
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            ClientError::NotFoundById {
                id_document: client_id.clone()
            }
        );
    }

    #[tokio::test]
    async fn test_15_given_error_in_repository_when_get_client_then_should_return_error() {
        // SETUP
        let mut client_balance_repository = MockClientBalanceRepository::default();
        client_balance_repository
            .expect_client_id_exists()
            .returning(|_| Box::pin(async { Ok(true) }));
        client_balance_repository
            .expect_get_client()
            .returning(|_| {
                Box::pin(async { Err(ClientError::Unknown(anyhow::anyhow!("kaaa boomo!!"))) })
            });
        let (client_balance_repository, balance_exporter) = setup_general_mocks(
            Some((
                client_balance_repository,
                Arc::new(Mutex::new(HashMap::new())),
                Arc::new(Mutex::new(HashMap::new())),
            )),
            None,
        );
        let client_balance_service = Service::new(client_balance_repository, balance_exporter);

        // GIVEN
        let client_id = ClientId::new("1").unwrap();
        let req = GetClientRequest::new(client_id.clone());

        // WHEN
        let result = client_balance_service.get_client_by_id(&req).await;

        // ASSERT
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            ClientError::Unknown(anyhow::anyhow!("kaaa boomo!!"))
        );
    }

    #[tokio::test]
    async fn test_16_given_one_client_when_store_balances_then_balances_are_zero_and_exported() {
        // SETUP
        let (client_balance_repository, balance_exporter) = setup_general_mocks(None, None);
        let client_balance_service = Service::new(client_balance_repository, balance_exporter);

        // GIVEN
        let req_create = CreateClientRequest::new(
            ClientName::new("John Doe").unwrap(),
            BirthDate::new("1990-01-01").unwrap(),
            Document::new("1234567890").unwrap(),
            Country::new("US").unwrap(),
        );
        let client = client_balance_service
            .create_client(&req_create)
            .await
            .unwrap();
        let client_id = client.id().clone();
        let req_credit =
            CreditTransactionRequest::new(client_id.clone(), Decimal::from(100)).unwrap();
        client_balance_service
            .credit_balance(&req_credit)
            .await
            .unwrap();
        let req_get = GetClientRequest::new(client_id.clone());

        // WHEN
        let result_store = client_balance_service.store_balances().await;
        let result_get = client_balance_service
            .get_balance_by_client_id(&req_get)
            .await
            .unwrap();

        // ASSERT
        assert!(result_store.is_ok());
        assert_eq!(result_get.balance(), &Decimal::ZERO);
    }

    #[tokio::test]
    async fn test_17_given_multiple_clients_when_store_balances_then_all_balances_are_zero_and_exported()
     {
        // SETUP
        let (client_balance_repository, balance_exporter) = setup_general_mocks(None, None);
        let client_balance_service = Service::new(client_balance_repository, balance_exporter);

        // GIVEN: crear dos clientes usando el servicio
        let req_create_1 = CreateClientRequest::new(
            ClientName::new("John Doe").unwrap(),
            BirthDate::new("1990-01-01").unwrap(),
            Document::new("1234567890").unwrap(),
            Country::new("US").unwrap(),
        );
        let req_create_2 = CreateClientRequest::new(
            ClientName::new("Jane Roe").unwrap(),
            BirthDate::new("1992-02-02").unwrap(),
            Document::new("9876543210").unwrap(),
            Country::new("AR").unwrap(),
        );
        let client_1 = client_balance_service
            .create_client(&req_create_1)
            .await
            .unwrap();
        let client_2 = client_balance_service
            .create_client(&req_create_2)
            .await
            .unwrap();
        let client_id1 = client_1.id().clone();
        let client_id2 = client_2.id().clone();
        let req_credit =
            CreditTransactionRequest::new(client_id1.clone(), Decimal::from(100)).unwrap();
        let req_debit =
            DebitTransactionRequest::new(client_id2.clone(), Decimal::from(-50)).unwrap();
        client_balance_service
            .credit_balance(&req_credit)
            .await
            .unwrap();
        client_balance_service
            .debit_balance(&req_debit)
            .await
            .unwrap();

        // WHEN
        let result_store = client_balance_service.store_balances().await;
        let req_get_1 = GetClientRequest::new(client_id1.clone());
        let req_get_2 = GetClientRequest::new(client_id2.clone());
        let balance_1 = client_balance_service
            .get_balance_by_client_id(&req_get_1)
            .await
            .unwrap();
        let balance_2 = client_balance_service
            .get_balance_by_client_id(&req_get_2)
            .await
            .unwrap();

        // ASSERT
        assert!(result_store.is_ok());
        assert_eq!(balance_1.balance(), &Decimal::ZERO);
        assert_eq!(balance_2.balance(), &Decimal::ZERO);
    }

    #[tokio::test]
    async fn test_18_given_balances_negative_and_positive_when_store_balances_then_all_zero() {
        // SETUP
        let (client_balance_repository, balance_exporter) = setup_general_mocks(None, None);
        let client_balance_service = Service::new(client_balance_repository, balance_exporter);

        // GIVEN
        let req_create_1 = CreateClientRequest::new(
            ClientName::new("John Doe").unwrap(),
            BirthDate::new("1990-01-01").unwrap(),
            Document::new("1234567890").unwrap(),
            Country::new("US").unwrap(),
        );
        let req_create_2 = CreateClientRequest::new(
            ClientName::new("Jane Roe").unwrap(),
            BirthDate::new("1992-02-02").unwrap(),
            Document::new("9876543210").unwrap(),
            Country::new("AR").unwrap(),
        );
        let req_create_3 = CreateClientRequest::new(
            ClientName::new("Foo Bar").unwrap(),
            BirthDate::new("1980-03-03").unwrap(),
            Document::new("5555555555").unwrap(),
            Country::new("BR").unwrap(),
        );
        let client_1 = client_balance_service
            .create_client(&req_create_1)
            .await
            .unwrap();
        let client_2 = client_balance_service
            .create_client(&req_create_2)
            .await
            .unwrap();
        let client_3 = client_balance_service
            .create_client(&req_create_3)
            .await
            .unwrap();
        let client_id1 = client_1.id().clone();
        let client_id2 = client_2.id().clone();
        let client_id3 = client_3.id().clone();
        let req_credit =
            CreditTransactionRequest::new(client_id1.clone(), Decimal::from(100)).unwrap();
        let req_debit =
            DebitTransactionRequest::new(client_id2.clone(), Decimal::from(-50)).unwrap();
        let req_credit_3 =
            CreditTransactionRequest::new(client_id3.clone(), Decimal::from(200)).unwrap();
        client_balance_service
            .credit_balance(&req_credit)
            .await
            .unwrap();
        client_balance_service
            .debit_balance(&req_debit)
            .await
            .unwrap();
        client_balance_service
            .credit_balance(&req_credit_3)
            .await
            .unwrap();

        // WHEN
        let result_store = client_balance_service.store_balances().await;
        let req_get_1 = GetClientRequest::new(client_id1.clone());
        let req_get_2 = GetClientRequest::new(client_id2.clone());
        let req_get_3 = GetClientRequest::new(client_id3.clone());
        let balance_1 = client_balance_service
            .get_balance_by_client_id(&req_get_1)
            .await
            .unwrap();
        let balance_2 = client_balance_service
            .get_balance_by_client_id(&req_get_2)
            .await
            .unwrap();
        let balance_3 = client_balance_service
            .get_balance_by_client_id(&req_get_3)
            .await
            .unwrap();

        // ASSERT
        assert!(result_store.is_ok());
        assert_eq!(balance_1.balance(), &Decimal::ZERO);
        assert_eq!(balance_2.balance(), &Decimal::ZERO);
        assert_eq!(balance_3.balance(), &Decimal::ZERO);
    }

    #[tokio::test]
    async fn test_19_given_balances_already_zero_when_store_balances_then_exporter_receives_zero() {
        // SETUP
        let (client_balance_repository, balance_exporter) = setup_general_mocks(None, None);
        let client_balance_service = Service::new(client_balance_repository, balance_exporter);

        // GIVEN
        let req_create = CreateClientRequest::new(
            ClientName::new("John Doe").unwrap(),
            BirthDate::new("1990-01-01").unwrap(),
            Document::new("1234567890").unwrap(),
            Country::new("US").unwrap(),
        );
        let client = client_balance_service
            .create_client(&req_create)
            .await
            .unwrap();
        let client_id = client.id().clone();

        // WHEN
        let result_store = client_balance_service.store_balances().await;
        let req_get = GetClientRequest::new(client_id.clone());
        let balance = client_balance_service
            .get_balance_by_client_id(&req_get)
            .await
            .unwrap();

        // ASSERT
        assert!(result_store.is_ok());
        assert_eq!(balance.balance(), &Decimal::ZERO);
    }

    #[tokio::test]
    async fn test_20_given_no_balances_when_store_balances_then_should_return_balances_empty() {
        // SETUP
        let mut client_balance_repository = MockClientBalanceRepository::default();
        client_balance_repository
            .expect_are_balances_empty()
            .returning(|| Box::pin(async { Ok(true) }));
        let (client_balance_repository, balance_exporter) = setup_general_mocks(
            Some((
                client_balance_repository,
                Arc::new(Mutex::new(HashMap::new())),
                Arc::new(Mutex::new(HashMap::new())),
            )),
            None,
        );
        let client_balance_service = Service::new(client_balance_repository, balance_exporter);

        // WHEN
        let result = client_balance_service.store_balances().await;

        // ASSERT
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), ClientError::BalancesEmpty);
    }

    #[tokio::test]
    async fn test_21_given_error_on_reset_all_balances_to_zero_when_store_balances_then_return_error_and_balances_remain_unchanged()
     {
        // SETUP
        let mut client_balance_repository = MockClientBalanceRepository::default();
        client_balance_repository
            .expect_reset_all_balances_to_zero()
            .returning(|| {
                Box::pin(async { Err(ClientError::Unknown(anyhow::anyhow!("ka boom!"))) })
            });
        let (client_balance_repository, balance_exporter) = setup_general_mocks(
            Some((
                client_balance_repository,
                Arc::new(Mutex::new(HashMap::new())),
                Arc::new(Mutex::new(HashMap::new())),
            )),
            None,
        );
        let client_balance_service = Service::new(client_balance_repository, balance_exporter);

        // GIVEN
        let req_create_1 = CreateClientRequest::new(
            ClientName::new("John Doe").unwrap(),
            BirthDate::new("1990-01-01").unwrap(),
            Document::new("1234567890").unwrap(),
            Country::new("US").unwrap(),
        );
        let req_create_2 = CreateClientRequest::new(
            ClientName::new("Jane Roe").unwrap(),
            BirthDate::new("1992-02-02").unwrap(),
            Document::new("9876543210").unwrap(),
            Country::new("AR").unwrap(),
        );
        let client_1 = client_balance_service
            .create_client(&req_create_1)
            .await
            .unwrap();
        let client_2 = client_balance_service
            .create_client(&req_create_2)
            .await
            .unwrap();
        let client_id1 = client_1.id().clone();
        let client_id2 = client_2.id().clone();
        let decimal_1_expected = Decimal::from(100);
        let decimal_2_expected = Decimal::from(-50);
        let req_credit =
            CreditTransactionRequest::new(client_id1.clone(), decimal_1_expected).unwrap();
        let req_debit =
            DebitTransactionRequest::new(client_id2.clone(), decimal_2_expected).unwrap();
        client_balance_service
            .credit_balance(&req_credit)
            .await
            .unwrap();
        client_balance_service
            .debit_balance(&req_debit)
            .await
            .unwrap();
        let req_get_1 = GetClientRequest::new(client_id1.clone());
        let req_get_2 = GetClientRequest::new(client_id2.clone());

        // WHEN
        let result_store = client_balance_service.store_balances().await;
        let result_balance_1 = client_balance_service
            .get_balance_by_client_id(&req_get_1)
            .await;
        let result_balance_2 = client_balance_service
            .get_balance_by_client_id(&req_get_2)
            .await;

        // THEN
        assert!(result_store.is_err());
        assert_eq!(
            result_store.err().unwrap(),
            ClientError::Unknown(anyhow::anyhow!("ka boom!"))
        );
        assert!(result_balance_1.is_ok());
        assert!(result_balance_2.is_ok());
        let balance_1 = result_balance_1.unwrap();
        let balance_2 = result_balance_2.unwrap();
        assert_eq!(balance_1.balance(), &decimal_1_expected);
        assert_eq!(balance_2.balance(), &decimal_2_expected);
    }

    #[tokio::test]
    async fn test_22_given_error_on_export_balances_when_store_balances_then_return_error_and_balances_remain_unchanged()
     {
        // SETUP
        let mut balance_exporter = MockBalanceExporter::default();
        balance_exporter.expect_export_balances().returning(|_| {
            Box::pin(async { Err(ClientError::Unknown(anyhow::anyhow!("ka boom!"))) })
        });
        let (client_balance_repository, balance_exporter) =
            setup_general_mocks(None, Some(balance_exporter));
        let client_balance_service = Service::new(client_balance_repository, balance_exporter);

        // GIVEN
        let req_create_1 = CreateClientRequest::new(
            ClientName::new("John Doe").unwrap(),
            BirthDate::new("1990-01-01").unwrap(),
            Document::new("1234567890").unwrap(),
            Country::new("US").unwrap(),
        );
        let req_create_2 = CreateClientRequest::new(
            ClientName::new("Jane Roe").unwrap(),
            BirthDate::new("1992-02-02").unwrap(),
            Document::new("9876543210").unwrap(),
            Country::new("AR").unwrap(),
        );
        let client_1 = client_balance_service
            .create_client(&req_create_1)
            .await
            .unwrap();
        let client_2 = client_balance_service
            .create_client(&req_create_2)
            .await
            .unwrap();
        let client_id1 = client_1.id().clone();
        let client_id2 = client_2.id().clone();
        let decimal_1_expected = Decimal::from(100);
        let decimal_2_expected = Decimal::from(-50);
        let req_credit =
            CreditTransactionRequest::new(client_id1.clone(), Decimal::from(100)).unwrap();
        let req_debit =
            DebitTransactionRequest::new(client_id2.clone(), Decimal::from(-50)).unwrap();
        client_balance_service
            .credit_balance(&req_credit)
            .await
            .unwrap();
        client_balance_service
            .debit_balance(&req_debit)
            .await
            .unwrap();
        let req_get_1 = GetClientRequest::new(client_id1.clone());
        let req_get_2 = GetClientRequest::new(client_id2.clone());

        // WHEN
        let result_store = client_balance_service.store_balances().await;
        let result_balance_1 = client_balance_service
            .get_balance_by_client_id(&req_get_1)
            .await;
        let result_balance_2 = client_balance_service
            .get_balance_by_client_id(&req_get_2)
            .await;

        // THEN
        assert!(result_store.is_err());
        assert_eq!(
            result_store.err().unwrap(),
            ClientError::Unknown(anyhow::anyhow!("ka boom!"))
        );
        assert!(result_balance_1.is_ok());
        assert!(result_balance_2.is_ok());
        let balance_1 = result_balance_1.unwrap();
        let balance_2 = result_balance_2.unwrap();
        assert_eq!(balance_1.balance(), &decimal_1_expected);
        assert_eq!(balance_2.balance(), &decimal_2_expected);
    }

    #[tokio::test]
    async fn test_23_given_error_on_export_balances_and_merge_old_balances_when_store_balances_then_return_error_and_old_balances_are_lost()
     {
        // SETUP
        let mut client_balance_repository = MockClientBalanceRepository::default();
        let mut balance_exporter = MockBalanceExporter::default();
        balance_exporter.expect_export_balances().returning(|_| {
            Box::pin(async { Err(ClientError::Unknown(anyhow::anyhow!("ka boom!"))) })
        });
        client_balance_repository
            .expect_merge_old_balances()
            .returning(|_| {
                Box::pin(async { Err(ClientError::Unknown(anyhow::anyhow!("ka boom!"))) })
            });

        let (client_balance_repository, balance_exporter) = setup_general_mocks(
            Some((
                client_balance_repository,
                Arc::new(Mutex::new(HashMap::new())),
                Arc::new(Mutex::new(HashMap::new())),
            )),
            Some(balance_exporter),
        );
        let client_balance_service = Service::new(client_balance_repository, balance_exporter);

        // GIVEN
        let req_create_1 = CreateClientRequest::new(
            ClientName::new("John Doe").unwrap(),
            BirthDate::new("1990-01-01").unwrap(),
            Document::new("1234567890").unwrap(),
            Country::new("US").unwrap(),
        );
        let req_create_2 = CreateClientRequest::new(
            ClientName::new("Jane Roe").unwrap(),
            BirthDate::new("1992-02-02").unwrap(),
            Document::new("9876543210").unwrap(),
            Country::new("AR").unwrap(),
        );
        let client_1 = client_balance_service
            .create_client(&req_create_1)
            .await
            .unwrap();
        let client_2 = client_balance_service
            .create_client(&req_create_2)
            .await
            .unwrap();
        let client_id1 = client_1.id().clone();
        let client_id2 = client_2.id().clone();
        let decimal_1_expected = Decimal::from(100);
        let decimal_2_expected = Decimal::from(-50);
        let req_credit =
            CreditTransactionRequest::new(client_id1.clone(), decimal_1_expected).unwrap();
        let req_debit =
            DebitTransactionRequest::new(client_id2.clone(), decimal_2_expected).unwrap();
        client_balance_service
            .credit_balance(&req_credit)
            .await
            .unwrap();
        client_balance_service
            .debit_balance(&req_debit)
            .await
            .unwrap();
        let req_get_1 = GetClientRequest::new(client_id1.clone());
        let req_get_2 = GetClientRequest::new(client_id2.clone());

        // WHEN
        let result_store = client_balance_service.store_balances().await;
        let result_balance_1 = client_balance_service
            .get_balance_by_client_id(&req_get_1)
            .await;
        let result_balance_2 = client_balance_service
            .get_balance_by_client_id(&req_get_2)
            .await;

        // THEN
        assert!(result_store.is_err());
        assert_eq!(
            result_store.err().unwrap(),
            ClientError::Unknown(anyhow::anyhow!("ka boom!"))
        );
        assert!(result_balance_1.is_ok());
        assert!(result_balance_2.is_ok());
        let balance_1 = result_balance_1.unwrap();
        let balance_2 = result_balance_2.unwrap();
        assert_eq!(balance_1.balance(), &Decimal::ZERO);
        assert_eq!(balance_2.balance(), &Decimal::ZERO);
    }
}
