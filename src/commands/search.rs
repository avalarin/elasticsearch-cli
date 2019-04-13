use crate::config::{ApplicationConfig, GetServerError};
use crate::commands::Command;
use crate::client::{Client, elastic::ElasticClient, SearchRequest};
use crate::display::*;

use clap::ArgMatches;

use std::iter::{Iterator};
use error::ApplicationError;

pub struct SearchCommand {
    pub client: Box<Client>,
    pub renderer: Box<Renderer>,
    pub request: SearchRequest
}

impl Command for SearchCommand {
    fn execute(&mut self) -> Result<(), ApplicationError> {
        info!("Executing search '{}' on index '{}'", self.request.query, self.request.index);

        let _ = self.client.execute(&self.request).map_err(|err| {
            error!("Cannot fetch items from server: {}", err)
        }).map(|collector| {
            collector.enumerate()
                .for_each(|(index, item)| {
                    self.renderer.render(&mut std::io::stdout(), &item, index);
                });
        });

        Ok(())
    }
}

impl SearchCommand {
    pub fn parse(config: &ApplicationConfig, matches: &ArgMatches, sub_match: &ArgMatches) -> Result<Self, ApplicationError> {
        let server = match config.get_server(matches.value_of("server")) {
            Ok(server) => Ok(server),
            Err(GetServerError::ServerNotFound(name)) => {
                error!("Server with name '{}' not found", name);
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
            .map(|s| s.to_string())
            .ok_or_else(|| {
                error!("Query must be specified");
                ApplicationError
            })?;

        let index = sub_match.value_of("index")
            .map(|s| s.to_string())
            .or_else(|| server.default_index.clone())
            .unwrap_or("*".to_string());

        let format = sub_match.value_of("output")
            .map(|f| match f {
                "pretty" => OutputFormat::Pretty,
                "json" => OutputFormat::JSON,
                custom => OutputFormat::Custom(custom.to_string())
            }).unwrap_or(OutputFormat::Pretty);

        let client = Box::new(ElasticClient::create(server.clone(), buffer_size));

        let extractor = sub_match.value_of("fields")
            .map(|s| JSONExtractor::filtered(s.split(',')))
            .unwrap_or_else(|| JSONExtractor::default());

        let renderer = Box::new(Renderer::create(format, extractor));

        Ok(SearchCommand {
            client,
            request: SearchRequest { query, index },
            renderer
        })
    }
}