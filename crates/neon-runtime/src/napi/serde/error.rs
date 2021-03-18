//! Error type and conversions for serde transcoding

use std::error;
use std::fmt;

use conv::{FloatError, PosOverflow, RangeError};
use serde_crate::{de, ser};

use crate::napi;

#[derive(Clone, Debug, PartialEq)]
/// This type represents all possible errors that can occur when serializing or
/// deserializing JavaScript types.
pub struct Error {
    kind: ErrorKind,
}

impl error::Error for Error {}

impl Error {
    fn new(kind: ErrorKind) -> Self {
        Self { kind }
    }

    /// Indicates if the error was due to an exception in the JavaScript VM
    /// If an exception is pending, all other JavaScript operations will fail
    /// until it is cleared.
    pub fn is_exception_pending(&self) -> bool {
        self.kind == ErrorKind::Napi(napi::Status::PendingException)
    }

    pub(super) fn expected_null() -> Self {
        ErrorKind::ExpectedNull.into()
    }

    pub(super) fn expected_string() -> Self {
        ErrorKind::ExpectedString.into()
    }

    pub(super) fn missing_key() -> Self {
        ErrorKind::MissingKey.into()
    }

    pub(super) fn unsupported_type(typ: napi::ValueType) -> Self {
        ErrorKind::UnsupportedType(typ).into()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) enum ErrorKind {
    // Serde
    Custom(String),
    // Serde reads and writes key/value pairs as distinct steps requiring
    // Neon to cache the intermediate key. This error is unexpected and should
    // never occur outside of a buggy serde implementation.
    MissingKey,

    // Number conversions
    FloatError(FloatError<f64>),
    I64Error(RangeError<i64>),
    U64Error(PosOverflow<u64>),
    UsizeError(PosOverflow<usize>),

    // deserialize_any
    ExpectedNull,
    ExpectedString,
    UnsupportedType(napi::ValueType),

    // N-API
    Napi(napi::Status),
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Error::new(kind)
    }
}

impl From<FloatError<f64>> for Error {
    fn from(other: FloatError<f64>) -> Self {
        ErrorKind::FloatError(other).into()
    }
}

impl From<RangeError<f64>> for Error {
    fn from(other: RangeError<f64>) -> Self {
        let err = match other {
            RangeError::PosOverflow(v) => FloatError::PosOverflow(v),
            RangeError::NegOverflow(v) => FloatError::NegOverflow(v),
        };

        err.into()
    }
}

impl From<RangeError<i64>> for Error {
    fn from(other: RangeError<i64>) -> Self {
        ErrorKind::I64Error(other).into()
    }
}

impl From<PosOverflow<u64>> for Error {
    fn from(other: PosOverflow<u64>) -> Self {
        ErrorKind::U64Error(other).into()
    }
}

impl From<PosOverflow<usize>> for Error {
    fn from(other: PosOverflow<usize>) -> Self {
        ErrorKind::UsizeError(other).into()
    }
}

impl From<napi::Status> for Error {
    fn from(other: napi::Status) -> Self {
        ErrorKind::Napi(other).into()
    }
}

impl de::Error for Error {
    fn custom<T: fmt::Display>(err: T) -> Self {
        Error {
            kind: ErrorKind::Custom(err.to_string()),
        }
    }
}

impl ser::Error for Error {
    fn custom<T: fmt::Display>(err: T) -> Self {
        de::Error::custom(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            ErrorKind::Custom(err) => f.write_str(err),
            ErrorKind::MissingKey => f.write_str("MissingKey"),
            ErrorKind::FloatError(err) => fmt::Display::fmt(err, f),
            ErrorKind::I64Error(err) => fmt::Display::fmt(err, f),
            ErrorKind::U64Error(err) => fmt::Display::fmt(err, f),
            ErrorKind::UsizeError(err) => fmt::Display::fmt(err, f),
            ErrorKind::ExpectedNull => f.write_str("ExpectedNull"),
            ErrorKind::ExpectedString => f.write_str("ExpectedString"),
            ErrorKind::UnsupportedType(typ) => write!(f, "UnsupportedType({:?})", typ),
            ErrorKind::Napi(err) => write!(f, "Napi({:?})", err),
        }
    }
}
