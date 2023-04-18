use std::fmt;
use std::num::ParseIntError;

/// Errors returned by construction and parsing operations.
#[derive(Debug)]
pub enum Error {
    /// IP address handling error
    IpAddr(ip::Error),
    /// The IP address or prefix couldn't be parsed.
    AddrParse(ip::Error),
    /// The IP prefix length was out of bounds.
    PrefixLen(ip::Error),
    /// The IP prefix range couldn't be parsed.
    RangeParse {
        /// The error returned during parsing, if any.
        source: Option<ParseIntError>,
    },
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IpAddr(err) => Some(err),
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
            Self::IpAddr(ref err) => write!(f, "IP address handling error: {}", err),
            Self::AddrParse(ref err) => err.fmt(f),
            Self::PrefixLen(ref err) => err.fmt(f),
            Self::RangeParse { .. } => f.write_str("invalid IP prefix range"),
        }
    }
}

impl From<ip::Error> for Error {
    fn from(err: ip::Error) -> Self {
        Self::IpAddr(err)
    }
}

impl From<ParseIntError> for Error {
    fn from(err: ParseIntError) -> Self {
        Self::RangeParse { source: Some(err) }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
