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
#[macro_use]
extern crate failure;

mod config;
mod commands;
mod error;
mod client;
mod display;

use clap::{App, ArgMatches};
use config::ApplicationConfig;
use commands::{Command};
use error::ApplicationError;

fn main() {
    if let Err(_) = run_application() {
        std::process::exit(1);
    }
}

fn run_application() -> Result<(), ApplicationError> {
    let yaml = load_yaml!("app.yaml");
    let app = App::from_yaml(yaml);
    let args = app.get_matches();

    configure_logger(&args)?;

    let config = args.value_of("config")
        .map_or_else(ApplicationConfig::load_default, ApplicationConfig::load_file)
        .map_err(|err| {
            error!("Cannot read configuration: {}", err);
            ApplicationError
        })?;

    match args.subcommand() {
        ("search", Some(sub_match)) => commands::SearchCommand::parse(&config, &args, sub_match)?.execute(),
        ("config", Some(sub_match)) => commands::ConfigCommand::parse(config, sub_match)?.execute(),
        _ => {
            println!("{}", args.usage());
            Err(ApplicationError)
        }
    }
}

fn configure_logger(args: &ArgMatches) -> Result<(), ApplicationError> {
    let verbose = args.occurrences_of("verbosity") as usize;
    let quiet = args.is_present("quiet");
    stderrlog::new()
        .module(module_path!())
        .quiet(quiet)
        .verbosity(verbose + 1)
        .init()
        .map_err(|err| {
            error!("Cannot configure logger: {}", err);
            ApplicationError
        })
}
