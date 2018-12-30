use std::clone::Clone;
use clap::{ArgMatches};
use commands::{Command, CommandError};
use config::{ApplicationConfig, ElasticSearchServer};
use serde_yaml;

pub struct ConfigCommand {
    pub config: ApplicationConfig,
    pub action: ConfigAction
}

#[derive(Clone)]
pub enum ConfigAction {
    AddServer{name: String, address: String, index: Option<String>},
    UpdateServer{name: String, address: Option<String>, index: Option<String>},
    UseServer{name: String}
}

impl ConfigCommand {
    pub fn new(config: ApplicationConfig, action: ConfigAction) -> Self {
        ConfigCommand{ config, action }
    }

    pub fn parse(config: ApplicationConfig, args: &ArgMatches) -> Result<Self, CommandError> {
        let action = match args.subcommand() {
            ("add", Some(add_match)) => {
                match add_match.subcommand() {
                    ("server", Some(server_match)) => {
                        let name = server_match.value_of("name").ok_or(CommandError::InvalidArgument("[name] is required"))?;
                        let address = server_match.value_of("address").ok_or(CommandError::InvalidArgument("[address] is required"))?;
                        let index = server_match.value_of("index");
                        Ok(ConfigAction::AddServer{ name: name.to_owned(), address: address.to_owned(), index: index.map(str::to_owned) })
                    },
                    _ => { Err(CommandError::InvalidArgument("Unknown resource")) }
                }
            },
            ("update", Some(update_match)) => {
                match update_match.subcommand() {
                    ("server", Some(server_match)) => {
                        let name = server_match.value_of("name").ok_or(CommandError::InvalidArgument("[name] is required"))?;
                        let address = server_match.value_of("address");
                        let index = server_match.value_of("index");
                        Ok(ConfigAction::UpdateServer{ name: name.to_owned(), address: address.map(str::to_owned), index: index.map(str::to_owned) })
                    },
                    _ => { Err(CommandError::InvalidArgument("Unknown resource")) }
                }
            },
            ("use", Some(use_match)) => {
                match use_match.subcommand() {
                    ("server", Some(server_match)) => {
                        let name = server_match.value_of("name").ok_or(CommandError::InvalidArgument("[name] is required"))?;
                        Ok(ConfigAction::UseServer{ name: name.to_owned() })
                    },
                    _ => { Err(CommandError::InvalidArgument("Unknown resource")) }
                }
            },
            _ => { Err(CommandError::InvalidArgument("Unknown action")) }
        }?;
        Ok(ConfigCommand::new(config, action))
    }
}

impl Command<CommandError> for ConfigCommand {
    fn execute(&mut self) -> Result<(), CommandError> {
        match self.action.clone() {
            ConfigAction::AddServer{name, address, index} => {
                if self.config.servers.iter().any(|server| server.name == name) {
                    return Err(CommandError::InvalidArgument("Server already exists"))
                }
                if self.config.default_server.is_none() {
                    self.config.default_server = Some(name.clone());
                }
                self.config.servers.push(ElasticSearchServer {
                    name,
                    server: address,
                    default_index: index
                });
            },
            ConfigAction::UpdateServer{name, address, index} => {
                let mut server = self.config.servers.iter_mut().find(|server| server.name == name)
                    .ok_or(CommandError::InvalidArgument("Server don't exists"))?;
                
                if let Some(addr) = address {
                    server.server = addr
                }
                if index.is_some() {
                    server.default_index = index;
                }
            },
            ConfigAction::UseServer{name} => {
                if self.config.servers.iter().find(|server| server.name == name).is_none() {
                    return Err(CommandError::InvalidArgument("Server don't exists"))
                }
                self.config.default_server = Some(name);
            }
        }

        info!("Saving new config to file {}", self.config.file_path);
        println!("{}\n{}", self.config.file_path, serde_yaml::to_string(&self.config).map_err(|_| CommandError::InvalidArgument("Can't serialize"))?);

        self.config.save_file()
            .map_err(|cause| CommandError::CommonError(Box::new(cause)))
    }
}
