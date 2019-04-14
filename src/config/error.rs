extern crate serde_yaml;

#[derive(Debug, Fail)]
pub enum GetServerError {
    #[fail(display = "server {} not found", server)]
    ServerNotFound { server: String },
    #[fail(display = "no servers configured")]
    NoConfiguredServers,
    #[fail(display = "server not specified")]
    ServerNotSpecified
}

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "cannot deserialize yaml: {}", inner)]
    YamlError { inner: serde_yaml::Error },
    #[fail(display = "cannot read file: {}", inner)]
    IOError { inner: std::io::Error },
    #[fail(display = "cannot find config file: {}", inner)]
    PathError { inner: std::env::JoinPathsError },
    #[fail(display = "cannot find home directory")]
    CannotFindHomeDirectory()
}

impl From<::std::env::JoinPathsError> for Error {
    fn from(error: ::std::env::JoinPathsError) -> Self {
        Error::PathError { inner: error }
    }
}

impl From<::std::io::Error> for Error {
    fn from(error: ::std::io::Error) -> Self {
        Error::IOError { inner: error }
    }
}

impl From<::serde_yaml::Error> for Error {
    fn from(error: ::serde_yaml::Error) -> Self {
        Error::YamlError { inner: error }
    }
}