use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use rust_decimal::Decimal;
use tokio::sync::Mutex;

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
    port::outbound::client_balance_repository::ClientBalanceRepository,
};

pub struct InMemoryRepository {
    clients: Arc<Mutex<HashMap<ClientId, (Client, Decimal)>>>,
    id_counter: AtomicUsize,
}

impl Default for InMemoryRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryRepository {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            id_counter: AtomicUsize::new(0),
        }
    }

    async fn update_balance(
        &self,
        client_id: &ClientId,
        amount: &Decimal,
    ) -> Result<Balance, ClientError> {
        let mut clients = self.clients.lock().await;
        let client_balance = clients
            .get_mut(client_id)
            .ok_or(ClientError::NotFoundById {
                id_document: client_id.clone(),
            })?;
        let new_decimal_balance = client_balance.1 + amount;
        client_balance.1 = new_decimal_balance;
        Ok(Balance::new(client_id.clone(), new_decimal_balance))
    }
}

impl ClientBalanceRepository for InMemoryRepository {
    async fn create_client(&self, req: &CreateClientRequest) -> Result<Client, ClientError> {
        let id = ClientId::new(&self.id_counter.fetch_add(1, Ordering::Relaxed).to_string())?;
        let client = Client::new(
            id.clone(),
            req.name().clone(),
            req.birth_date().clone(),
            req.document().clone(),
            req.country().clone(),
        );
        let mut clients = self.clients.lock().await;
        if clients
            .iter()
            .any(|(_, (client, _))| client.document() == req.document())
        {
            return Err(ClientError::Duplicate {
                document: req.document().to_string(),
            });
        }
        clients.insert(id, (client.clone(), Decimal::from(0)));
        Ok(client)
    }

    async fn client_id_exists(&self, client_id: &ClientId) -> Result<bool, ClientError> {
        let clients = self.clients.lock().await;
        Ok(clients.contains_key(client_id))
    }

    async fn get_client_by_document(&self, document: &Document) -> Result<Client, ClientError> {
        let clients = self.clients.lock().await;
        let (_, (client, _)) = clients
            .iter()
            .find(|(_, (client, _))| client.document() == document)
            .ok_or(ClientError::NotFoundByDocument {
                document: document.clone(),
            })?;
        Ok(client.clone())
    }

    async fn credit_balance(&self, req: &CreditTransactionRequest) -> Result<Balance, ClientError> {
        self.update_balance(req.client_id(), req.amount()).await
    }

    async fn get_client(&self, req: &GetClientRequest) -> Result<Client, ClientError> {
        let clients = self.clients.lock().await;
        let (client, _) = clients
            .get(req.client_id())
            .ok_or(ClientError::NotFoundById {
                id_document: req.client_id().clone(),
            })?;
        Ok(client.clone())
    }

    async fn debit_balance(&self, req: &DebitTransactionRequest) -> Result<Balance, ClientError> {
        self.update_balance(req.client_id(), req.amount()).await
    }

    async fn get_balance_by_client_id(
        &self,
        req: &GetClientRequest,
    ) -> Result<Balance, ClientError> {
        let client_balances = self.clients.lock().await;
        let (client, balance) =
            client_balances
                .get(req.client_id())
                .ok_or(ClientError::NotFoundById {
                    id_document: req.client_id().clone(),
                })?;
        Ok(Balance::new(client.id().clone(), *balance))
    }

    async fn reset_all_balances_to_zero(&self) -> Result<Vec<Balance>, ClientError> {
        let mut clients = self.clients.lock().await;
        let old_balances = clients
            .values_mut()
            .map(|(client, balance)| {
                let old_balance = *balance;
                *balance = Decimal::from(0);
                Balance::new(client.id().clone(), old_balance)
            })
            .collect();
        Ok(old_balances)
    }

    async fn are_balances_empty(&self) -> Result<bool, ClientError> {
        let clients = self.clients.lock().await;
        Ok(clients.is_empty())
    }

    async fn merge_old_balances(
        &self,
        old_client_balances: Vec<Balance>,
    ) -> Result<(), ClientError> {
        let mut clients = self.clients.lock().await;
        old_client_balances.iter().for_each(|old_client_balance| {
            let old_balance = old_client_balance.balance();
            if let Some((_, balance)) = clients.get_mut(old_client_balance.client_id()) {
                let new_balance = *old_balance + *balance;
                *balance = new_balance;
            } else {
                tracing::warn!(
                    "client not found by id {} and balance of this client will be ignored...",
                    old_client_balance.client_id()
                );
            }
        });
        Ok(())
    }
}
