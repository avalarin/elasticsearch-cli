use config::{SecretsWriter, WriteSecretError};
use commands::config::{PasswordQuestioner, PasswordQuestionerError};

use std::cell::{Cell, RefCell};
use commands::config::resolver::ConfigActionResolver;
use std::sync::Arc;

pub fn create_resolver() -> (ConfigActionResolver, Arc<TestPasswordQuestioner>, Arc<TestSecrets>) {
    let secrets = Arc::new(TestSecrets::new());
    let questioner = Arc::new(TestPasswordQuestioner::new());
    let resolver = ConfigActionResolver::new(
        questioner.clone(),
        secrets.clone()
    );
    (resolver, questioner, secrets)
}

pub struct TestPasswordQuestioner {
    is_called: Cell<bool>
}
impl TestPasswordQuestioner {
    pub fn new() -> Self { Self { is_called: Cell::new(false) } }
    pub fn was_asked(&self) -> bool {
        self.is_called.get()
    }
}
impl PasswordQuestioner for TestPasswordQuestioner {
    fn ask_password(&self, _username: &str) -> Result<String, PasswordQuestionerError> {
        self.is_called.set(true);
        Ok("asked_password".to_string())
    }
}

pub struct TestSecrets{
    password: RefCell<String>
}
impl TestSecrets {
    pub fn new() -> Self { Self { password: RefCell::new("".to_string()) } }
    pub fn assert_check(&self, pwd: String) { assert_eq!(pwd, *self.password.borrow()); }
}
impl SecretsWriter for TestSecrets {
    fn write(&self, _key: &str, secret: &str) -> Result<(), WriteSecretError> {
        self.password.replace(secret.to_string());
        Ok(())
    }
}