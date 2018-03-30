use std::fs::File;
use std::vec::Vec;
use std::env;
use std::path::PathBuf;
use config::Error;
use serde_yaml;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ApplicationConfig {
    pub default_server: String,
    pub servers: Vec<ElasticSearchServer>
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ElasticSearchServer {
    pub name:   String,
    pub server: String,
    pub default_index: Option<String>,
    pub skip_path: Option<String>
}

impl ApplicationConfig {

    pub fn load_default() -> Result<ApplicationConfig, Error> {
        env::home_dir()
            .ok_or(Error::CannotFindHomeDirectory())
            .map(|home_dir| home_dir.join(PathBuf::from(".elastic-cli")).into_os_string())
            .and_then(|os_str| os_str.into_string().map_err(|_| Error::CannotFindHomeDirectory()))
            .and_then(|path| ApplicationConfig::load_file(&path))
    }

    pub fn load_file(path: &str) -> Result<ApplicationConfig, Error> {
        info!("Loading config from: {}", path);

        File::open(path)
            .map_err(From::from)
            .and_then(|file| serde_yaml::from_reader(file).map_err(From::from))
    }

    pub fn get_server(&self, name: &str) -> Option<&ElasticSearchServer> {
        self.servers.iter().find(|server| server.name == name)
    }

}