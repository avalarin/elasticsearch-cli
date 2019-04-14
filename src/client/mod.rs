mod fetcher;
pub mod elastic;
pub mod kibana;

pub use self::fetcher::*;

pub struct SearchRequest {
    pub index: String,
    pub query: String
}

pub trait Client {
    fn execute(&self, request: &SearchRequest) -> Result<Collector<serde_json::Value>, ClientError>;
}

#[derive(Debug, Fail)]
pub enum ClientError {
    #[fail(display = "{}", inner)]
    RequestError { inner: String }
}

impl From<FetcherError> for ClientError {
    fn from(err: FetcherError) -> Self {
        match err {
            FetcherError::RequestError { inner } => ClientError::RequestError { inner: inner.clone() }
        }
    }
}