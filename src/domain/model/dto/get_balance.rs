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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::value::client_id::ClientId;

    #[test]
    fn test_01_given_a_client_id_when_creating_get_client_request_then_field_should_be_accessible()
    {
        let client_id = ClientId::default();
        let req = GetClientRequest::new(client_id.clone());
        assert_eq!(req.client_id(), &client_id);
    }
}
