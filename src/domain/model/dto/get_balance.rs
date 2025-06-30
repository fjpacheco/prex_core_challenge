use derive_more::From;

use crate::domain::model::value::client_id::ClientId;

#[allow(unused_imports)]
use crate::domain::model::entity::client::Client;

/// The fields required by the domain to get a [Client].
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, From)]
pub struct GetClientRequest {
    client_id: ClientId,
}

impl GetClientRequest {
    pub fn new(client_id: ClientId) -> Self {
        Self { client_id }
    }

    pub fn client_id(&self) -> &ClientId {
        &self.client_id
    }
}
