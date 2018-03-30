use serde_json::{Value};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EsResponse {
    pub hits: EsResponseHits
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EsResponseHits {
    pub total: i32,
    pub hits: Vec<Value>
}