use super::{Client, SearchRequest, ClientError, Fetcher, FetcherError, Collector};

use config::{ElasticSearchServer, SecretsReader, Credentials};
use serde_json::Value;
use elastic::prelude::SearchResponse;
use reqwest::Url;
use std::sync::Arc;

pub struct KibanaProxyClient {
    secrets: Arc<dyn SecretsReader>,
    server_config: ElasticSearchServer,
    buffer_size: usize
}

pub struct KibanaProxyFetcher {
    url: Url,
    credentials: Option<Credentials>,
    client: reqwest::Client,
    query: String,
    buffer_size: usize
}

impl KibanaProxyClient {
    pub fn create(
        secrets: Arc<dyn SecretsReader>,
        server_config: ElasticSearchServer,
        buffer_size: usize
    ) -> Self {
        KibanaProxyClient {
            secrets,
            server_config,
            buffer_size
        }
    }
}

impl Client for KibanaProxyClient {
    fn execute(&self, request: &SearchRequest) -> Result<Collector<Value>, ClientError> {
        let client = reqwest::Client::new();

        let mut url = Url::parse_with_params(
            self.server_config.server.clone().as_ref(),
            vec![("method", "POST"), ("path", format!("{}/_search", request.index).as_ref())]
        ).map_err(|err| {
            error!("Invalid server address: {}", err);
            ClientError::RequestError { inner: format!("invalid server address: {}", err) }
        })?;
        url.set_path("/api/console/proxy");

        let credentials = self.server_config.username.as_ref()
            .map(|username| {
                self.secrets.get_credentials(&username).map_err(|err| {
                    error!("Cannot read credentials: {}", err);
                    ClientError::RequestError { inner: format!("cannot read credentials: {}", err) }
                })
            })
            .unwrap_or_else(|| Ok(None))?;


        let fetcher = KibanaProxyFetcher::create(url, credentials, client, request, self.buffer_size);

        Collector::create(fetcher)
            .map_err(From::from)
    }
}

impl KibanaProxyFetcher {
    pub fn create(
        url: Url,
        credentials: Option<Credentials>,
        client: reqwest::Client,
        request: &SearchRequest,
        buffer_size: usize
    ) -> KibanaProxyFetcher {
        KibanaProxyFetcher {
            url,
            credentials,
            client,
            query: request.query.clone(),
            buffer_size
        }
    }
}

impl Fetcher<Value> for KibanaProxyFetcher {
    fn fetch_next(&self, from: usize) -> Result<(usize, Vec<Value>), FetcherError> {
        let mut request = self.client.post(self.url.clone());

        if let Some(Credentials { username, password }) = &self.credentials {
            request = request.basic_auth(username.to_owned(), Some(password.to_owned()));
        }

        request
            .header("kbn-xsrf", "reporting")
            .body(json!({
                "size": self.buffer_size,
                "from": from,
                "query": {
                    "query_string" : {
                        "query" : self.query
                    }
                }
            }).to_string())
            .send()
            .map_err(|err| {
                error!("Cannot read response from kibana: {}", err);
                FetcherError::RequestError { inner: format!("cannot read response from kibana: {}", err) }
            })
            .and_then(|resp| {
                if resp.status().is_success() {
                    Ok(resp)
                } else {
                    error!("Kibana responded {}", resp.status());
                    Err(FetcherError::RequestError { inner: format!("kibana responded {}", resp.status()) })
                }
            })
            .and_then(|mut resp| {
                resp.json::<SearchResponse<Value>>()
                    .map_err(|err| {
                        error!("Cannot parse json response from kibana: {}", err);
                        FetcherError::RequestError { inner: format!("cannot parse json response from kibana: {}", err) }
                    })
            })
            .map(|resp| {
                (resp.total() as usize, resp.documents().cloned().collect())
            })

    }
}
