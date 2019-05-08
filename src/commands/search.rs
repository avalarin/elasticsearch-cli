use crate::config::{ApplicationConfig, ElasticSearchServer, ElasticSearchServerType, GetServerError};
use crate::commands::Command;
use crate::client::{Client, elastic::ElasticClient, kibana::KibanaProxyClient, stub::StubClient, SearchRequest};
use crate::display::*;

use clap::ArgMatches;

use std::string::ToString;
use std::sync::Arc;
use error::ApplicationError;
use config::SecretsReader;

pub struct SearchCommand {
    pub client: Box<Client>,
    pub renderer: Box<Renderer>,
    pub request: SearchRequest,
    pub secrets: Arc<SecretsReader>
}

impl Command for SearchCommand {
    fn execute(&mut self) -> Result<(), ApplicationError> {
        info!("Executing search '{}' on index '{}'", self.request.query, self.request.index);

        let _ = self.client.execute(&self.request).map_err(|err| {
            error!("Cannot fetch items from server: {}", err)
        }).map(|collector| {
            self.renderer.render(collector);
        });

        Ok(())
    }
}

impl SearchCommand {
    pub fn parse(config: &ApplicationConfig, secrets: Arc<SecretsReader>, matches: &ArgMatches, sub_match: &ArgMatches) -> Result<Self, ApplicationError> {
        let server = match config.get_server(matches.value_of("server")) {
            Ok(server) => Ok(server),
            Err(GetServerError::ServerNotFound { server }) => {
                error!("Server with name '{}' not found", server);
                Err(ApplicationError)
            }
            Err(GetServerError::ServerNotSpecified) => {
                error!("The server is not specified.");
                error!("Hint: use 'elastic-cli config use server <name>'");
                error!("Hint: use option --server, e.g. 'elastic-cli --server <name> search ...'");
                Err(ApplicationError)
            }
            Err(GetServerError::NoConfiguredServers) => {
                error!("There are no servers in the config file");
                error!("Hint: use 'elastic-cli config add server <name> --address <address>'");
                Err(ApplicationError)
            }
        }?;

        let pager_enabled = sub_match.is_present("pager");

        let _size = sub_match.value_of("size").map(str::parse).unwrap_or(Ok(1000))
            .map_err(|err| {
                error!("Argument 'size' has invalid value: {}", err);
                ApplicationError
            })?;

        let buffer_size = sub_match.value_of("buffer").map(str::parse).unwrap_or(Ok(1000))
            .map_err(|err| {
                error!("Argument 'buffer' has invalid value: {}", err);
                ApplicationError
            })?;

        let query = sub_match.value_of("query")
            .map(ToString::to_string)
            .ok_or_else(|| {
                error!("Query must be specified");
                ApplicationError
            })?;

        let index = sub_match.value_of("index")
            .map(ToString::to_string)
            .or_else(|| server.default_index.clone())
            .unwrap_or_else(|| "*".to_string());

        let format = sub_match.value_of("output")
            .map(|f| match f {
                "pretty" => OutputFormat::Pretty,
                "json" => OutputFormat::JSON,
                custom => OutputFormat::Custom(custom.to_string())
            }).unwrap_or(OutputFormat::Pretty);

        let extractor = sub_match.value_of("fields")
            .map(|s| JSONExtractor::filtered(s.split(',')))
            .unwrap_or_else(JSONExtractor::default);

        let renderer = Self::create_renderer(pager_enabled, format, extractor);
        let client = Self::create_client(secrets.clone(), server, buffer_size);

        Ok(SearchCommand {
            client,
            request: SearchRequest { query, index },
            renderer,
            secrets
        })
    }

    fn create_renderer(
        pager_enabled: bool,
        format: OutputFormat,
        extractor: JSONExtractor
    ) -> Box<Renderer> {
        if pager_enabled {
            Box::new(PagedRenderer::new(format, extractor))
        } else {
            Box::new(SimpleRenderer::new(format, extractor))
        }
    }

    fn create_client(
        secrets: Arc<SecretsReader>,
        server: &ElasticSearchServer,
        buffer_size: usize
    ) -> Box<Client> {
        match server.server_type {
            ElasticSearchServerType::Elastic => Box::new(ElasticClient::create(secrets, server.clone(), buffer_size)),
            ElasticSearchServerType::Kibana => Box::new(KibanaProxyClient::create(secrets, server.clone(), buffer_size)),
            ElasticSearchServerType::Stub => Box::new(StubClient::new(buffer_size))
        }
    }
}