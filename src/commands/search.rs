use config::ElasticSearchServer;
use commands::{Command, CommandError};
use reqwest;
use serde_json;
use es;
use colored::*;
use std::collections::{HashSet, HashMap, BTreeSet};
use std::iter::{FromIterator};
use std::str::FromStr;
use strfmt::strfmt;

pub struct SearchCommand {
    pub server_config: ElasticSearchServer,

    pub query: String,
    pub index: Option<String>,
    pub skip_path: Option<String>,
    pub fields: Option<HashSet<String>>,
    pub output_format: OutputFormat,

    results: Vec<SearchItem>
}

pub enum OutputFormat {
    Pretty(),
    JSON(),
    Custom(String)
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

struct SearchItem {
    keys_max_length: usize,
    fields: HashMap<String, String>,
    json: serde_json::Value
}

impl SearchCommand {
    pub fn new<S1, S2, S3, S4>(
        server_config: &ElasticSearchServer, 
        index: Option<S1>, 
        query: S2, 
        skip_path: Option<S3>, 
        fields: Option<Vec<S4>>, 
        output_format: OutputFormat
    ) -> Self where S1: Into<String>, S2: Into<String>, S3: Into<String>, S4: Into<String> + Clone
    {
        SearchCommand {
            server_config: server_config.clone(),
            query: query.into(),
            index: index.map(Into::into),
            skip_path: skip_path.map(Into::into),
            fields: fields.map(|f| f.iter().cloned().map(|s| s.into()).collect::<Vec<String>>())
                          .map(HashSet::from_iter),
            output_format: output_format,
            results: vec![]
        }
    }

    fn get_index(&self) -> Result<String, CommandError> {
        self.index.clone()
            .or(self.server_config.default_index.clone())
            .ok_or(CommandError::InvalidArgument("index required"))
    }

    fn is_field_ok(&self, field: &str) -> bool {
        self.fields.clone()
            .map(|f| f.contains(field))
            .unwrap_or(true)
    }

    fn get_skip_path(&self) -> Option<String> {
        self.skip_path.clone()
            .or(self.server_config.skip_path.clone())
    }

    fn try_skip_path<'a>(&self, value: &'a serde_json::Value) -> &'a serde_json::Value {
        match (self.get_skip_path(), value) {
            (Some(ref path), &serde_json::Value::Object(ref object)) => &object[path],
            _ => value
        }
    }

    fn collect(&mut self, response: es::EsResponse) {
        for hit in response.hits.hits.iter() {
            let mut map = HashMap::new();
            let skipped = self.try_skip_path(hit);
            self.collect_hit(vec![], skipped, &mut map);

            let max_length = map.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
            let item = SearchItem { 
                fields: map,
                keys_max_length: max_length,
                json: skipped.clone()
            };
            self.results.push(item);
        }
    }

    fn collect_hit(&self, path: Vec<String>, hit: &serde_json::Value, map: &mut HashMap<String, String>) {
        match hit {
            &serde_json::Value::Object(ref object) => {
                for (key, value) in object {
                    let mut new_path = path.clone();
                    new_path.push(key.clone());
                    self.collect_hit(new_path, &value, map);
                }
            },
            &serde_json::Value::Array(ref array) => {
                for (index, value) in array.iter().enumerate() {
                    let mut new_path = path.clone();
                    new_path.push(index.to_string());
                    self.collect_hit(new_path, &value, map);
                }
            },
            primitive => {
                let str_prefix = path.join(".");
                if self.is_field_ok(&str_prefix) {
                    map.insert(str_prefix, self.prepare_primitive(primitive));
                }
            }
        }
    }

    fn display_pretty(&self) {
        for (index, item) in self.results.iter().enumerate() {
            if index > 0 {
                println!("{}", "-".repeat(item.keys_max_length).blue().bold());
            }
            for key in item.fields.keys().collect::<BTreeSet<&String>>() {
                println!("{:indent$}: {}", key.green().bold(), self.format_string(&item.fields[key]), indent=item.keys_max_length);
            }
        }
    }

    fn display_json(&self) {
        for item in self.results.iter() {
            println!("{}", item.json);
        }
    }

    fn display_custom(&self, format: &str) {
        for item in self.results.iter() {
            println!("{}", strfmt(format, &item.fields).unwrap_or("Cannot format item".to_owned()));
        }
    }

    fn prepare_primitive(&self, value: &serde_json::Value) -> String {
        match value {
            &serde_json::Value::String(ref str_value) => format!("{}", str_value),
            primitive => format!("{}", primitive)
        }
    }

    fn format_string(&self, value: &str) -> String {
        Some(value.into())
            .map(|s| str::replace(s, "\\n", "\n"))
            .map(|s| str::replace(&s, "\\t", "\t"))
            .unwrap()
    }
}

impl Command<CommandError> for SearchCommand {
    fn execute(&mut self) -> Result<(), CommandError> {
        let index = self.get_index()?;
        info!("Executing search {} on index {}", self.query, index);

        let client = reqwest::Client::new();
        let url = reqwest::Url::parse(self.server_config.server.as_ref())
            .and_then(|u| u.join(&format!("{}/_search", index)))?;

        let req = json!({
            "query": {
                "query_string" : {
                    "query" : self.query
                }
            }
        }).to_string();

        info!("Sending request to {}: {}", url, req);
        
        let resp = client.post(url).body(req).send()?;

        let parsed = parse(resp)?;

        info!("Found {} results", parsed.hits.total);

        self.collect(parsed);

        match self.output_format {
            OutputFormat::Pretty() => self.display_pretty(),
            OutputFormat::JSON() => self.display_json(),
            OutputFormat::Custom(ref format) => self.display_custom(format)
        };

        Ok(())
    }
}

fn parse(mut resp: reqwest::Response) -> Result<es::EsResponse, CommandError> {
    resp.text()
        .map_err(From::from)
        .and_then(|json| serde_json::from_str(json.as_ref()).map_err(From::from))
}