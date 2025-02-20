#[derive(thiserror::Error, Debug)]
// TODO: understanding these macros?
pub enum Error {
    #[error("invalid input: `{0}`")]
    InvalidInput(String),
    // TODO: what is transparent
    #[error(transparent)]
    IO(#[from] std::io::Error),
}
