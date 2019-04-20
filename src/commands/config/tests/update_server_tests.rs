use super::{create_config, create_config_with_one_server, create_resolver};

use config::{ElasticSearchServerType};
use commands::config::resolver::{ConfigActionError};

use commands::ConfigAction;

#[test]
fn should_fails_on_updating_nonexistent_server() {
    let config = create_config();
    let (resolver, _, _) = create_resolver();

    let result = resolver.resolve(ConfigAction::UpdateServer {
        name: "test".to_string(),
        address: None,
        server_type: None,
        index: None,
        username: None,
        password: None,
        ask_password: false
    }, config);

    assert_eq!(Err(ConfigActionError::ServerDoesNotExists { server_name: "test".to_string() }), result);
}

#[test]
fn should_not_change_config_if_no_options_provided() {
    let config = create_config_with_one_server();
    let (resolver, _, _) = create_resolver();

    let new_config = resolver.resolve(ConfigAction::UpdateServer {
        name: "test".to_string(),
        address: None,
        server_type: None,
        index: None,
        username: None,
        password: None,
        ask_password: false
    }, config.clone()).unwrap();

    assert_eq!(config, new_config);
}

#[test]
fn should_update_address() {
    let mut config = create_config_with_one_server();
    let (resolver, _, _) = create_resolver();

    let new_config = resolver.resolve(ConfigAction::UpdateServer {
        name: "test".to_string(),
        address: Some("updated_address".to_string()),
        server_type: None,
        index: None,
        username: None,
        password: None,
        ask_password: false
    }, config.clone()).unwrap();

    config.servers.get_mut(0).unwrap().server = "updated_address".to_string();

    assert_eq!(config, new_config);
}


#[test]
fn should_update_server_type() {
    let mut config = create_config_with_one_server();
    let (resolver, _, _) = create_resolver();

    let new_config = resolver.resolve(ConfigAction::UpdateServer {
        name: "test".to_string(),
        address: None,
        server_type: Some(ElasticSearchServerType::Kibana),
        index: None,
        username: None,
        password: None,
        ask_password: false
    }, config.clone()).unwrap();

    config.servers.get_mut(0).unwrap().server_type = ElasticSearchServerType::Kibana;

    assert_eq!(config, new_config);
}

#[test]
fn should_update_default_index() {
    let mut config = create_config_with_one_server();
    let (resolver, _, _) = create_resolver();

    let new_config = resolver.resolve(ConfigAction::UpdateServer {
        name: "test".to_string(),
        address: None,
        server_type: None,
        index: Some("updated_index".to_string()),
        username: None,
        password: None,
        ask_password: false
    }, config.clone()).unwrap();

    config.servers.get_mut(0).unwrap().default_index = Some("updated_index".to_string());

    assert_eq!(config, new_config);
}

#[test]
fn should_asks_for_password_if_username_is_present() {
    let mut config = create_config_with_one_server();
    let (resolver, password, secrets) = create_resolver();

    let new_config = resolver.resolve(ConfigAction::UpdateServer {
        name: "test".to_string(),
        address: None,
        server_type: None,
        index: None,
        username: Some("updated_username".to_string()),
        password: None,
        ask_password: false
    }, config.clone()).unwrap();

    config.servers.get_mut(0).unwrap().username = Some("updated_username".to_string());

    assert!(password.was_asked());
    assert_eq!(config, new_config);
    secrets.assert_check("asked_password".to_string())
}

#[test]
fn should_not_ask_for_password_if_password_is_provided() {
    let mut config = create_config_with_one_server();
    let (resolver, password, secrets) = create_resolver();

    let new_config = resolver.resolve(ConfigAction::UpdateServer {
        name: "test".to_string(),
        address: None,
        server_type: None,
        index: None,
        username: Some("updated_username".to_string()),
        password: Some("updated_password".to_string()),
        ask_password: false
    }, config.clone()).unwrap();

    config.servers.get_mut(0).unwrap().username = Some("updated_username".to_string());

    assert!(!password.was_asked());
    assert_eq!(config, new_config);
    secrets.assert_check("updated_password".to_string())
}

#[test]
fn should_not_ask_for_password_if_ask_password_is_not_provided() {
    let config = create_config_with_one_server();
    let (resolver, password, _) = create_resolver();

    resolver.resolve(ConfigAction::UpdateServer {
        name: "test".to_string(),
        address: None,
        server_type: None,
        index: None,
        username: None,
        password: None,
        ask_password: false
    }, config.clone()).unwrap();

    assert!(!password.was_asked());
}

#[test]
fn should_fails_if_password_provided_for_config_without_username() {
    let config = create_config_with_one_server();
    let (resolver, _password, _secrets) = create_resolver();

    let result = resolver.resolve(ConfigAction::UpdateServer {
        name: "test".to_string(),
        address: None,
        server_type: None,
        index: None,
        username: None,
        password: Some("updated_password".to_string()),
        ask_password: false
    }, config.clone());

    assert_eq!(Err(ConfigActionError::UsernameShouldBeSpecified), result);
}

#[test]
fn should_fails_if_password_ask_provided_for_config_without_username() {
    let config = create_config_with_one_server();
    let (resolver, _password, _secrets) = create_resolver();

    let result = resolver.resolve(ConfigAction::UpdateServer {
        name: "test".to_string(),
        address: None,
        server_type: None,
        index: None,
        username: None,
        password: None,
        ask_password: true
    }, config.clone());

    assert_eq!(Err(ConfigActionError::UsernameShouldBeSpecified), result);
}