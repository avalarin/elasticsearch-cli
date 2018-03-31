use std::error::Error;

quick_error! {
    #[derive(Debug)]
    pub enum ApplicationError {
        GeneralError(cause: Box<Error>) {
            description(cause.description())
            display("Error occured: {}", cause)
        }
    }
}

// impl ApplicationError {
//     fn wrap<E>(cause: E) -> Self where E: Error + Clone {
//         let cloned = cause.clone();
//         ApplicationError::GeneralError(Box::new(cloned))
//     }
// }

// impl <E: Error> From<E> for ApplicationError {
//     fn from(cause: E) -> Self {
//         ApplicationError::GeneralError(Box::new(cause))
//     }
// }