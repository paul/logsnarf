use thiserror::Error;

use crate::{decoder, metric_writer, parser};

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] ::std::io::Error),

    #[error(transparent)]
    ParseError(#[from] parser::ParseError),

    #[error(transparent)]
    DecodeError(#[from] decoder::DecodeError),

    // #[error(transparent)]
    // CredentialsStoreError(#[from] credentials::CredentialsStoreError),
    #[error(transparent)]
    AdapterError(#[from] metric_writer::WriterError),

    #[error("{0}")]
    Msg(String),
}

impl From<&'static str> for Error {
    fn from(s: &'static str) -> Self {
        Error::Msg(s.to_owned())
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Msg(s)
    }
}

pub type Result<T> = std::result::Result<T, Error>;