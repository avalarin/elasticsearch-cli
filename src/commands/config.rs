use std::clone::Clone;
use clap::{ArgMatches};
use commands::{Command};
use config::{ApplicationConfig, ElasticSearchServer, ElasticSearchServerType, SecretsStorage};
use serde_yaml;
use error::ApplicationError;

use std::str::FromStr;
use std::sync::Arc;

pub struct ConfigCommand {
    pub config: ApplicationConfig,
    pub action: ConfigAction,
    pub secrets: Arc<SecretsStorage>
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
    pub fn new(config: ApplicationConfig, secrets: Arc<SecretsStorage>, action: ConfigAction) -> Self {
        ConfigCommand { config, action, secrets }
    }

    pub fn parse(config: ApplicationConfig, secrets: Arc<SecretsStorage>, args: &ArgMatches) -> Result<Self, ApplicationError> {
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

    fn read_password_from_tty(username: &String) -> Result<String, ApplicationError> {
        rpassword::read_password_from_tty(Some(&format!("Enter the password for the user {}: ", username)))
            .map_err(|err| {
                error!("Cannot read a password: {}", err);
                ApplicationError
            })
    }

    fn fetch_credentials(username: Option<String>, password: Option<String>, ask_password: bool) -> Result<Option<(String, String)>, ApplicationError> {
        match (username, password, ask_password) {
            (Some(username), Some(password), _) => Ok(Some((username, password))),
            (Some(username), None, true) => ConfigCommand::read_password_from_tty(&username).map(|p| Some((username, p))),
            (Some(_), None, false) => {
                error!("The password should be provided: neither --password nor --ask-password is used");
                Err(ApplicationError)
            }
            (None, Some(_), _) | (None, None, true) => {
                error!("The username should be specified, use --username");
                Err(ApplicationError)
            },
            (None, None, false) => Ok(None)
        }
    }
}

impl Command for ConfigCommand {
    fn execute(&mut self) -> Result<(), ApplicationError> {
        match self.action.clone() {
            ConfigAction::AddServer {
                name,
                address,
                server_type,
                index,
                username,
                password
            } => {
                if self.config.servers.iter().any(|server| server.name == name) {
                    error!("Cannot create new server: server with that name already exists");
                    return Err(ApplicationError);
                }
                if self.config.default_server.is_none() {
                    self.config.default_server = Some(name.clone());
                }
                self.config.servers.push(ElasticSearchServer {
                    name,
                    server: address,
                    server_type,
                    default_index: index,
                    username: username.clone()
                });

                let password_needed = username.is_some();
                if let Some((username, password)) = ConfigCommand::fetch_credentials(username, password, password_needed)? {
                    info!("Saving password to the system keychain...");
                    self.secrets.write(&username, &password)
                        .map_err(|err| {
                            error!("Cannot save password: {}", err);
                            ApplicationError
                        })?;
                }
            }
            ConfigAction::UpdateServer {
                name,
                address,
                server_type,
                index,
                username,
                password,
                ask_password
            } => {
                let mut server = self.config.servers.iter_mut().find(|server| server.name == name)
                    .ok_or_else(|| {
                        error!("Server with name {} doesn't exists", name);
                        ApplicationError
                    })?;

                if let Some(addr) = address {
                    server.server = addr
                }
                if let Some(server_type) = server_type {
                    server.server_type = server_type
                }
                if index.is_some() {
                    server.default_index = index;
                }
                if username.is_some() {
                    server.username = username.clone();
                }

                if let Some((username, password)) = ConfigCommand::fetch_credentials(server.username.clone(), password, ask_password)? {
                    info!("Saving password to the system keychain...");
                    self.secrets.write(&username, &password)
                        .map_err(|err| {
                            error!("Cannot save password: {}", err);
                            ApplicationError
                        })?;
                }
            }
            ConfigAction::UseServer { name } => {
                if self.config.servers.iter().find(|server| server.name == name).is_none() {
                    error!("Server with name {} doesn't exists", name);
                    return Err(ApplicationError);
                }
                self.config.default_server = Some(name);
            }
            ConfigAction::Show => {}
        }

        info!("Saving new config to file {}", self.config.file_path);
        println!("{}\n{}", self.config.file_path, serde_yaml::to_string(&self.config)
            .map_err(|err| {
                error!("Can't serialize configuration: {}", err);
                ApplicationError
            })?);

        self.config.save_file()
            .map_err(|err| {
                error!("Can't save configuration: {}", err);
                ApplicationError
            })
    }
}
