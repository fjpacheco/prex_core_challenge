use derive_more::From;

use crate::domain::model::value::{
    birth_date::BirthDate, client_name::ClientName, country::Country, document::Document,
};

#[allow(unused_imports)]
use crate::domain::model::entity::client::Client;

/// The fields required by the domain to create an [Client].
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, From)]
pub struct CreateClientRequest {
    name: ClientName,
    birth_date: BirthDate,
    document: Document,
    country: Country,
}

impl CreateClientRequest {
    pub fn new(
        name: ClientName,
        birth_date: BirthDate,
        document: Document,
        country: Country,
    ) -> Self {
        Self {
            name,
            birth_date,
            document,
            country,
        }
    }

    pub fn name(&self) -> &ClientName {
        &self.name
    }

    pub fn birth_date(&self) -> &BirthDate {
        &self.birth_date
    }

    pub fn document(&self) -> &Document {
        &self.document
    }

    pub fn country(&self) -> &Country {
        &self.country
    }
}
