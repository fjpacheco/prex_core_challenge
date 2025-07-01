use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct StoreBalancesHttpResponseBody {
    message: String,
}

impl StoreBalancesHttpResponseBody {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
    pub fn success_store() -> Self {
        Self::new("Successfully stored balances")
    }
}
