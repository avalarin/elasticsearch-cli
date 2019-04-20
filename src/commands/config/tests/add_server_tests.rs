use super::{create_config, create_resolver};

use config::{ElasticSearchServer, ElasticSearchServerType};
use commands::config::resolver::{ConfigActionError};

use commands::ConfigAction;

#[test]
fn should_not_creates_2_servers_with_same_name() {
    let mut config = create_config();
    let (resolver, _, _) = create_resolver();

    config.servers.push(ElasticSearchServer {
        name: "test".to_string(),
        server: "".to_string(),
        server_type: ElasticSearchServerType::Elastic,
        default_index: None,
        username: None
    });

    let result = resolver.resolve(ConfigAction::AddServer {
        name: "test".to_string(),
        address: "".to_string(),
        server_type: ElasticSearchServerType::Elastic,
        index: None,
        username: None,
        password: None
    }, config);

    assert_eq!(Err(ConfigActionError::ServerAlreadyExists { server_name: "test".to_string() }), result);
}

#[test]
fn should_sets_default_server_if_its_not_set() {
    let config = create_config();
    let (resolver, _, _) = create_resolver();

    let new_config = resolver.resolve(ConfigAction::AddServer {
        name: "test".to_string(),
        address: "".to_string(),
        server_type: ElasticSearchServerType::Elastic,
        index: None,
        username: None,
        password: None
    }, config).unwrap();

    assert_eq!(Some("test".to_string()), new_config.default_server);
}

#[test]
fn should_puts_new_server() {
    let config = create_config();
    let (resolver, _, _) = create_resolver();

    let new_config = resolver.resolve(ConfigAction::AddServer {
        name: "test".to_string(),
        address: "address".to_string(),
        server_type: ElasticSearchServerType::Kibana,
        index: Some("index".to_string()),
        username: Some("username".to_string()),
        password: None
    }, config).unwrap();

    assert_eq!(vec![
        ElasticSearchServer {
            name: "test".to_string(),
            server: "address".to_string(),
            server_type: ElasticSearchServerType::Kibana,
            default_index: Some("index".to_string()),
            username: Some("username".to_string())
        }
    ], new_config.servers);
}

#[test]
fn should_asks_for_password_if_username_is_present() {
    let config = create_config();
    let (resolver, password, secrets) = create_resolver();

    resolver.resolve(ConfigAction::AddServer {
        name: "test".to_string(),
        address: "address".to_string(),
        server_type: ElasticSearchServerType::Kibana,
        index: Some("index".to_string()),
        username: Some("username".to_string()),
        password: None
    }, config).unwrap();

    assert_eq!(true, password.was_asked());
    secrets.assert_check("asked_password".to_string());
}

#[test]
fn should_not_asks_for_password_if_username_is_not_present() {
    let config = create_config();
    let (resolver, password, secrets) = create_resolver();

    resolver.resolve(ConfigAction::AddServer {
        name: "test".to_string(),
        address: "address".to_string(),
        server_type: ElasticSearchServerType::Kibana,
        index: Some("index".to_string()),
        username: None,
        password: None
    }, config).unwrap();

    assert_eq!(false, password.was_asked());
    secrets.assert_check("".to_string());
}

#[test]
fn should_not_asks_for_password_if_password_is_present() {
    let config = create_config();
    let (resolver, password, secrets) = create_resolver();

    resolver.resolve(ConfigAction::AddServer {
        name: "test".to_string(),
        address: "address".to_string(),
        server_type: ElasticSearchServerType::Kibana,
        index: Some("index".to_string()),
        username: Some("username".to_string()),
        password: Some("password".to_string())
    }, config).unwrap();

    assert_eq!(false, password.was_asked());
    secrets.assert_check("password".to_string());
}

#[test]
fn should_fails_when_password_is_specified_but_username_is_not() {
    let config = create_config();
    let (resolver, _password, _secrets) = create_resolver();

    let result = resolver.resolve(ConfigAction::AddServer {
        name: "test".to_string(),
        address: "address".to_string(),
        server_type: ElasticSearchServerType::Kibana,
        index: Some("index".to_string()),
        username: None,
        password: Some("password".to_string())
    }, config);

    assert_eq!(Err(ConfigActionError::UsernameShouldBeSpecified), result);
}