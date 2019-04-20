mod mocks;
mod add_server_tests;
mod update_server_tests;
mod use_server_tests;
mod show_tests;

use self::mocks::create_resolver;
pub use self::add_server_tests::*;
pub use self::update_server_tests::*;
pub use self::use_server_tests::*;
pub use self::show_tests::*;

use config::{ApplicationConfig, ElasticSearchServer, ElasticSearchServerType};

fn create_config() -> ApplicationConfig {
    ApplicationConfig {
        file_path: "".to_string(),
        default_server: None,
        servers: vec![ ]
    }
}

fn create_config_with_one_server() -> ApplicationConfig {
    ApplicationConfig {
        file_path: "".to_string(),
        default_server: None,
        servers: vec![
            ElasticSearchServer {
                name: "test".to_string(),
                server: "address".to_string(),
                server_type: ElasticSearchServerType::Elastic,
                default_index: None,
                username: None
            }
        ]
    }
}
