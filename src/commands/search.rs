use config::ElasticSearchServer;
use commands::{Command, CommandError};
use crate::client::{Client, elastic::ElasticClient, SearchRequest};
use crate::display::*;

use std::iter::{Iterator};

pub struct SearchCommand {
    pub client: Box<Client>,
    pub renderer: Box<Renderer>,
    pub request: SearchRequest
}

impl Command<CommandError> for SearchCommand {
    fn execute(&mut self) -> Result<(), CommandError> {
        info!("Executing search {} on index {}", self.request.query, self.request.index);

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
    pub fn new<S1, S2, S3>(
        server_config: &ElasticSearchServer,
        index: Option<S1>,
        query: impl Into<String>,
        fields: Option<Vec<S2>>,
        display_format: Option<S3>
    ) -> Self
        where S1: Into<String>, S2: Into<String> + Clone, S3: Into<String>
    {
        let format = display_format.map(|f| {
            match f.into().as_ref() {
                "pretty" => OutputFormat::Pretty,
                "json" => OutputFormat::JSON,
                custom => OutputFormat::Custom(custom.to_string())
            }
        }).unwrap_or(OutputFormat::Pretty);

        let extractor = fields
            .map(|f| JSONExtractor::filtered(f.iter().map(|s| s.clone().into())))
            .unwrap_or_else(|| JSONExtractor::default());

        SearchCommand {
            client: Box::new(ElasticClient::create(server_config.clone(), 20)),
            request: SearchRequest {
                query: query.into(),
                index: index.map(|s| s.into())
                    .or_else(|| server_config.default_index.clone())
                    .unwrap_or("*".to_string())
            },
            renderer: Box::new(Renderer::create(format, extractor))
        }
    }
}