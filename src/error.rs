use std::fmt;
use std::num::ParseIntError;

use ipnet::{AddrParseError, PrefixLenError};

#[derive(Debug)]
pub enum Error {
    AddrParse(AddrParseError),
    PrefixLen(PrefixLenError),
    RangeParse { source: Option<ParseIntError> },
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::RangeParse {
                source: Some(source),
            } => Some(source),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::AddrParse(ref err) => err.fmt(f),
            Self::PrefixLen(ref err) => err.fmt(f),
            Self::RangeParse { .. } => f.write_str("invalid IP prefix range"),
        }
    }
}

impl From<AddrParseError> for Error {
    fn from(err: AddrParseError) -> Self {
        Self::AddrParse(err)
    }
}

impl From<ParseIntError> for Error {
    fn from(err: ParseIntError) -> Self {
        Self::RangeParse { source: Some(err) }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
