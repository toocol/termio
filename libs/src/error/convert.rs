use super::{code::ErrorCode::*, Error};
use std::{fmt, io};

impl From<io::Error> for Error {
    #[inline]
    fn from(value: io::Error) -> Self {
        Self::new_empty(IoError, value.to_string())
    }
}

impl From<serde_json::Error> for Error {
    #[inline]
    fn from(value: serde_json::Error) -> Self {
        Self::new_empty(SerdeJsonError, value.to_string())
    }
}

impl From<fmt::Error> for Error {
    fn from(value: fmt::Error) -> Self {
        Self::new_empty(FmtError, value.to_string())
    }
}
