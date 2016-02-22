use std::fmt;
use std::result;
use std::string::FromUtf8Error;

use yaml_rust::{emitter, scanner};

use serde::de;

/// This type represents all possible errors that can occur when serializing or
/// deserializing a value using YAML.
pub enum Error {
    Syntax(String),
    EndOfStream,
    UnknownField(String),
    MissingField(&'static str),

    EmitError(emitter::EmitError),
    ScanError(scanner::ScanError),
    FromUtf8Error(FromUtf8Error),

    AliasUnsupported,
    TooManyDocuments(usize),
    VariantMapWrongSize(String, usize),
    VariantNotAMap(String),
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Syntax(ref msg) =>
                write!(f, "syntax error: {}", msg),
            Error::EndOfStream =>
                write!(f, "EOF while parsing a value"),
            Error::UnknownField(ref field) =>
                write!(f, "unknown field \"{}\"", field),
            Error::MissingField(ref field) =>
                write!(f, "missing field \"{}\"", field),
            Error::EmitError(ref err) => err.fmt(f),
            Error::ScanError(ref err) => err.fmt(f),
            Error::FromUtf8Error(ref err) => err.fmt(f),
            Error::AliasUnsupported =>
                write!(f, "YAML aliases are not supported"),
            Error::TooManyDocuments(n) =>
                write!(f, "expected a single YAML document but found {}", n),
            Error::VariantMapWrongSize(ref variant, size) =>
                write!(f, "expected a YAML map of size 1 while parsing variant {} but was size {}", variant, size),
            Error::VariantNotAMap(ref variant) =>
                write!(f, "expected a YAML map while parsing variant {}", variant),
        }
    }
}

impl From<emitter::EmitError> for Error {
    fn from(error: emitter::EmitError) -> Error {
        Error::EmitError(error)
    }
}

impl From<scanner::ScanError> for Error {
    fn from(error: scanner::ScanError) -> Error {
        Error::ScanError(error)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(error: FromUtf8Error) -> Error {
        Error::FromUtf8Error(error)
    }
}

impl de::Error for Error {
    fn syntax(msg: &str) -> Error {
        Error::Syntax(String::from(msg))
    }

    fn end_of_stream() -> Error {
        Error::EndOfStream
    }

    fn unknown_field(field: &str) -> Error {
        Error::UnknownField(String::from(field))
    }

    fn missing_field(field: &'static str) -> Error {
        Error::MissingField(field)
    }
}

/// Helper alias for `Result` objects that return a YAML `Error`.
pub type Result<T> = result::Result<T, Error>;
