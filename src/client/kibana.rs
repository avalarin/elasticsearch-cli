use super::{Client, SearchRequest, ClientError, Fetcher, FetcherError, Collector};

use config::ElasticSearchServer;
use serde_json::Value;
use elastic::prelude::SearchResponse;
use reqwest::Url;

pub struct KibanaProxyClient {
    server_config: ElasticSearchServer,
    buffer_size: usize
}

pub struct KibanaProxyFetcher {
    server_config: ElasticSearchServer,
    client: reqwest::Client,
    index: String,
    query: String,
    buffer_size: usize
}

impl KibanaProxyClient {
    pub fn create(
        server_config: ElasticSearchServer,
        buffer_size: usize
    ) -> Self {
        KibanaProxyClient {
            server_config,
            buffer_size
        }
    }
}

impl Client for KibanaProxyClient {
    fn execute(&self, request: &SearchRequest) -> Result<Collector<Value>, ClientError> {
        let client = reqwest::Client::new();
        let fetcher = KibanaProxyFetcher::create(self.server_config.clone(), client, request, self.buffer_size);

        Collector::create(fetcher)
            .map_err(From::from)
    }
}

impl KibanaProxyFetcher {
    pub fn create(
        server_config: ElasticSearchServer,
        client: reqwest::Client,
        request: &SearchRequest,
        buffer_size: usize
    ) -> KibanaProxyFetcher {
        KibanaProxyFetcher {
            server_config,
            client,
            query: request.query.clone(),
            index: request.index.clone(),
            buffer_size
        }
    }
}

impl Fetcher<Value> for KibanaProxyFetcher {
    fn fetch_next(&self, from: usize) -> Result<(usize, Vec<Value>), FetcherError> {
        let mut url = Url::parse_with_params(
            self.server_config.server.clone().as_ref(),
            vec![("method", "POST"), ("path", format!("{}/_search", self.index).as_ref())]
        )
        .map_err(|err| {
            error!("Invalid server address: {}", err);
            FetcherError::RequestError { inner: format!("invalid server address: {}", err) }
        })?;
        url.set_path("/api/console/proxy");

        let mut request = self.client.post(url);

        if let Some(username) = &self.server_config.username {
            request = request.basic_auth(username.to_owned(), self.server_config.password.clone());
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
