use keyring::{Keyring, KeyringError};

pub trait SecretsReader {
    fn read(&self, key: &str) -> Result<Option<String>, ReadSecretError>;

    fn get_credentials(&self, username: &str) -> Result<Option<Credentials>, ReadSecretError>;
}

pub trait SecretsWriter {
    fn write(&self, key: &str, secret: &str) -> Result<(), WriteSecretError>;
}

pub struct SystemSecretsStorage {
    service: String
}

#[derive(Debug, Fail)]
#[fail(display = "cannot read secret by key {}: {}", key, inner)]
pub struct ReadSecretError {
    key: String,
    inner: String
}

#[derive(Debug, Fail, PartialEq)]
#[fail(display = "cannot save secret by key {}: {}", key, inner)]
pub struct WriteSecretError {
    key: String,
    inner: String
}

#[derive(Clone)]
pub struct Credentials {
    pub username: String,
    pub password: String
}

impl SystemSecretsStorage {
    pub fn new(service: impl Into<String>) -> Self {
        Self { service: service.into() }
    }
}

impl SecretsReader for SystemSecretsStorage {
    fn read(&self, key: &str) -> Result<Option<String>, ReadSecretError> {
        let secret = Keyring::new(&self.service, key)
            .get_password();
        match secret {
            Ok(secret) => Ok(Some(secret)),
            Err(KeyringError::NoPasswordFound) => Ok(None),
            Err(err) => Err(ReadSecretError { key: key.to_string(), inner: format!("{}", err) })
        }
    }

    fn get_credentials(&self, username: &str) -> Result<Option<Credentials>, ReadSecretError> {
        self.read(username)
            .map(|password| {
                password.map(|password| Credentials { username: username.to_string(), password })
            })
    }
}

impl SecretsWriter for SystemSecretsStorage {
    fn write(&self, key: &str, secret: &str) -> Result<(), WriteSecretError> {
        Keyring::new(&self.service, &key)
            .set_password(secret)
            .map_err(|err| {
                WriteSecretError { key: key.to_string(), inner: format!("{}", err) }
            })
    }
}