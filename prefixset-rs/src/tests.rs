use std::{fmt, num::ParseIntError};

use crate::error::Error;

#[derive(Debug)]
pub enum TestError {
    Wrapped(Error),
    Ip(ip::Error),
    Int(ParseIntError),
    Str(&'static str),
}

impl std::error::Error for TestError {}

impl fmt::Display for TestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Wrapped(err) => err.fmt(f),
            Self::Ip(err) => err.fmt(f),
            Self::Int(err) => err.fmt(f),
            Self::Str(msg) => f.write_str(msg),
        }
    }
}

impl From<Error> for TestError {
    fn from(err: Error) -> Self {
        Self::Wrapped(err)
    }
}

impl From<ip::Error> for TestError {
    fn from(err: ip::Error) -> Self {
        Self::Ip(err)
    }
}

impl From<ParseIntError> for TestError {
    fn from(err: ParseIntError) -> Self {
        Self::Int(err)
    }
}

impl From<&'static str> for TestError {
    fn from(msg: &'static str) -> Self {
        Self::Str(msg)
    }
}

pub type TestResult = Result<(), TestError>;
