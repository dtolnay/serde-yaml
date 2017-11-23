// Copyright 2016 Serde YAML Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::error;
use std::fmt::{self, Display, Debug};
use std::io;
use std::result;
use std::str;
use std::string;

use yaml_rust::emitter;
use yaml_rust::scanner::{self, Marker, ScanError};

use serde::{de, ser};

use path::Path;

/// This type represents all possible errors that can occur when serializing or
/// deserializing YAML data.
pub struct Error(Box<ErrorImpl>);

/// Alias for a `Result` with the error type `serde_yaml::Error`.
pub type Result<T> = result::Result<T, Error>;

/// This type represents all possible errors that can occur when serializing or
/// deserializing a value using YAML.
#[derive(Debug)]
pub enum ErrorImpl {
    Message(String, Option<Pos>),

    Emit(emitter::EmitError),
    Scan(scanner::ScanError),
    Io(io::Error),
    Utf8(str::Utf8Error),
    FromUtf8(string::FromUtf8Error),

    EndOfStream,
    MoreThanOneDocument,
}

#[derive(Debug)]
pub struct Pos {
    marker: Marker,
    path: String,
}


/// This type represents the location that an error occured
#[derive(Debug)]
pub struct ErrorMarker {
    index: usize,
    line: usize,
    col: usize,
} 

impl ErrorMarker {
    /// The byte index of the error
    pub fn index(&self) -> usize {
        self.index
    }

    /// The line of the error
    pub fn line(&self) -> usize {
        self.line
    }

    /// The column of the error
    pub fn col(&self) -> usize {
        self.col
    }

    fn from_marker(marker: &Marker) -> Self {
        ErrorMarker {
            col: marker.col(),
            index: marker.index(),
            line: marker.line(),
        }
    }
}
  
impl Error {
    /// Returns the ErrorMarker from the error if one exists.
    /// Allows for the exact location of the error to be gathered.
    /// Not all errors have a location, so this will return `None`
    /// # Examples
    ///
    /// ```rust
    /// # extern crate serde_yaml;
    /// # use serde_yaml::{Value, Error};
    ///
    /// # fn yaml(i: &str) -> Result<Value, Error> { serde_yaml::from_str(i) }
    /// # fn main() {
    /// # 
    /// // The `@` character as the first character makes this invalid yaml
    /// let result: Result<Value, Error> = yaml(r#"@invalid_yaml"#);
    /// 
    /// let marker = result.unwrap_err().marker().unwrap();
    ///
    /// assert_eq!(marker.line(), 1);
    /// assert_eq!(marker.col(), 0);
    ///
    ///
    /// # }
    /// ```
    pub fn marker(&self) -> Option<ErrorMarker> {
        match *self.0 {
            ErrorImpl::Message(_, Some(ref pos)) => Some(ErrorMarker::from_marker(&pos.marker)),
            ErrorImpl::Scan(ref scan) => Some(ErrorMarker::from_marker(scan.marker())),
            _ => None
        }
    }

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
    pub fn io(err: io::Error) -> Error {
        Error(Box::new(ErrorImpl::Io(err)))
    }

    // Not public API. Should be pub(crate).
    #[doc(hidden)]
    pub fn emitter(err: emitter::EmitError) -> Error {
        Error(Box::new(ErrorImpl::Emit(err)))
    }

    // Not public API. Should be pub(crate).
    #[doc(hidden)]
    pub fn scanner(err: scanner::ScanError) -> Error {
        Error(Box::new(ErrorImpl::Scan(err)))
    }

    // Not public API. Should be pub(crate).
    #[doc(hidden)]
    pub fn str_utf8(err: str::Utf8Error) -> Error {
        Error(Box::new(ErrorImpl::Utf8(err)))
    }

    // Not public API. Should be pub(crate).
    #[doc(hidden)]
    pub fn string_utf8(err: string::FromUtf8Error) -> Error {
        Error(Box::new(ErrorImpl::FromUtf8(err)))
    }

    // Not public API. Should be pub(crate).
    #[doc(hidden)]
    pub fn fix_marker(mut self, marker: Marker, path: Path) -> Self {
        if let ErrorImpl::Message(_, ref mut none @ None) = *self.0.as_mut() {
            *none = Some(Pos {
                             marker: marker,
                             path: path.to_string(),
                         });
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
            ErrorImpl::MoreThanOneDocument => "deserializing from YAML containing more than one document is not supported",
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

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self.0 {
            ErrorImpl::Message(ref msg, None) => Display::fmt(msg, f),
            ErrorImpl::Message(ref msg, Some(Pos { marker, ref path })) => {
                if path == "." {
                    write!(f, "{}", ScanError::new(marker, msg))
                } else {
                    write!(f, "{}: {}", path, ScanError::new(marker, msg))
                }
            }
            ErrorImpl::Emit(emitter::EmitError::FmtError(_)) => f.write_str("yaml-rust fmt error"),
            ErrorImpl::Emit(emitter::EmitError::BadHashmapKey) => f.write_str("bad hash map key"),
            ErrorImpl::Scan(ref err) => Display::fmt(err, f),
            ErrorImpl::Io(ref err) => Display::fmt(err, f),
            ErrorImpl::Utf8(ref err) => Display::fmt(err, f),
            ErrorImpl::FromUtf8(ref err) => Display::fmt(err, f),
            ErrorImpl::EndOfStream => f.write_str("EOF while parsing a value"),
            ErrorImpl::MoreThanOneDocument => {
                f.write_str("deserializing from YAML containing more than one document is not supported")
            }
        }
    }
}

// Remove two layers of verbosity from the debug representation. Humans often
// end up seeing this representation because it is what unwrap() shows.
impl Debug for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self.0 {
            ErrorImpl::Message(ref msg, ref pos) => {
                formatter.debug_tuple("Message")
                    .field(msg)
                    .field(pos)
                    .finish()
            }
            ErrorImpl::Emit(ref emit) => formatter.debug_tuple("Emit").field(emit).finish(),
            ErrorImpl::Scan(ref scan) => formatter.debug_tuple("Scan").field(scan).finish(),
            ErrorImpl::Io(ref io) => formatter.debug_tuple("Io").field(io).finish(),
            ErrorImpl::Utf8(ref utf8) => formatter.debug_tuple("Utf8").field(utf8).finish(),
            ErrorImpl::FromUtf8(ref from_utf8) => {
                formatter.debug_tuple("FromUtf8").field(from_utf8).finish()
            }
            ErrorImpl::EndOfStream => formatter.debug_tuple("EndOfStream").finish(),
            ErrorImpl::MoreThanOneDocument => formatter.debug_tuple("MoreThanOneDocument").finish(),
        }
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
