use std::error::Error;
use reqwest;
use serde_json;

pub trait Command<E>
    where E: Error
{
    fn execute(&mut self) -> Result<(), E>;
}

quick_error! {
    #[derive(Debug)]
    pub enum CommandError {
        InvalidArgument(descr: &'static str) {
            description(descr)
            display("Invalid arguments: {}", descr)
        }
        CommonError(cause: Box<Error>) {
            description(cause.description())
            display("Error: {}", cause.description())

            from(cause: reqwest::UrlError) -> (Box::new(cause))
            from(cause: reqwest::Error) -> (Box::new(cause))
            from(cause: serde_json::Error) -> (Box::new(cause))
        }
    }
}