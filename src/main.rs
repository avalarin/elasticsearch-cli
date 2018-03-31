#[macro_use] extern crate clap; 
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
mod error;

use std::str::FromStr;
use clap::{App, ArgMatches};
use config::{ApplicationConfig};
use commands::{Command, CommandError};

fn main() {
    match run_application() {
        Err(cause) => {
            error!("{}", cause);
            std::process::exit(1);
        }
        _ => { }
    }
}

fn run_application() -> Result<(), error::ApplicationError> {
    let yaml = load_yaml!("app.yaml");
    let app = App::from_yaml(yaml);
    let args = app.get_matches();

    configure_logger(&args)?;

    let config = args.value_of("config")
        .map_or_else(ApplicationConfig::load_default, ApplicationConfig::load_file)
        .map_err(|cause| error::ApplicationError::GeneralError(Box::new(cause)))?;

    match args.subcommand() {
        ("search", Some(sub_match)) => execute_search(config, &args, sub_match),
        ("config", Some(sub_match)) => execute_config(config, sub_match),
        _ => { 
            println!("{}", args.usage());
            Ok(())
        }
    }.map_err(|cause| error::ApplicationError::GeneralError(Box::new(cause)))
}

fn configure_logger(args: &ArgMatches) -> Result<(), error::ApplicationError> {
    let verbose = args.occurrences_of("verbosity") as usize;
    let quiet = args.is_present("quiet");
    stderrlog::new()
        .module(module_path!())
        .quiet(quiet)
        .verbosity(verbose + 1)
        .init()
        .map_err(|cause| error::ApplicationError::GeneralError(Box::new(cause)))
}

fn execute_search(config: config::ApplicationConfig, matches: &ArgMatches, sub_match: &ArgMatches) -> Result<(), commands::CommandError> {
    let server = config.get_server(matches.value_of("server"))
        .ok_or(CommandError::InvalidArgument("server not found"))?;
        
    info!("Using server {}", server.server);

    let query = sub_match.value_of("query").ok_or(CommandError::InvalidArgument("query required"))?;
    let index = sub_match.value_of("index");
    let path = sub_match.value_of("path");
    let fields = sub_match.value_of("fields").map(|f| f.split(",").collect());
    let output_format = sub_match.value_of("output")
                .map(commands::OutputFormat::from_str)
                .unwrap_or(Ok(commands::OutputFormat::Pretty()))?;
    let mut command = commands::SearchCommand::new(server, index, query, path, fields, output_format);
    command.execute()
}

fn execute_config(config: ApplicationConfig, sub_m: &ArgMatches) -> Result<(), commands::CommandError> {
    commands::ConfigCommand::parse(config, sub_m)?.execute()
}
