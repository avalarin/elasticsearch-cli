use std::error::Error;

pub trait Command<P, E>
    where E: Error
{
    fn execute(&self, params: P) -> Result<(), E>;
}

quick_error! {
    #[derive(Debug)]
    pub enum CommandError {
        InvalidArgument(descr: &'static str) {
            description(descr)
            display("Invalid arguments: {}", descr)
        }
        CommonError(descr: &'static str) {
            description(descr)
            display("Error: {}", descr)
        }
    }
}