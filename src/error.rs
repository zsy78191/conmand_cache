use std::fmt;
use rust_i18n::t;

#[derive(Debug)]
pub enum Error {
    HomeEnv(std::env::VarError),
    Io(std::io::Error),
    CommandFailed(i32),
    InvalidArgument,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::HomeEnv(_) => write!(f, "{}", t!("error.home_env")),
            Error::Io(e) => write!(f, "{}", t!("error.io", detail = e)),
            Error::CommandFailed(code) => {
                write!(f, "{}", t!("error.command_failed", code = code))
            }
            Error::InvalidArgument => write!(f, "{}", t!("error.invalid_arg")),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::HomeEnv(e) => Some(e),
            Error::Io(e) => Some(e),
            Error::CommandFailed(_) | Error::InvalidArgument => None,
        }
    }
}

impl From<std::env::VarError> for Error {
    fn from(e: std::env::VarError) -> Self {
        Error::HomeEnv(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
