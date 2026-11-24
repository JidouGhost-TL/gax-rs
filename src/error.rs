//! Error types for GAX parsing and synthesis.

use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Error {
    InvalidMagic,
    UnsupportedVersion(u8),
    UnexpectedEof,
    InvalidPattern(u32),
    InvalidSample(u32),
    CorruptedData(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMagic => write!(f, "Invalid GAX container magic"),
            Self::UnsupportedVersion(v) => write!(f, "Unsupported GAX version: {v}"),
            Self::UnexpectedEof => write!(f, "Unexpected end of file while parsing GAX"),
            Self::InvalidPattern(id) => write!(f, "Invalid pattern index {id}"),
            Self::InvalidSample(id) => write!(f, "Invalid sample index {id}"),
            Self::CorruptedData(msg) => write!(f, "Corrupted GAX stream: {msg}"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

pub type Result<T> = core::result::Result<T, Error>;
