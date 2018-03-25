extern crate clap; 
#[macro_use] extern crate serde_derive;
extern crate serde_yaml;
extern crate pretty_env_logger;
#[macro_use] extern crate log;
#[macro_use] extern crate quick_error;

mod config;
mod commands;

use clap::{App, SubCommand};
use config::ApplicationConfig;
use commands::{Command, CommandError};

fn main() { 
    pretty_env_logger::init().expect("Cannot create logger");
    
    let matches = App::new("elastic-cli")
       .version("0.1")
       .about("Does great things!")
       .author("Alexandr P.")
       .arg_from_usage("--config=[FILE] 'Path to the configuration file'")
       .arg_from_usage("--server=[SERVER] 'Server name'")
       .subcommand(SubCommand::with_name("search")
            .about("Search logs by the query")
            .arg_from_usage("-p, --period=[PEROID] 'For example: today, week, month'")
            .arg_from_usage("-i, --index=[INDEX] 'Elasticsearch index or index pattern'")
            .arg_from_usage("-q, --query=[QUERY] 'Query'"))
       .get_matches();

    let config = matches.value_of("config")
        .map_or_else(ApplicationConfig::load_default, ApplicationConfig::load_file)
        .expect("Cannot load config");

    let server_name = matches.value_of("server").unwrap_or(&config.default_server);
    let server = config.get_server(server_name).expect("Cannot find server");
    debug!("Using server {} ({})", server_name, server.server);

    match matches.subcommand() {
        ("search", Some(sub_m)) => {
            sub_m.value_of("query")
                .ok_or(CommandError::InvalidArgument("query required"))
                .map(|query| (sub_m.value_of("index"), query))
                .map(|(index, query)| commands::SearchParams::new(index, query))
                .and_then(|params| commands::SearchCommand::new(server).execute(params))
        },
        _ => { 
            println!("{}", matches.usage());
            Ok(())
        }
    }.expect("Cannot execute command");
}