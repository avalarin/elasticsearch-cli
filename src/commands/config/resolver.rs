use super::{
    ConfigAction, ApplicationConfig, PasswordQuestioner
};

use crate::config::{ElasticSearchServer, SecretsWriter, WriteSecretError};

use std::sync::Arc;
use commands::config::password_questioner::PasswordQuestionerError;

pub struct ConfigActionResolver {
    password_questioner: Arc<dyn PasswordQuestioner>,
    secrets_writer: Arc<dyn SecretsWriter>
}

#[derive(Debug, Fail, PartialEq)]
pub enum ConfigActionError {
    #[fail(display = "server {} already exists", server_name)]
    ServerAlreadyExists { server_name: String },
    #[fail(display = "server {} does not exists", server_name)]
    ServerDoesNotExists { server_name: String },
    #[fail(display = "username should be specified, use --username")]
    UsernameShouldBeSpecified,
    #[fail(display = "cannot save password: {}", inner)]
    CannotSavePassword { inner: WriteSecretError },
    #[fail(display = "{}", inner)]
    CannotRetrievePassword { inner: PasswordQuestionerError },
}

impl ConfigActionResolver {
    pub fn new(
        password_questioner: Arc<dyn PasswordQuestioner>,
        secrets_writer: Arc<dyn SecretsWriter>
    ) -> Self {
        Self {
            password_questioner,
            secrets_writer
        }
    }

    pub fn resolve(&self, action: ConfigAction, mut config: ApplicationConfig) -> Result<ApplicationConfig, ConfigActionError> {
        match action {
            ConfigAction::AddServer {
                name,
                address,
                server_type,
                index,
                username,
                password
            } => {
                if config.servers.iter().any(|server| server.name == name) {
                    return Err(ConfigActionError::ServerAlreadyExists { server_name: name })
                }
                if config.default_server.is_none() {
                    config.default_server = Some(name.clone());
                }
                config.servers.push(ElasticSearchServer {
                    name,
                    server: address,
                    server_type,
                    default_index: index,
                    username: username.clone()
                });

                let password_needed = username.is_some();
                if let Some((username, password)) = self.fetch_credentials(username, password, password_needed)? {
                    info!("Saving password to the system keychain...");
                    self.secrets_writer.write(&username, &password)
                        .map_err(|err| {
                            ConfigActionError::CannotSavePassword { inner: err }
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
                let mut server = config.servers.iter_mut().find(|server| server.name == name)
                    .ok_or_else(|| {
                        ConfigActionError::ServerDoesNotExists { server_name: name }
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

                let password_needed = username.is_some() || ask_password;
                if let Some((username, password)) = self.fetch_credentials(server.username.clone(), password, password_needed)? {
                    info!("Saving password to the system keychain...");
                    self.secrets_writer.write(&username, &password)
                        .map_err(|err| {
                            ConfigActionError::CannotSavePassword { inner: err }
                        })?;
                }
            }
            ConfigAction::UseServer { name } => {
                if config.servers.iter().find(|server| server.name == name).is_none() {
                    return Err(ConfigActionError::ServerDoesNotExists { server_name: name })
                }
                config.default_server = Some(name);
            }
            ConfigAction::Show => {}
        };

        Ok(config)
    }

    fn fetch_credentials(
        &self,
        username: Option<String>,
        password: Option<String>,
        ask_password: bool
    ) -> Result<Option<(String, String)>, ConfigActionError> {
        match (username, password, ask_password) {
            (Some(username), Some(password), _) => Ok(Some((username, password))),
            (Some(username), None, true) => {
                self.password_questioner.ask_password(&username)
                    .map(|p| Some((username, p)))
                    .map_err(|err| {
                        ConfigActionError::CannotRetrievePassword { inner: err }
                    })
            },
            (None, Some(_), _) | (None, None, true) => {
                Err(ConfigActionError::UsernameShouldBeSpecified)
            },
            (_, None, false) => Ok(None)
        }
    }

}