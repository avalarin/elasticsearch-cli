extern crate serde_yaml;

use std;
use std::fmt;

quick_error! {
    #[derive(Debug)]
    pub enum GetServerError {
        ServerNotFound(server: String) {
            description("Server {} not found")
            display("Server {} not found", server)
        }
        NoConfiguredServers {
            description("No servers configured")
            display("No servers configured")
        }
        ServerNotSpecified {
            description("Server not specified")
            display("Server not specified")
        }
    }
}

pub enum Error {
    YamlError(Box<std::error::Error>),
    FileSystemError(Box<std::error::Error>),
    CannotFindHomeDirectory()
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::YamlError(ref cause) => cause.fmt(f),
            Error::FileSystemError(ref cause) => cause.fmt(f),
            Error::CannotFindHomeDirectory() => write!(f, "Cannot find home directory")
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::YamlError(ref cause) => cause.fmt(f),
            Error::FileSystemError(ref cause) => cause.fmt(f),
            Error::CannotFindHomeDirectory() => write!(f, "Cannot find home directory")
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::YamlError(ref cause) => cause.description(),
            Error::FileSystemError(ref cause) => cause.description(),
            Error::CannotFindHomeDirectory() => "Cannot find home directory"
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            Error::YamlError(_) => None,
            Error::FileSystemError(_) => None,
            Error::CannotFindHomeDirectory() => None
        }
    }
}

impl From<::std::env::JoinPathsError> for Error {
    fn from(error: ::std::env::JoinPathsError) -> Self {
        Error::FileSystemError(Box::new(error))
    }
}

impl From<::std::io::Error> for Error {
    fn from(error: ::std::io::Error) -> Self {
        Error::FileSystemError(Box::new(error))
    }
}

impl From<::serde_yaml::Error> for Error {
    fn from(error: ::serde_yaml::Error) -> Self {
        Error::YamlError(Box::new(error))
    }
}