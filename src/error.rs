use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("HOME environment variable not set")]
    HomeEnv(#[from] std::env::VarError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("command failed with exit code: {0}")]
    CommandFailed(i32),

    #[error("`-s` requires a search term")]
    InvalidArgument,
}

pub type Result<T> = std::result::Result<T, Error>;
