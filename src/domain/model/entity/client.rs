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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::value::{
        birth_date::BirthDate, client_id::ClientId, client_name::ClientName, country::Country,
        document::Document,
    };

    #[test]
    fn test_01_given_valid_data_when_creating_client_then_fields_should_be_accessible() {
        let id = ClientId::default();
        let name = ClientName::new("John Doe").unwrap();
        let birth_date = BirthDate::new("1990-01-01").unwrap();
        let document = Document::new("1234567890").unwrap();
        let country = Country::new("Argentina").unwrap();
        let client = Client::new(
            id.clone(),
            name.clone(),
            birth_date.clone(),
            document.clone(),
            country.clone(),
        );
        assert_eq!(client.id(), &id);
        assert_eq!(client.name(), &name);
        assert_eq!(client.birth_date(), &birth_date);
        assert_eq!(client.document(), &document);
        assert_eq!(client.country(), &country);
    }
}
