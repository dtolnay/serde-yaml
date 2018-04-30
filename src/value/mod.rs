// Copyright 2016 Serde YAML Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![deny(unsafe_code, missing_docs)]

use std::hash::{Hash, Hasher};

use serde::de::{Deserialize, DeserializeOwned};
use serde::Serialize;
use yaml_rust::Yaml;

use error::Error;
use mapping::Mapping;
use ser::Serializer;

use self::index::Index;
pub use number::Number;

/// Represents any valid YAML value.
#[derive(Clone, PartialOrd, Debug)]
pub enum Value {
    /// Represents a YAML null value.
    Null,
    /// Represents a YAML boolean.
    Bool(bool),
    /// Represents a YAML numerical value, whether integer or floating point.
    Number(Number),
    /// Represents a YAML string.
    String(String),
    /// Represents a YAML sequence in which the elements are
    /// `serde_yaml::Value`.
    Sequence(Sequence),
    /// Represents a YAML mapping in which the keys and values are both
    /// `serde_yaml::Value`.
    Mapping(Mapping),
}

/// A YAML sequence in which the elements are `serde_yaml::Value`.
pub type Sequence = Vec<Value>;

/// Convert a `T` into `serde_yaml::Value` which is an enum that can represent
/// any valid YAML data.
///
/// This conversion can fail if `T`'s implementation of `Serialize` decides to
/// return an error.
///
/// ```rust
/// # use serde_yaml::Value;
/// let val = serde_yaml::to_value("s").unwrap();
/// assert_eq!(val, Value::String("s".to_owned()));
/// ```
#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
pub fn to_value<T>(value: T) -> Result<Value, Error>
where
    T: Serialize,
{
    value.serialize(Serializer).map(yaml_to_value)
}

/// Interpret a `serde_yaml::Value` as an instance of type `T`.
///
/// This conversion can fail if the structure of the Value does not match the
/// structure expected by `T`, for example if `T` is a struct type but the Value
/// contains something other than a YAML map. It can also fail if the structure
/// is correct but `T`'s implementation of `Deserialize` decides that something
/// is wrong with the data, for example required struct fields are missing from
/// the YAML map or some number is too big to fit in the expected primitive
/// type.
///
/// ```rust
/// # use serde_yaml::Value;
/// let val = Value::String("foo".to_owned());
/// let s: String = serde_yaml::from_value(val).unwrap();
/// assert_eq!("foo", s);
/// ```
pub fn from_value<T>(value: Value) -> Result<T, Error>
where
    T: DeserializeOwned,
{
    Deserialize::deserialize(value)
}

impl Value {
    /// Index into a YAML sequence or map. A string index can be used to access
    /// a value in a map, and a usize index can be used to access an element of
    /// an sequence.
    ///
    /// Returns `None` if the type of `self` does not match the type of the
    /// index, for example if the index is a string and `self` is a sequence or
    /// a number. Also returns `None` if the given key does not exist in the map
    /// or the given index is not within the bounds of the sequence.
    ///
    /// ```rust
    /// # extern crate serde_yaml;
    /// # use serde_yaml::Value;
    /// #
    /// # fn yaml(i: &str) -> serde_yaml::Value { serde_yaml::from_str(i).unwrap() }
    /// # fn main() {
    /// let object: Value = yaml(r#"{ A: 65, B: 66, C: 67 }"#);
    /// let x = object.get("A").unwrap();
    /// assert_eq!(x, 65);
    ///
    /// let sequence: Value = yaml(r#"[ "A", "B", "C" ]"#);
    /// let x = sequence.get(2).unwrap();
    /// assert_eq!(x, &Value::String("C".into()));
    ///
    /// assert_eq!(sequence.get("A"), None);
    /// # }
    /// ```
    ///
    /// Square brackets can also be used to index into a value in a more concise
    /// way. This returns `Value::Null` in cases where `get` would have returned
    /// `None`.
    ///
    /// ```rust
    /// # extern crate serde_yaml;
    /// # use serde_yaml::Value;
    /// #
    /// # fn yaml(i: &str) -> serde_yaml::Value { serde_yaml::from_str(i).unwrap() }
    /// # fn main() {
    /// let object = yaml(r#"
    /// ---
    /// A: [a, á, à]
    /// B: [b, b́]
    /// C: [c, ć, ć̣, ḉ]
    /// 42: true
    /// "#);
    /// assert_eq!(object["B"][0], Value::String("b".into()));
    ///
    /// assert_eq!(object[Value::String("D".into())], Value::Null);
    /// assert_eq!(object["D"], Value::Null);
    /// assert_eq!(object[0]["x"]["y"]["z"], Value::Null);
    ///
    /// assert_eq!(object[42], Value::Bool(true));
    /// # }
    /// ```
    pub fn get<I: Index>(&self, index: I) -> Option<&Value> {
        index.index_into(self)
    }

    /// Returns true if the `Value` is a Null. Returns false otherwise.
    ///
    /// For any Value on which `is_null` returns true, `as_null` is guaranteed
    /// to return `Some(())`.
    ///
    /// ```rust
    /// # use serde_yaml::Value;
    /// let v: Value = serde_yaml::from_str("null").unwrap();
    /// assert!(v.is_null());
    /// ```
    ///
    /// ```rust
    /// # use serde_yaml::Value;
    /// let v: Value = serde_yaml::from_str("false").unwrap();
    /// assert!(!v.is_null());
    /// ```
    pub fn is_null(&self) -> bool {
        if let Value::Null = *self {
            true
        } else {
            false
        }
    }

    /// If the `Value` is a Null, returns (). Returns None otherwise.
    ///
    /// ```rust
    /// # use serde_yaml::Value;
    /// let v: Value = serde_yaml::from_str("null").unwrap();
    /// assert_eq!(v.as_null(), Some(()));
    /// ```
    ///
    /// ```rust
    /// # use serde_yaml::Value;
    /// let v: Value = serde_yaml::from_str("false").unwrap();
    /// assert_eq!(v.as_null(), None);
    /// ```
    pub fn as_null(&self) -> Option<()> {
        match *self {
            Value::Null => Some(()),
            _ => None,
        }
    }

    /// Returns true if the `Value` is a Boolean. Returns false otherwise.
    ///
    /// For any Value on which `is_boolean` returns true, `as_bool` is
    /// guaranteed to return the boolean value.
    ///
    /// ```rust
    /// # use serde_yaml::Value;
    /// let v: Value = serde_yaml::from_str("true").unwrap();
    /// assert!(v.is_bool());
    /// ```
    ///
    /// ```rust
    /// # use serde_yaml::Value;
    /// let v: Value = serde_yaml::from_str("42").unwrap();
    /// assert!(!v.is_bool());
    /// ```
    pub fn is_bool(&self) -> bool {
        self.as_bool().is_some()
    }

    /// If the `Value` is a Boolean, returns the associated bool. Returns None
    /// otherwise.
    ///
    /// ```rust
    /// # use serde_yaml::Value;
    /// let v: Value = serde_yaml::from_str("true").unwrap();
    /// assert_eq!(v.as_bool(), Some(true));
    /// ```
    ///
    /// ```rust
    /// # use serde_yaml::Value;
    /// let v: Value = serde_yaml::from_str("42").unwrap();
    /// assert_eq!(v.as_bool(), None);
    /// ```
    pub fn as_bool(&self) -> Option<bool> {
        match *self {
            Value::Bool(b) => Some(b),
            _ => None,
        }
    }

    /// Returns true if the `Value` is a Number. Returns false otherwise.
    ///
    /// ```rust
    /// # use serde_yaml::Value;
    /// let v: Value = serde_yaml::from_str("5").unwrap();
    /// assert!(v.is_number());
    /// ```
    ///
    /// ```rust
    /// # use serde_yaml::Value;
    /// let v: Value = serde_yaml::from_str("true").unwrap();
    /// assert!(!v.is_number());
    /// ```
    pub fn is_number(&self) -> bool {
        match *self {
            Value::Number(_) => true,
            _ => false,
        }
    }

    /// Returns true if the `Value` is an integer between `i64::MIN` and
    /// `i64::MAX`.
    ///
    /// For any Value on which `is_i64` returns true, `as_i64` is guaranteed to
    /// return the integer value.
    ///
    /// ```rust
    /// # use serde_yaml::Value;
    /// let v: Value = serde_yaml::from_str("1337").unwrap();
    /// assert!(v.is_i64());
    /// ```
    ///
    /// ```rust
    /// # use serde_yaml::Value;
    /// let v: Value = serde_yaml::from_str("null").unwrap();
    /// assert!(!v.is_i64());
    /// ```
    pub fn is_i64(&self) -> bool {
        self.as_i64().is_some()
    }

    /// If the `Value` is an integer, represent it as i64 if possible. Returns
    /// None otherwise.
    ///
    /// ```rust
    /// # use serde_yaml::Value;
    /// let v: Value = serde_yaml::from_str("1337").unwrap();
    /// assert_eq!(v.as_i64(), Some(1337));
    /// ```
    ///
    /// ```rust
    /// # use serde_yaml::Value;
    /// let v: Value = serde_yaml::from_str("false").unwrap();
    /// assert_eq!(v.as_i64(), None);
    /// ```
    pub fn as_i64(&self) -> Option<i64> {
        match *self {
            Value::Number(ref n) => n.as_i64(),
            _ => None,
        }
    }

    /// Returns true if the `Value` is an integer between `u64::MIN` and
    /// `u64::MAX`.
    ///
    /// For any Value on which `is_u64` returns true, `as_u64` is guaranteed to
    /// return the integer value.
    ///
    /// ```rust
    /// # use serde_yaml::Value;
    /// let v: Value = serde_yaml::from_str("1337").unwrap();
    /// assert!(v.is_u64());
    /// ```
    ///
    /// ```rust
    /// # use serde_yaml::Value;
    /// let v: Value = serde_yaml::from_str("null").unwrap();
    /// assert!(!v.is_u64());
    /// ```
    pub fn is_u64(&self) -> bool {
        self.as_u64().is_some()
    }

    /// If the `Value` is an integer, represent it as i64 if possible. Returns
    /// None otherwise.
    ///
    /// ```rust
    /// # use serde_yaml::Value;
    /// let v: Value = serde_yaml::from_str("1337").unwrap();
    /// assert_eq!(v.as_u64(), Some(1337));
    /// ```
    ///
    /// ```rust
    /// # use serde_yaml::Value;
    /// let v: Value = serde_yaml::from_str("false").unwrap();
    /// assert_eq!(v.as_u64(), None);
    /// ```
    pub fn as_u64(&self) -> Option<u64> {
        match *self {
            Value::Number(ref n) => n.as_u64(),
            _ => None,
        }
    }

    /// Returns true if the `Value` is a number that can be represented by f64.
    ///
    /// For any Value on which `is_f64` returns true, `as_f64` is guaranteed to
    /// return the floating point value.
    ///
    /// Currently this function returns true if and only if both `is_i64` and
    /// `is_u64` return false but this is not a guarantee in the future.
    ///
    /// ```rust
    /// # use serde_yaml::Value;
    /// let v: Value = serde_yaml::from_str("256.01").unwrap();
    /// assert!(v.is_f64());
    /// ```
    ///
    /// ```rust
    /// # use serde_yaml::Value;
    /// let v: Value = serde_yaml::from_str("true").unwrap();
    /// assert!(!v.is_f64());
    /// ```
    pub fn is_f64(&self) -> bool {
        match *self {
            Value::Number(ref n) => n.is_f64(),
            _ => false,
        }
    }

    /// If the `Value` is a number, represent it as f64 if possible. Returns
    /// None otherwise.
    ///
    /// ```rust
    /// # use serde_yaml::Value;
    /// let v: Value = serde_yaml::from_str("13.37").unwrap();
    /// assert_eq!(v.as_f64(), Some(13.37));
    /// ```
    ///
    /// ```rust
    /// # use serde_yaml::Value;
    /// let v: Value = serde_yaml::from_str("false").unwrap();
    /// assert_eq!(v.as_f64(), None);
    /// ```
    pub fn as_f64(&self) -> Option<f64> {
        match *self {
            Value::Number(ref i) => i.as_f64(),
            _ => None,
        }
    }

    /// Returns true if the `Value` is a String. Returns false otherwise.
    ///
    /// For any Value on which `is_string` returns true, `as_str` is guaranteed
    /// to return the string slice.
    ///
    /// ```rust
    /// # use serde_yaml::Value;
    /// let v: Value = serde_yaml::from_str("'lorem ipsum'").unwrap();
    /// assert!(v.is_string());
    /// ```
    ///
    /// ```rust
    /// # use serde_yaml::Value;
    /// let v: Value = serde_yaml::from_str("42").unwrap();
    /// assert!(!v.is_string());
    /// ```
    pub fn is_string(&self) -> bool {
        self.as_str().is_some()
    }

    /// If the `Value` is a String, returns the associated str. Returns None
    /// otherwise.
    ///
    /// ```rust
    /// # use serde_yaml::Value;
    /// let v: Value = serde_yaml::from_str("'lorem ipsum'").unwrap();
    /// assert_eq!(v.as_str(), Some("lorem ipsum"));
    /// ```
    ///
    /// ```rust
    /// # use serde_yaml::Value;
    /// let v: Value = serde_yaml::from_str("false").unwrap();
    /// assert_eq!(v.as_str(), None);
    /// ```
    pub fn as_str(&self) -> Option<&str> {
        match *self {
            Value::String(ref s) => Some(s),
            _ => None,
        }
    }

    /// Returns true if the `Value` is a sequence. Returns false otherwise.
    ///
    /// ```rust
    /// # use serde_yaml::Value;
    /// let v: Value = serde_yaml::from_str("[1, 2, 3]").unwrap();
    /// assert!(v.is_sequence());
    /// ```
    ///
    /// ```rust
    /// # use serde_yaml::Value;
    /// let v: Value = serde_yaml::from_str("true").unwrap();
    /// assert!(!v.is_sequence());
    /// ```
    pub fn is_sequence(&self) -> bool {
        self.as_sequence().is_some()
    }

    /// If the `Value` is a sequence, return a reference to it if possible.
    /// Returns None otherwise.
    ///
    /// ```rust
    /// # use serde_yaml::{Value, Number};
    /// let v: Value = serde_yaml::from_str("[1, 2]").unwrap();
    /// assert_eq!(v.as_sequence(), Some(&vec![Value::Number(Number::from(1)), Value::Number(Number::from(2))]));
    /// ```
    ///
    /// ```rust
    /// # use serde_yaml::Value;
    /// let v: Value = serde_yaml::from_str("false").unwrap();
    /// assert_eq!(v.as_sequence(), None);
    /// ```
    pub fn as_sequence(&self) -> Option<&Sequence> {
        match *self {
            Value::Sequence(ref seq) => Some(seq),
            _ => None,
        }
    }

    /// If the `Value` is a sequence, return a mutable reference to it if
    /// possible. Returns None otherwise.
    ///
    /// ```rust
    /// # use serde_yaml::{Value, Number};
    /// let mut v: Value = serde_yaml::from_str("[1]").unwrap();
    /// let s = v.as_sequence_mut().unwrap();
    /// s.push(Value::Number(Number::from(2)));
    /// assert_eq!(s, &vec![Value::Number(Number::from(1)), Value::Number(Number::from(2))]);
    /// ```
    ///
    /// ```rust
    /// # use serde_yaml::Value;
    /// let mut v: Value = serde_yaml::from_str("false").unwrap();
    /// assert_eq!(v.as_sequence_mut(), None);
    /// ```
    pub fn as_sequence_mut(&mut self) -> Option<&mut Sequence> {
        match *self {
            Value::Sequence(ref mut seq) => Some(seq),
            _ => None,
        }
    }

    /// Returns true if the `Value` is a mapping. Returns false otherwise.
    ///
    /// ```rust
    /// # use serde_yaml::Value;
    /// let v: Value = serde_yaml::from_str("a: 42").unwrap();
    /// assert!(v.is_mapping());
    /// ```
    ///
    /// ```rust
    /// # use serde_yaml::Value;
    /// let v: Value = serde_yaml::from_str("true").unwrap();
    /// assert!(!v.is_mapping());
    /// ```
    pub fn is_mapping(&self) -> bool {
        self.as_mapping().is_some()
    }

    /// If the `Value` is a mapping, return a reference to it if possible.
    /// Returns None otherwise.
    ///
    /// ```rust
    /// # use serde_yaml::{Value, Mapping, Number};
    /// let v: Value = serde_yaml::from_str("a: 42").unwrap();
    ///
    /// let mut expected = Mapping::new();
    /// expected.insert(Value::String("a".into()),Value::Number(Number::from(42)));
    ///
    /// assert_eq!(v.as_mapping(), Some(&expected));
    /// ```
    ///
    /// ```rust
    /// # use serde_yaml::Value;
    /// let v: Value = serde_yaml::from_str("false").unwrap();
    /// assert_eq!(v.as_mapping(), None);
    /// ```
    pub fn as_mapping(&self) -> Option<&Mapping> {
        match *self {
            Value::Mapping(ref map) => Some(map),
            _ => None,
        }
    }

    /// If the `Value` is a mapping, return a reference to it if possible.
    /// Returns None otherwise.
    ///
    /// ```rust
    /// # use serde_yaml::{Value, Mapping, Number};
    /// let mut v: Value = serde_yaml::from_str("a: 42").unwrap();
    /// let m = v.as_mapping_mut().unwrap();
    /// m.insert(Value::String("b".into()),Value::Number(Number::from(21)));
    ///
    /// let mut expected = Mapping::new();
    /// expected.insert(Value::String("a".into()),Value::Number(Number::from(42)));
    /// expected.insert(Value::String("b".into()),Value::Number(Number::from(21)));
    ///
    /// assert_eq!(m, &expected);
    /// ```
    ///
    /// ```rust
    /// # use serde_yaml::{Value, Mapping};
    /// let mut v: Value = serde_yaml::from_str("false").unwrap();
    /// assert_eq!(v.as_mapping_mut(), None);
    /// ```
    pub fn as_mapping_mut(&mut self) -> Option<&mut Mapping> {
        match *self {
            Value::Mapping(ref mut map) => Some(map),
            _ => None,
        }
    }
}

fn yaml_to_value(yaml: Yaml) -> Value {
    match yaml {
        Yaml::Real(f) => match f.parse::<f64>() {
            Ok(f) => Value::Number(f.into()),
            Err(_) => Value::String(f),
        },
        Yaml::Integer(i) => Value::Number(i.into()),
        Yaml::String(s) => Value::String(s),
        Yaml::Boolean(b) => Value::Bool(b),
        Yaml::Array(sequence) => Value::Sequence(sequence.into_iter().map(yaml_to_value).collect()),
        Yaml::Hash(hash) => Value::Mapping(
            hash.into_iter()
                .map(|(k, v)| (yaml_to_value(k), yaml_to_value(v)))
                .collect(),
        ),
        Yaml::Alias(_) => panic!("alias unsupported"),
        Yaml::Null => Value::Null,
        Yaml::BadValue => panic!("bad value"),
    }
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match *self {
            Value::Null => 0.hash(state),
            Value::Bool(b) => (1, b).hash(state),
            Value::Number(ref i) => (2, i).hash(state),
            Value::String(ref s) => (3, s).hash(state),
            Value::Sequence(ref seq) => (4, seq).hash(state),
            Value::Mapping(ref map) => (5, map).hash(state),
        }
    }
}

mod from;
mod index;
mod partial_eq;

mod de;
mod ser;
