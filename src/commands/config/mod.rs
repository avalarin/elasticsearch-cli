mod resolver;
mod password_questioner;

#[cfg(test)]
mod tests;

pub use self::resolver::*;
pub use self::password_questioner::*;

use clap::{ArgMatches};
use commands::{Command};
use config::{ApplicationConfig, ElasticSearchServerType, SecretsWriter};
use serde_yaml;
use error::ApplicationError;

use std::str::FromStr;
use std::sync::Arc;

pub struct ConfigCommand {
    pub config: ApplicationConfig,
    pub action: ConfigAction,
    pub resolver: ConfigActionResolver
}

#[derive(Clone)]
pub enum ConfigAction {
    AddServer {
        name: String,
        address: String,
        server_type: ElasticSearchServerType,
        index: Option<String>,
        username: Option<String>,
        password: Option<String>
    },
    UpdateServer {
        name: String,
        address: Option<String>,
        server_type: Option<ElasticSearchServerType>,
        index: Option<String>,
        username: Option<String>,
        password: Option<String>,
        ask_password: bool
    },
    UseServer { name: String },
    Show
}

impl ConfigCommand {
    pub fn new(config: ApplicationConfig, secrets: Arc<dyn SecretsWriter>, action: ConfigAction) -> Self {
        ConfigCommand {
            config,
            action,
            resolver: ConfigActionResolver::new(
                Arc::new(TtyPasswordQuestioner::new()),
                secrets
            )
        }
    }

    pub fn parse(config: ApplicationConfig, secrets: Arc<dyn SecretsWriter>, args: &ArgMatches) -> Result<Self, ApplicationError> {
        let action = match args.subcommand() {
            ("add", Some(add_match)) => {
                match add_match.subcommand() {
                    ("server", Some(server_match)) => {
                        let name = server_match.value_of("name").ok_or_else(|| {
                            error!("Argument 'name' is required");
                            ApplicationError
                        })?;
                        let address = server_match.value_of("address").ok_or_else(|| {
                            error!("Argument 'address' is required");
                            ApplicationError
                        })?;
                        let index = server_match.value_of("index");
                        let username = server_match.value_of("username");
                        let password = server_match.value_of("password");
                        let server_type = server_match.value_of("type")
                            .map(FromStr::from_str)
                            .unwrap_or(Ok(ElasticSearchServerType::Elastic))
                            .map_err(|err| {
                                error!("{}", err);
                                ApplicationError
                            })?;
                        Ok(ConfigAction::AddServer {
                            name: name.to_owned(),
                            address: address.to_owned(),
                            server_type,
                            index: index.map(str::to_owned),
                            username: username.map(str::to_owned),
                            password: password.map(str::to_owned),
                        })
                    },
                    (resource, _) => {
                        error!("Unknown resource - {}", resource);
                        Err(ApplicationError)
                    }
                }
            }
            ("update", Some(update_match)) => {
                match update_match.subcommand() {
                    ("server", Some(server_match)) => {
                        let name = server_match.value_of("name").ok_or_else(|| {
                            error!("Argument 'name' is required");
                            ApplicationError
                        })?;
                        let address = server_match.value_of("address");
                        let index = server_match.value_of("index");
                        let username = server_match.value_of("username");
                        let password = server_match.value_of("password");
                        let ask_password = server_match.is_present("ask-password");
                        let server_type = server_match.value_of("type")
                            .map(ElasticSearchServerType::from_str)
                            .map_or(Ok(None), |v| v.map(Some))
                            .map_err(|err| {
                                error!("{}", err);
                                ApplicationError
                            })?;

                        Ok(ConfigAction::UpdateServer {
                            name: name.to_owned(),
                            address: address.map(str::to_owned),
                            server_type,
                            index: index.map(str::to_owned),
                            username: username.map(str::to_owned),
                            password: password.map(str::to_owned),
                            ask_password
                        })
                    },
                    (resource, _) => {
                        error!("Unknown resource - {}", resource);
                        Err(ApplicationError)
                    }
                }
            }
            ("use", Some(use_match)) => {
                match use_match.subcommand() {
                    ("server", Some(server_match)) => {
                        let name = server_match.value_of("name").ok_or_else(|| {
                            error!("Argument 'name' is required");
                            ApplicationError
                        })?;
                        Ok(ConfigAction::UseServer { name: name.to_owned() })
                    }
                    (resource, _) => {
                        error!("Unknown resource - {}", resource);
                        Err(ApplicationError)
                    }
                }
            }
            ("show", _) => Ok(ConfigAction::Show),
            (action, _) => {
                error!("Unknown configuration action - {}", action);
                Err(ApplicationError)
            }
        }?;
        Ok(ConfigCommand::new(config, secrets, action))
    }
}

impl Command for ConfigCommand {
    fn execute(&mut self) -> Result<(), ApplicationError> {
        let new_config = self.resolver.resolve(self.action.clone(), self.config.clone())
            .map_err(|err| {
                error!("Cannot perform action: {}", err);
                ApplicationError
            })?;

        info!("Saving new config to file {}", new_config.file_path);
        println!("{}\n{}", new_config.file_path, serde_yaml::to_string(&new_config)
            .map_err(|err| {
                error!("Can't serialize configuration: {}", err);
                ApplicationError
            })?);

        new_config.save_file()
            .map_err(|err| {
                error!("Can't save configuration: {}", err);
                ApplicationError
            })
    }
}
