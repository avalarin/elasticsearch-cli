use config::ElasticSearchServer;
use commands::{Command, CommandError};
use std::error::Error;

pub struct SearchCommand {
    server_config: ElasticSearchServer
}

pub struct SearchParams {
    pub query: String,
    pub index: Option<String>
}

impl SearchParams {
    pub fn new<S1, S2>(index: Option<S1>, query: S2) -> Self 
        where S1: Into<String>, S2: Into<String> 
    {
        SearchParams {
            query: query.into(),
            index: index.map(Into::into)
        }
    }
}

impl SearchCommand {
    pub fn new(server_config: &ElasticSearchServer) -> Self {
        SearchCommand { server_config: server_config.clone() }
    }

    fn execute_with_index(&self, params: SearchParams, index: String)  -> Result<(), CommandError> {
        info!("Executing search {} on index {}", params.query, index);
        Ok(())

        // let mut client = Client::new(self.server_config.server.as_ref())
        //     .map_err(|parse_err| CommandError::CommonError(parse_err.description()))?;

        // client.search_uri()
        //     .with_indexes(&[index.as_ref()])
        //     .with_query(params.query)
        //     .send()
        //     .map_err(|es_err| CommandError::CommonError(es_err.description()))
        //     .map(|resp| ())
        //     //.and_then(|resp| self.display(resp))
    }
}

impl Command<SearchParams, CommandError> for SearchCommand {
    fn execute(&self, params: SearchParams) -> Result<(), CommandError> {
        params.index.clone()
            .or(self.server_config.default_index.clone())
            .ok_or(CommandError::InvalidArgument("index required"))
            .and_then(|index| self.execute_with_index(params, index))
    }
}
