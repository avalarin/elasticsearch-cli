#[macro_use]
extern crate clap;
extern crate elastic;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
#[macro_use]
extern crate serde_json;
extern crate stderrlog;
#[macro_use]
extern crate log;
#[macro_use]
extern crate quick_error;
extern crate reqwest;
extern crate colored;
extern crate strfmt;
extern crate dirs;
extern crate base64;

mod config;
mod commands;
mod error;

use std::str::FromStr;
use clap::{App, ArgMatches};
use config::ApplicationConfig;
use commands::{Command, CommandError};

fn main() {
    if let Err(cause) = run_application() {
        error!("{}", cause);
        std::process::exit(1);
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
        ("search", Some(sub_match)) => execute_search(&config, &args, sub_match),
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

fn execute_search(config: &config::ApplicationConfig, matches: &ArgMatches, sub_match: &ArgMatches) -> Result<(), commands::CommandError> {
    let server = match config.get_server(matches.value_of("server")) {
        Ok(server) => server,
        Err(config::GetServerError::ServerNotFound(name)) => {
            error!("Server with name '{}' not found", name);
            return Ok(());
        }
        Err(config::GetServerError::ServerNotSpecified) => {
            error!("The server is not specified.");
            error!("Hint: use 'elastic-cli config use server <name>'");
            error!("Hint: use option --server, e.g. 'elastic-cli --server <name> search ...'");
            return Ok(());
        }
        Err(config::GetServerError::NoConfiguredServers) => {
            error!("There are no servers in the config file");
            error!("Hint: use 'elastic-cli config add server <name> --address <address>'");
            return Ok(());
        }
    };

    let size = sub_match.value_of("size").map(str::parse).unwrap_or(Ok(1000)).map_err(|_| CommandError::InvalidArgument("size has invalid value"))?;
    let buffer_size = sub_match.value_of("buffer").map(str::parse).unwrap_or(Ok(1000)).map_err(|_| CommandError::InvalidArgument("buffer has invalid value"))?;
    let query = sub_match.value_of("query").ok_or(CommandError::InvalidArgument("query required"))?;
    let index = sub_match.value_of("index");
    let fields = sub_match.value_of("fields").map(|f| f.split(',').collect());
    let output_format = sub_match.value_of("output")
        .map(commands::OutputFormat::from_str)
        .unwrap_or(Ok(commands::OutputFormat::Pretty()))?;
    let mut command = commands::SearchCommand::new(buffer_size, size, server, index, query, fields, output_format);
    command.execute()
}

fn execute_config(config: ApplicationConfig, sub_m: &ArgMatches) -> Result<(), commands::CommandError> {
    commands::ConfigCommand::parse(config, sub_m)?.execute()
}
