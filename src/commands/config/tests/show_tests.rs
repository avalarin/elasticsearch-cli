use super::{create_config_with_one_server, create_resolver};

use commands::ConfigAction;

#[test]
fn should_do_nothing() {
    let config = create_config_with_one_server();
    let (resolver, _, _) = create_resolver();
    let new_config = resolver.resolve(ConfigAction::Show {}, config.clone()).unwrap();
    assert_eq!(config, new_config);
}