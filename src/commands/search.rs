use config::ElasticSearchServer;
use commands::{Command, CommandError};
use serde_json;
use colored::*;
use std::collections::{HashSet, HashMap};
use std::iter::{FromIterator, Iterator};
use std::str::FromStr;
use strfmt::strfmt;
use elastic::prelude::*;
use elastic::http::header::{Authorization, Basic};

pub struct SearchCommand {
    pub server_config: ElasticSearchServer,

    pub buffer_size: i32,
    pub size: i32,
    pub query: String,
    pub index: Option<String>,
    pub fields: Option<HashSet<String>>,
    pub output_format: OutputFormat,

    from: u64,
}

pub enum OutputFormat {
    Pretty(),
    JSON(),
    Custom(String),
}

impl FromStr for OutputFormat {
    type Err = CommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pretty" => Ok(OutputFormat::Pretty()),
            "json" => Ok(OutputFormat::JSON()),
            custom => Ok(OutputFormat::Custom(custom.to_owned()))
        }
    }
}

impl SearchCommand {
    pub fn new<S1, S2, S3>(
        buffer_size: i32,
        size: i32,
        server_config: &ElasticSearchServer,
        index: Option<S1>,
        query: S2,
        fields: Option<Vec<S3>>,
        output_format: OutputFormat,
    ) -> Self
        where S1: Into<String>, S2: Into<String>, S3: Into<String> + Clone
    {
        SearchCommand {
            buffer_size,
            size,
            server_config: server_config.clone(),
            query: query.into(),
            index: index.map(Into::into),
            fields: fields.map(|f| f.iter().cloned().map(|s| s.into()).collect::<Vec<String>>())
                .map(HashSet::from_iter),
            output_format,
            from: 0,
        }
    }

    fn get_index(&self) -> Result<String, CommandError> {
        self.index.clone()
            .or_else(|| self.server_config.default_index.clone())
            .ok_or(CommandError::InvalidArgument("index required"))
    }

    fn is_field_ok(&self, field: &str) -> bool {
        self.fields.clone()
            .map(|f| f.contains(field))
            .unwrap_or(true)
    }

    fn query_next(&mut self) -> Result<bool, CommandError> {
        let index = self.get_index()?;

//        Placeholder for future Bearer Token auth
        let token = match &self.server_config.username {
            Some(user) => match &self.server_config.password {
                Some(pass) => Some(base64::encode(&format!("{}:{}", user, pass))),
                None => Some(base64::encode(&format!("{}:", user)))
            }
            None => None
        };

        let mut builder = SyncClientBuilder::new();
        if let Some(username) = &self.server_config.username {
            builder = builder.params(
                |p| {
                    p.header(Authorization(Basic {
                        username: username.to_owned(),
                        password: self.server_config.password.clone(),
                    }))
                }
            );
        }

        let client = builder.base_url(self.server_config.server.as_ref())
            .build()
            .map_err(|err| {
                error!("Cannot create elasticsearch client: {}", err);
                CommandError::CommonError(Box::new(err))
            })?;

        let resp = client.search::<serde_json::Value>()
            .index(index)
            .body(json!({
                "size": self.buffer_size,
                "from": self.from,
                "query": {
                    "query_string" : {
                        "query" : self.query
                    }
                }
            }))
            .send()
            .map_err(|err| {
                error!("Cannot read response from elasticsearch: {}", err);
                CommandError::CommonError(Box::new(err))
            })?;

        let total_count = resp.total();
        let result_count = resp.documents().count() as u64;
        info!("Loaded {}/{} results", self.from + result_count, total_count);

        resp.documents()
            .enumerate()
            .for_each(|(index, item)| self.display(index, item));

        self.from += result_count;
        Ok(self.from != total_count)
    }

    fn collect(&self, document: &serde_json::Value) -> HashMap<String, String> {
        let mut map = HashMap::new();
        self.collect_hit(&[], document, &mut map);
        map
    }

    fn collect_hit(&self, path: &[String], hit: &serde_json::Value, map: &mut HashMap<String, String>) {
        match hit {
            &serde_json::Value::Object(ref object) => {
                for (key, value) in object {
                    let mut new_path = path.to_owned();
                    new_path.push(key.clone());
                    self.collect_hit(&new_path, &value, map);
                }
            }
            &serde_json::Value::Array(ref array) => {
                for (index, value) in array.iter().enumerate() {
                    let mut new_path = path.to_owned();
                    new_path.push(index.to_string());
                    self.collect_hit(&new_path, &value, map);
                }
            }
            primitive => {
                let str_prefix = path.join(".");
                if self.is_field_ok(&str_prefix) {
                    map.insert(str_prefix, self.prepare_primitive(primitive));
                }
            }
        }
    }

    fn display(&self, index: usize, item: &serde_json::Value) {
        match self.output_format {
            OutputFormat::JSON() => {
                println!("{}", item)
            }
            OutputFormat::Pretty() => {
                if index > 0 {
                    println!("{}", "-".repeat(4).blue().bold());
                }
                for (key, value) in &self.collect(item) {
                    println!("{}: {}", key.green().bold(), self.format_string(&value));
                }
            }
            OutputFormat::Custom(ref format) => {
                println!("{}", strfmt(format, &self.collect(item)).unwrap_or_else(|_| "Cannot format item".to_owned()));
            }
        };
    }

    fn prepare_primitive(&self, value: &serde_json::Value) -> String {
        match value {
            &serde_json::Value::String(ref str_value) => str_value.to_string(),
            primitive => primitive.to_string()
        }
    }

    fn format_string(&self, value: &str) -> String {
        Some(value)
            .map(|s| str::replace(s, "\\n", "\n"))
            .map(|s| str::replace(&s, "\\t", "\t"))
            .unwrap()
    }
}

impl Command<CommandError> for SearchCommand {
    fn execute(&mut self) -> Result<(), CommandError> {
        let index = self.get_index()?;
        info!("Executing search {} on index {}", self.query, index);

        while self.query_next()? {};

        Ok(())
    }
}
