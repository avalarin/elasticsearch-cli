use rpassword::read_password_from_tty;

#[derive(Debug, Fail, PartialEq)]
#[fail(display = "{}", inner)]
pub struct PasswordQuestionerError {
    inner: String
}

pub trait PasswordQuestioner {
    fn ask_password(&self, username: &str) -> Result<String, PasswordQuestionerError>;
}

pub struct TtyPasswordQuestioner {

}

impl TtyPasswordQuestioner {
    pub fn new() -> Self {
        Self {}
    }
}

impl PasswordQuestioner for TtyPasswordQuestioner {
    fn ask_password(&self, username: &str) -> Result<String, PasswordQuestionerError> {
        read_password_from_tty(Some(&format!("Enter the password for the user {}: ", username)))
            .map_err(|err| {
                PasswordQuestionerError { inner: format!("{}", err) }
            })
    }
}