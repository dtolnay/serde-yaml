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
/// deserializing YAML data.
pub struct Error(Box<ErrorImpl>);

/// Alias for a `Result` with the error type `serde_yaml::Error`.
pub type Result<T> = result::Result<T, Error>;

/// This type represents all possible errors that can occur when serializing or
/// deserializing a value using YAML.
#[derive(Debug)]
pub enum ErrorImpl {
    Message(String, Option<Marker>),

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
    pub fn end_of_stream() -> Self {
        Error(Box::new(ErrorImpl::EndOfStream))
    }

    // Not public API. Should be pub(crate).
    #[doc(hidden)]
    pub fn more_than_one_document() -> Self {
        Error(Box::new(ErrorImpl::MoreThanOneDocument))
    }

    // Not public API. Should be pub(crate).
    #[doc(hidden)]
    pub fn fix_marker(mut self, marker: Marker) -> Self {
        if let ErrorImpl::Message(_, ref mut none @ None) = *self.0.as_mut() {
            *none = Some(marker);
        }
        self
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self.0 {
            ErrorImpl::Message(ref msg, _) => msg,
            ErrorImpl::Emit(_) => "emit error",
            ErrorImpl::Scan(_) => "scan error",
            ErrorImpl::Io(ref err) => err.description(),
            ErrorImpl::Utf8(ref err) => err.description(),
            ErrorImpl::FromUtf8(ref err) => err.description(),
            ErrorImpl::EndOfStream => "EOF while parsing a value",
            ErrorImpl::MoreThanOneDocument => {
                "deserializing from YAML containing more than one document is not supported"
            }
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self.0 {
            ErrorImpl::Scan(ref err) => Some(err),
            ErrorImpl::Io(ref err) => Some(err),
            ErrorImpl::Utf8(ref err) => Some(err),
            ErrorImpl::FromUtf8(ref err) => Some(err),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self.0 {
            ErrorImpl::Message(ref msg, None) => write!(f, "{}", msg),
            ErrorImpl::Message(ref msg, Some(marker)) => {
                write!(f, "{}", ScanError::new(marker, msg))
            }
            ErrorImpl::Emit(ref err) => write!(f, "{:?}", err),
            ErrorImpl::Scan(ref err) => err.fmt(f),
            ErrorImpl::Io(ref err) => err.fmt(f),
            ErrorImpl::Utf8(ref err) => err.fmt(f),
            ErrorImpl::FromUtf8(ref err) => err.fmt(f),
            ErrorImpl::EndOfStream => write!(f, "EOF while parsing a value"),
            ErrorImpl::MoreThanOneDocument => {
                write!(f, "deserializing from YAML containing more than one document is not supported")
            }
        }
    }
}

// Remove two layers of verbosity from the debug representation. Humans often
// end up seeing this representation because it is what unwrap() shows.
impl fmt::Debug for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self.0 {
            ErrorImpl::Message(ref msg, ref marker) => {
                formatter.debug_tuple("Message")
                    .field(msg)
                    .field(marker)
                    .finish()
            }
            ErrorImpl::Emit(ref emit) => {
                formatter.debug_tuple("Emit")
                    .field(emit)
                    .finish()
            }
            ErrorImpl::Scan(ref scan) => {
                formatter.debug_tuple("Scan")
                    .field(scan)
                    .finish()
            }
            ErrorImpl::Io(ref io) => {
                formatter.debug_tuple("Io")
                    .field(io)
                    .finish()
            }
            ErrorImpl::Utf8(ref utf8) => {
                formatter.debug_tuple("Utf8")
                    .field(utf8)
                    .finish()
            }
            ErrorImpl::FromUtf8(ref from_utf8) => {
                formatter.debug_tuple("FromUtf8")
                    .field(from_utf8)
                    .finish()
            }
            ErrorImpl::EndOfStream => {
                formatter.debug_tuple("EndOfStream")
                    .finish()
            }
            ErrorImpl::MoreThanOneDocument => {
                formatter.debug_tuple("MoreThanOneDocument")
                    .finish()
            }
        }
    }
}

impl From<emitter::EmitError> for Error {
    fn from(err: emitter::EmitError) -> Error {
        Error(Box::new(ErrorImpl::Emit(err)))
    }
}

impl From<scanner::ScanError> for Error {
    fn from(err: scanner::ScanError) -> Error {
        Error(Box::new(ErrorImpl::Scan(err)))
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error(Box::new(ErrorImpl::Io(err)))
    }
}

impl From<str::Utf8Error> for Error {
    fn from(err: str::Utf8Error) -> Error {
        Error(Box::new(ErrorImpl::Utf8(err)))
    }
}

impl From<string::FromUtf8Error> for Error {
    fn from(err: string::FromUtf8Error) -> Error {
        Error(Box::new(ErrorImpl::FromUtf8(err)))
    }
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error(Box::new(ErrorImpl::Message(msg.to_string(), None)))
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error(Box::new(ErrorImpl::Message(msg.to_string(), None)))
    }
}
