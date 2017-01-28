// Copyright 2016 Serde YAML Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::error;
use std::fmt::{self, Display};
use std::io;
use std::result;
use std::str;
use std::string;

use yaml_rust::emitter;
use yaml_rust::scanner::{self, Marker, ScanError};

use serde::{de, ser};

/// This type represents all possible errors that can occur when serializing or
/// deserializing a value using YAML.
#[derive(Debug)]
pub enum Error {
    Custom(String, Option<Marker>),

    Emit(emitter::EmitError),
    Scan(scanner::ScanError),
    Io(io::Error),
    Utf8(str::Utf8Error),
    FromUtf8(string::FromUtf8Error),

    EndOfStream,
    MoreThanOneDocument,
}

impl Error {
    // Not public API. Should be pub(crate).
    #[doc(hidden)]
    pub fn fix_marker(self, marker: Marker) -> Self {
        if let Error::Custom(msg, None) = self {
            Error::Custom(msg, Some(marker))
        } else {
            self
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Custom(ref msg, _) => msg,
            Error::Emit(_) => "emit error",
            Error::Scan(_) => "scan error",
            Error::Io(ref err) => err.description(),
            Error::Utf8(ref err) => err.description(),
            Error::FromUtf8(ref err) => err.description(),
            Error::EndOfStream => "EOF while parsing a value",
            Error::MoreThanOneDocument => {
                "deserializing from YAML containing more than one document is not supported"
            }
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Scan(ref err) => Some(err),
            Error::Io(ref err) => Some(err),
            Error::Utf8(ref err) => Some(err),
            Error::FromUtf8(ref err) => Some(err),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Custom(ref msg, None) => write!(f, "{}", msg),
            Error::Custom(ref msg, Some(marker)) => {
                write!(f, "{}", ScanError::new(marker, msg))
            }
            Error::Emit(ref err) => write!(f, "{:?}", err),
            Error::Scan(ref err) => err.fmt(f),
            Error::Io(ref err) => err.fmt(f),
            Error::Utf8(ref err) => err.fmt(f),
            Error::FromUtf8(ref err) => err.fmt(f),
            Error::EndOfStream => write!(f, "EOF while parsing a value"),
            Error::MoreThanOneDocument => {
                write!(f, "deserializing from YAML containing more than one document is not supported")
            }
        }
    }
}

impl From<emitter::EmitError> for Error {
    fn from(err: emitter::EmitError) -> Error {
        Error::Emit(err)
    }
}

impl From<scanner::ScanError> for Error {
    fn from(err: scanner::ScanError) -> Error {
        Error::Scan(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<str::Utf8Error> for Error {
    fn from(err: str::Utf8Error) -> Error {
        Error::Utf8(err)
    }
}

impl From<string::FromUtf8Error> for Error {
    fn from(err: string::FromUtf8Error) -> Error {
        Error::FromUtf8(err)
    }
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Custom(msg.to_string(), None)
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Custom(msg.to_string(), None)
    }
}

/// Helper alias for `Result` objects that return a YAML `Error`.
pub type Result<T> = result::Result<T, Error>;
