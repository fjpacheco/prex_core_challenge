use crate::domain::model::value::{
    birth_date::BirthDate, client_id::ClientId, client_name::ClientName, country::Country,
    document::Document,
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Client {
    id: ClientId,
    name: ClientName,
    birth_date: BirthDate,
    document: Document,
    country: Country,
}

impl Client {
    pub fn new(
        id: ClientId,
        name: ClientName,
        birth_date: BirthDate,
        document: Document,
        country: Country,
    ) -> Self {
        Self {
            id,
            name,
            birth_date,
            document,
            country,
        }
    }

    pub fn id(&self) -> &ClientId {
        &self.id
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
