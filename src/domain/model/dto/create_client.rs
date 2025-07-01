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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::value::{
        birth_date::BirthDate, client_name::ClientName, country::Country, document::Document,
    };

    #[test]
    fn test_01_given_valid_data_when_creating_create_client_request_then_fields_should_be_accessible()
     {
        let name = ClientName::new("John Doe").unwrap();
        let birth_date = BirthDate::new("1990-01-01").unwrap();
        let document = Document::new("1234567890").unwrap();
        let country = Country::new("Argentina").unwrap();
        let req = CreateClientRequest::new(
            name.clone(),
            birth_date.clone(),
            document.clone(),
            country.clone(),
        );
        assert_eq!(req.name(), &name);
        assert_eq!(req.birth_date(), &birth_date);
        assert_eq!(req.document(), &document);
        assert_eq!(req.country(), &country);
    }
}
