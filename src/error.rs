use serde::{de, ser};
use std::fmt::{self, Display};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Message(String),

    Syntax,
    Identifier,
    ExpectedBoolean,
    ExpectedU8,
    ExpectedOption,
    ExpectedNull,
    TrailingCharacters,
    CannotDeserializeAny,
    UnknownSeqLength,

    IoError(std::io::Error),
    TryFromIntError(std::num::TryFromIntError),
    FromUtf8Error(std::string::FromUtf8Error),
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(err: std::string::FromUtf8Error) -> Self {
        Error::FromUtf8Error(err)
    }
}

impl From<std::num::TryFromIntError> for Error {
    fn from(err: std::num::TryFromIntError) -> Self {
        Error::TryFromIntError(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err)
    }
}

impl std::error::Error for Error {}
