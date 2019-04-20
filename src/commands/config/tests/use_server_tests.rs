use super::{create_config, create_config_with_one_server, create_resolver};

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
fn should_set_new_default_server() {
    let config = create_config_with_one_server();
    let (resolver, _, _) = create_resolver();

    let new_config = resolver.resolve(ConfigAction::UseServer {
        name: "test".to_string(),
    }, config).unwrap();

    assert_eq!(Some("test".to_string()), new_config.default_server);
}