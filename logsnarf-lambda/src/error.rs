use std::{error, fmt};

#[derive(Debug)]
pub enum Error {
    InitError(&'static str),
    ClientError(&'static str),
    InternalError(&'static str),
    SdkError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        match self {
            Error::InitError(msg) => write!(f, "InitError: {}", msg),
            Error::ClientError(msg) => write!(f, "ClientError: {}", msg),
            Error::InternalError(msg) => write!(f, "InternalError: {}", msg),
            Error::SdkError(err) => write!(f, "SdkError: {}", err),
        }
    }
}

impl error::Error for Error {}

/// This is defined as a convenience.
pub type Result<T> = std::result::Result<T, Error>;
