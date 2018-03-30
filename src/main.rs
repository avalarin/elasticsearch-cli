extern crate clap; 
#[macro_use] extern crate serde_derive;
extern crate serde_yaml;
#[macro_use] extern crate serde_json;
extern crate stderrlog;
#[macro_use] extern crate log;
#[macro_use] extern crate quick_error;
extern crate reqwest;
extern crate colored;
extern crate strfmt;

mod config;
mod commands;
mod es;

use std::str::FromStr;
use clap::{App, Arg, ArgMatches, SubCommand};
use config::ApplicationConfig;
use commands::{Command, CommandError};

fn main() {
    let matches = App::new("elastic-cli")
       .version("0.1")
       .about("Does great things!")
       .author("Alexandr P.")
       .arg(Arg::with_name("verbosity").short("v").multiple(true).help("Increase message verbosity"))
       .arg(Arg::with_name("quiet").short("q").help("Silence all output"))
       .arg_from_usage("--config=[FILE] 'Path to the configuration file'")
       .arg_from_usage("--server=[SERVER] 'Server name'")
       .subcommand(SubCommand::with_name("search")
            .about("Search logs by the query")
            .arg_from_usage("-p, --period=[PEROID] 'For example: today, week, month'")
            .arg_from_usage("-i, --index=[INDEX] 'Elasticsearch index or index pattern'")
            .arg_from_usage("-q, --query=[QUERY] 'Query'")
            .arg_from_usage("--skip=[SKIP] 'Base'")
            .arg_from_usage("-f, --fields=[FIELDS] 'Fields'")
            .arg_from_usage("-o, --output=[OUTPUT] 'Output format'"))
       .get_matches();

    let verbose = matches.occurrences_of("verbosity") as usize;
    let quiet = matches.is_present("quiet");
    stderrlog::new()
        .module(module_path!())
        .quiet(quiet)
        .verbosity(verbose + 1)
        .init()
        .expect("Cannot initialize logger");

    let config = matches.value_of("config")
        .map_or_else(ApplicationConfig::load_default, ApplicationConfig::load_file)
        .expect("Cannot load config");

    let server_name = matches.value_of("server").unwrap_or(&config.default_server);
    let server = config.get_server(server_name).expect("Cannot find server");
    info!("Using server {} ({})", server_name, server.server);

    match matches.subcommand() {
        ("search", Some(sub_m)) => execute_search(server, sub_m),
        _ => { 
            println!("{}", matches.usage());
            Ok(())
        }
    }.expect("Cannot execute command");
}

fn execute_search(server: &config::ElasticSearchServer, sub_match: &ArgMatches) -> Result<(), commands::CommandError> {
    let query = sub_match.value_of("query").ok_or(CommandError::InvalidArgument("query required"))?;
    let index = sub_match.value_of("index");
    let skip_path = sub_match.value_of("skip");
    let fields = sub_match.value_of("fields").map(|f| f.split(",").collect());
    let output_format = sub_match.value_of("output")
                .map(commands::OutputFormat::from_str)
                .unwrap_or(Ok(commands::OutputFormat::Pretty()))?;
    let mut command = commands::SearchCommand::new(server, index, query, skip_path, fields, output_format);
    command.execute()
}