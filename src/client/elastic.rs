use super::{Client, SearchRequest, ClientError, Fetcher, FetcherError, Collector};

use config::{ElasticSearchServer, SecretsStorage, Credentials};
use serde_json::Value;
use elastic::prelude::*;
use elastic::http::header::{Authorization, Basic};
use elastic::client::SyncSender;

use std::iter::Iterator;
use std::sync::Arc;

pub struct ElasticClient {
    secrets: Arc<SecretsStorage>,
    server_config: ElasticSearchServer,
    buffer_size: usize
}

pub struct ElasticFetcher {
    client: elastic::client::Client<SyncSender>,
    index: String,
    query: String,
    buffer_size: usize
}

impl ElasticClient {
    pub fn create(
        secrets: Arc<SecretsStorage>,
        server_config: ElasticSearchServer,
        buffer_size: usize
    ) -> Self {
        ElasticClient { secrets, server_config, buffer_size }
    }
}

impl Client for ElasticClient {
    fn execute(&self, request: &SearchRequest) -> Result<Collector<Value>, ClientError> {
        let credentials = self.server_config.username.as_ref()
            .map(|username| {
                self.secrets.get_credentials(&username).map_err(|err| {
                    error!("Cannot read credentials: {}", err);
                    ClientError::RequestError { inner: format!("cannot read credentials: {}", err) }
                })
            })
            .unwrap_or_else(|| Ok(None))?;

        // TODO Bearer Token auth
        let _token = credentials.clone().map(|Credentials{ username, password }| {
            Some(base64::encode(&format!("{}:{}", username, password)))
        });

        let mut builder = SyncClientBuilder::new();
        if let Some(Credentials{ username, password }) = credentials {
            builder = builder.params(
                |p| {
                    p.header(Authorization(Basic {
                        username: username.clone(),
                        password: Some(password.clone()),
                    }))
                }
            );
        }

        let client = builder.base_url(self.server_config.server.clone())
            .build()
            .map_err(|err| {
                error!("Cannot create elasticsearch client: {}", err);
                ClientError::RequestError { inner: format!("{}", err) }
            })?;

        let fetcher = ElasticFetcher::create(client, request, self.buffer_size);

        Collector::create(fetcher)
            .map_err(From::from)
    }
}

impl Fetcher<Value> for ElasticFetcher {
    fn fetch_next(&self, from: usize) -> Result<(usize, Vec<Value>), FetcherError> {
        self.client.search::<serde_json::Value>()
            .index(self.index.clone())
            .body(json!({
                "size": self.buffer_size,
                "from": from,
                "query": {
                    "query_string" : {
                        "query" : self.query
                    }
                }
            }))
            .send()
            .map(|resp| (resp.total() as usize, resp.documents().cloned().collect()))
            .map_err(|err| {
                error!("Cannot read response from elasticsearch: {}", err);
                FetcherError::RequestError { inner: format!("cannot read response from elasticsearch:{}", err) }
            })
    }
}

impl ElasticFetcher {
    pub fn create(
        client: elastic::client::Client<SyncSender>,
        request: &SearchRequest,
        buffer_size: usize
    ) -> ElasticFetcher {
        ElasticFetcher {
            client,
            query: request.query.clone(),
            index: request.index.clone(),
            buffer_size
        }
    }
}