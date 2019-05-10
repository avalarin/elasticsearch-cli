#[macro_use]
extern crate clap;
extern crate elastic;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
#[macro_use]
extern crate serde_json;
extern crate stderrlog;
#[macro_use]
extern crate log;
extern crate reqwest;
extern crate colored;
extern crate strfmt;
extern crate dirs;
extern crate base64;
#[macro_use]
extern crate failure;
extern crate keyring;
extern crate rpassword;
extern crate termion;
extern crate core;

mod config;
mod commands;
mod error;
mod client;
mod display;
mod ui;
mod utils;

use clap::{App, ArgMatches};
use config::{ApplicationConfig, SystemSecretsStorage};
use commands::{Command};
use error::ApplicationError;

use std::sync::Arc;

fn main() {
    if run_application().is_err() {
        std::process::exit(1);
    }
}

fn run_application() -> Result<(), ApplicationError> {
    let yaml = load_yaml!("app.yaml");
    let app = App::from_yaml(yaml)
        .name(env!("CARGO_PKG_NAME"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .version(env!("CARGO_PKG_VERSION"));
    let args = app.get_matches();

    configure_logger(&args)?;

    let secrets = Arc::new(SystemSecretsStorage::new("elastic-cli"));

    let config = args.value_of("config")
        .map_or_else(ApplicationConfig::load_default, ApplicationConfig::load_file)
        .map_err(|err| {
            error!("Cannot read configuration: {}", err);
            ApplicationError
        })?;

//    match args.subcommand() {
//        ("search", Some(sub_match)) => commands::SearchCommand::parse(&config, secrets, &args, sub_match)?.execute(),
//        ("config", Some(sub_match)) => commands::ConfigCommand::parse(config, secrets, sub_match)?.execute(),
//        _ => {
//            println!("{}", args.usage());
//            Err(ApplicationError)
//        }
//    }

    ui::core::UiCore::start(
        ui::views::RootView::new(),
        ui::reducers::RootReducer::new(),
        ui::state::State::default()
    );

    Ok(())
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
