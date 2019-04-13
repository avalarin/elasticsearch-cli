use crate::ApplicationError;

pub trait Command {
    fn execute(&mut self) -> Result<(), ApplicationError>;
}