// Copyright 2016 Serde YAML Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

pub use yaml_rust::Yaml as Value;

use serde::ser::Serialize;
use serde::de::Deserialize;
use super::{Deserializer, Result, Serializer};

/// Shortcut function to encode a `T` into a YAML `Value`.
///
/// ```rust
/// use serde_yaml::{Value, to_value};
/// let val = to_value("foo");
/// assert_eq!(val, Value::String("foo".to_owned()))
/// ```
pub fn to_value<T: ?Sized>(value: &T) -> Value
    where T: Serialize,
{
    let mut ser = Serializer::new();
    value.serialize(&mut ser).unwrap();
    ser.take()
}

/// Shortcut function to decode a YAML `Value` into a `T`.
///
/// ```rust
/// use serde_yaml::{Value, from_value};
/// let val = Value::String("foo".to_owned());
/// assert_eq!("foo", from_value::<String>(val).unwrap());
/// ```
pub fn from_value<T>(value: Value) -> Result<T>
    where T: Deserialize,
{
    let mut de = Deserializer::new(&value);
    Deserialize::deserialize(&mut de)
}
