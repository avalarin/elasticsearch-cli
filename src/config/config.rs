use std::fs::{File, OpenOptions};
use std::vec::Vec;
use std::env;
use std::path::{Path, PathBuf};
use std::io::Write;
use config;
use serde_yaml;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ApplicationConfig {
    #[serde(skip_deserializing,skip_serializing)]
    pub file_path: String,

    pub default_server: Option<String>,
    pub servers: Vec<ElasticSearchServer>
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ElasticSearchServer {
    pub name:   String,
    pub server: String,
    pub default_index: Option<String>,
    pub default_path: Option<String>
}

impl ApplicationConfig {

    pub fn load_default() -> Result<ApplicationConfig, config::Error> {
        env::home_dir()
            .ok_or(config::Error::CannotFindHomeDirectory())
            .map(|home_dir| home_dir.join(PathBuf::from(".elastic-cli")).into_os_string())
            .and_then(|os_str| os_str.into_string().map_err(|_| config::Error::CannotFindHomeDirectory()))
            .and_then(|path| ApplicationConfig::load_file_or_create(&path))
    }

    fn load_file_or_create(path: &str) -> Result<ApplicationConfig, config::Error> {
        if !Path::new(path).exists() {
            warn!("File {} does not exist, creating...", path);
            let config = ApplicationConfig{
                file_path: path.to_owned(),
                default_server: None,
                servers: vec![]
            };
            return config.save_file().map(|_| config)
        }
        ApplicationConfig::load_file(path)
    }

    pub fn load_file(path: &str) -> Result<ApplicationConfig, config::Error> {
        info!("Loading config from: {}", path);

        File::open(path)
            .map_err(|err| {
                error!("Cannot open file {}: {}", path, err);
                From::from(err)
            })
            .and_then(|file| serde_yaml::from_reader::<File, ApplicationConfig>(file).map_err(From::from))
            .map(|mut config| {
                config.file_path = path.to_owned();
                config
            })
    }

    pub fn get_server<S>(&self, name: Option<S>) -> Option<&ElasticSearchServer> where S: Into<String> {
        let server_name = match (name, &self.default_server) {
            (Some(s_name), _) => s_name.into(),
            (None, &Some(ref s_name)) => s_name.to_owned(),
            _ => return None
        };

        return self.servers.iter().find(|server| server.name == server_name)
    }

    pub fn save_file(&self) -> Result<(), config::Error> {
        let mut file = self.open_file_or_create()?;
        let yaml = serde_yaml::to_string(self).map_err(|err| {
            error!("Cannot serialize configuration: {}", err);
            config::Error::FileSystemError(Box::new(err))
        })?;
        file.write_all(yaml.as_bytes()).map_err(|err| {
            error!("Cannot write configuration file: {}", err);
            From::from(err)
        })
    }

    fn open_file_or_create(&self) -> Result<File, config::Error> {
        if Path::new(&self.file_path).exists() {
            OpenOptions::new().write(true).open(&self.file_path).map_err(|err| {
                error!("Cannot open configuration file {}: {}", self.file_path, err);
                config::Error::FileSystemError(Box::new(err))
            })
        } else {
            File::create(&self.file_path).map_err(|err| {
                error!("Cannot create empty configuration file {}: {}", self.file_path, err);
                config::Error::FileSystemError(Box::new(err))
            })
        }
    }

}