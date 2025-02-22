#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid input: `{0}`")]
    InvalidInput(String),
    #[error(transparent)]
    IO(#[from] std::io::Error),
}
