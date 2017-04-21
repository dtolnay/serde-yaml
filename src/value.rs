// Copyright 2016 Serde YAML Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
#![deny(unsafe_code, missing_docs)]

use std::fmt;
use std::hash::{Hash, Hasher};
use std::mem;
use std::vec;
use std::ops;

use num_traits::NumCast;
use serde::{self, Serialize};
use serde::de::{Deserialize, DeserializeOwned, Deserializer, Unexpected, Visitor};
use yaml_rust::Yaml;

use error::Error;
use mapping::Mapping;
use ser::Serializer;

/// Represents any valid YAML value.
#[derive(Clone, PartialOrd, Debug)]
pub enum Value {
    /// Represents a YAML null value.
    Null,
    /// Represents a YAML boolean.
    Bool(bool),
    /// Represents a YAML integer value.
    I64(i64),
    /// Represents a YAML floating-point value.
    F64(f64),
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
    where T: Serialize
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
    where T: DeserializeOwned
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
    /// assert_eq!(x, &Value::I64(65));
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
    /// assert_eq!(object[Value::I64(42)], Value::Bool(true));
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
        if let Value::Null = *self { true } else { false }
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
            Value::I64(i) => Some(i),
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
        self.as_f64().is_some()
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
            Value::F64(i) => Some(i),
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
    /// # use serde_yaml::Value;
    /// let v: Value = serde_yaml::from_str("[1, 2]").unwrap();
    /// assert_eq!(v.as_sequence(), Some(&vec![Value::I64(1), Value::I64(2)]));
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
    /// # use serde_yaml::Value;
    /// let mut v: Value = serde_yaml::from_str("[1]").unwrap();
    /// let s = v.as_sequence_mut().unwrap();
    /// s.push(Value::I64(2));
    /// assert_eq!(s, &vec![Value::I64(1), Value::I64(2)]);
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
    /// # use serde_yaml::{Value, Mapping};
    /// let v: Value = serde_yaml::from_str("a: 42").unwrap();
    ///
    /// let mut expected = Mapping::new();
    /// expected.insert(Value::String("a".into()),Value::I64(42));
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
    /// # use serde_yaml::{Value, Mapping};
    /// let mut v: Value = serde_yaml::from_str("a: 42").unwrap();
    /// let m = v.as_mapping_mut().unwrap();
    /// m.insert(Value::String("b".into()),Value::I64(21));
    ///
    /// let mut expected = Mapping::new();
    /// expected.insert(Value::String("a".into()),Value::I64(42));
    /// expected.insert(Value::String("b".into()),Value::I64(21));
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
        Yaml::Real(f) => {
            match f.parse() {
                Ok(f) => Value::F64(f),
                Err(_) => Value::String(f),
            }
        }
        Yaml::Integer(i) => Value::I64(i),
        Yaml::String(s) => Value::String(s),
        Yaml::Boolean(b) => Value::Bool(b),
        Yaml::Array(sequence) => Value::Sequence(sequence.into_iter().map(yaml_to_value).collect()),
        Yaml::Hash(hash) => {
            Value::Mapping(hash.into_iter().map(|(k, v)| (yaml_to_value(k), yaml_to_value(v))).collect())
        }
        Yaml::Alias(_) => panic!("alias unsupported"),
        Yaml::Null => Value::Null,
        Yaml::BadValue => panic!("bad value"),
    }
}

impl From<Value> for Yaml {
    fn from(yaml: Value) -> Self {
        match yaml {
            Value::F64(f) => Yaml::Real(format!("{}", f)),
            Value::I64(i) => Yaml::Integer(i),
            Value::String(s) => Yaml::String(s),
            Value::Bool(s) => Yaml::Boolean(s),
            Value::Sequence(seq) => Yaml::Array(seq.into_iter().map(Into::into).collect()),
            Value::Mapping(map) => {
                Yaml::Hash(map.into_iter().map(|(k, v)| (k.into(), v.into())).collect())
            }
            Value::Null => Yaml::Null,
        }
    }
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer
    {
        match *self {
            Value::Null => serializer.serialize_unit(),
            Value::Bool(b) => serializer.serialize_bool(b),
            Value::I64(i) => serializer.serialize_i64(i),
            Value::F64(f) => serializer.serialize_f64(f),
            Value::String(ref s) => serializer.serialize_str(s),
            Value::Sequence(ref seq) => seq.serialize(serializer),
            Value::Mapping(ref hash) => {
                use serde::ser::SerializeMap;
                let mut map = serializer.serialize_map(Some(hash.len()))?;
                for (k, v) in hash {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de>
    {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = Value;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("any YAML value")
            }

            fn visit_bool<E>(self, b: bool) -> Result<Value, E>
                where E: serde::de::Error
            {
                Ok(Value::Bool(b))
            }

            fn visit_i64<E>(self, i: i64) -> Result<Value, E>
                where E: serde::de::Error
            {
                Ok(Value::I64(i))
            }

            fn visit_u64<E>(self, u: u64) -> Result<Value, E>
                where E: serde::de::Error
            {
                match NumCast::from(u) {
                    Some(i) => Ok(Value::I64(i)),
                    None => Ok(Value::String(u.to_string())),
                }
            }

            fn visit_f64<E>(self, f: f64) -> Result<Value, E>
                where E: serde::de::Error
            {
                Ok(Value::F64(f))
            }

            fn visit_str<E>(self, s: &str) -> Result<Value, E>
                where E: serde::de::Error
            {
                Ok(Value::String(s.to_owned()))
            }

            fn visit_string<E>(self, s: String) -> Result<Value, E>
                where E: serde::de::Error
            {
                Ok(Value::String(s))
            }

            fn visit_unit<E>(self) -> Result<Value, E>
                where E: serde::de::Error
            {
                Ok(Value::Null)
            }

            fn visit_none<E>(self) -> Result<Value, E>
                where E: serde::de::Error
            {
                Ok(Value::Null)
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Value, D::Error>
                where D: serde::Deserializer<'de>
            {
                Deserialize::deserialize(deserializer)
            }

            fn visit_seq<V>(self, mut visitor: V) -> Result<Value, V::Error>
                where V: serde::de::SeqAccess<'de>
            {
                let mut vec = Vec::new();

                while let Some(element) = visitor.next_element()? {
                    vec.push(element);
                }

                Ok(Value::Sequence(vec))
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<Value, V::Error>
                where V: serde::de::MapAccess<'de>
            {
                let mut values = Mapping::new();

                while let Some((key, value)) = visitor.next_entry()? {
                    values.insert(key, value);
                }

                Ok(Value::Mapping(values))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

impl PartialEq for Value {
    #[allow(unsafe_code)]
    fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (&Value::Null, &Value::Null) => true,
            (&Value::Bool(a), &Value::Bool(b)) => a == b,
            (&Value::I64(a), &Value::I64(b)) => a == b,
            (&Value::F64(a), &Value::F64(b)) => {
                if a.is_nan() && b.is_nan() {
                    // compare NaN for bitwise equality
                    let (a, b): (i64, i64) = unsafe { (mem::transmute(a), mem::transmute(b)) };
                    a == b
                } else {
                    a == b
                }
            }
            (&Value::String(ref a), &Value::String(ref b)) => a == b,
            (&Value::Sequence(ref a), &Value::Sequence(ref b)) => a == b,
            (&Value::Mapping(ref a), &Value::Mapping(ref b)) => a == b,
            _ => false,
        }
    }
}

impl PartialEq<str> for Value {
    /// Compare `str` with YAML value
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use serde_yaml::Value;
    /// assert!(Value::String("lorem".into()) == *"lorem");
    /// ```
    fn eq(&self, other: &str) -> bool {
        self.as_str().map_or(false, |s| s == other)
    }
}

impl<'a> PartialEq<&'a str> for Value {
    /// Compare `&str` with YAML value
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use serde_yaml::Value;
    /// assert!(Value::String("lorem".into()) == "lorem");
    /// ```
    fn eq(&self, other: &&str) -> bool {
        self.as_str().map_or(false, |s| s == *other)
    }
}

impl PartialEq<Value> for str {
    /// Compare YAML value with `str`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use serde_yaml::Value;
    /// assert!(*"lorem" == Value::String("lorem".into()));
    /// ```
    fn eq(&self, other: &Value) -> bool {
        other.as_str().map_or(false, |s| s == self)
    }
}

impl<'a> PartialEq<Value> for &'a str {
    /// Compare `&str` with YAML value
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use serde_yaml::Value;
    /// assert!("lorem" == Value::String("lorem".into()));
    /// ```
    fn eq(&self, other: &Value) -> bool {
        other.as_str().map_or(false, |s| s == *self)
    }
}

impl PartialEq<String> for Value {
    /// Compare YAML value with String
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use serde_yaml::Value;
    /// assert!(Value::String("lorem".into()) == "lorem".to_string());
    /// ```
    fn eq(&self, other: &String) -> bool {
        self.as_str().map_or(false, |s| s == other)
    }
}


impl PartialEq<Value> for String {
    /// Compare `String` with YAML value
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use serde_yaml::Value;
    /// assert!("lorem".to_string() == Value::String("lorem".into()));
    /// ```
    fn eq(&self, other: &Value) -> bool {
        other.as_str().map_or(false, |s| s == self)
    }
}

macro_rules! partialeq_numeric {
    ($([$($ty:ty)*], $conversion:ident, $base:ty)*) => {
        $($(
            impl PartialEq<$ty> for Value {
                fn eq(&self, other: &$ty) -> bool {
                    self.$conversion().map_or(false, |i| i == (*other as $base))
                }
            }

            impl PartialEq<Value> for $ty {
                fn eq(&self, other: &Value) -> bool {
                    other.$conversion().map_or(false, |i| i == (*self as $base))
                }
            }

            impl<'a> PartialEq<$ty> for &'a Value {
                fn eq(&self, other: &$ty) -> bool {
                    self.$conversion().map_or(false, |i| i == (*other as $base))
                }
            }

            impl<'a> PartialEq<$ty> for &'a mut Value {
                fn eq(&self, other: &$ty) -> bool {
                    self.$conversion().map_or(false, |i| i == (*other as $base))
                }
            }
        )*)*
    }
}

partialeq_numeric! {
    [i8 i16 i32 i64 isize], as_i64, i64
    [u8 u16 u32 usize], as_i64, i64
    [f32 f64], as_f64, f64
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match *self {
            Value::Null => 0.hash(state),
            Value::Bool(b) => (1, b).hash(state),
            Value::I64(i) => (2, i).hash(state),
            Value::F64(_) => {
                // you should feel bad for using f64 as a map key
                3.hash(state);
            }
            Value::String(ref s) => (4, s).hash(state),
            Value::Sequence(ref seq) => (5, seq).hash(state),
            Value::Mapping(ref map) => (6, map).hash(state),
        }
    }
}

impl<'de> Deserializer<'de> for Value {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
        where V: Visitor<'de>
    {
        match self {
            Value::Null => visitor.visit_unit(),
            Value::Bool(v) => visitor.visit_bool(v),
            Value::I64(i) => visitor.visit_i64(i),
            Value::F64(f) => visitor.visit_f64(f),
            Value::String(v) => visitor.visit_string(v),
            Value::Sequence(v) => {
                let len = v.len();
                let mut deserializer = SeqDeserializer::new(v);
                let seq = visitor.visit_seq(&mut deserializer)?;
                let remaining = deserializer.iter.len();
                if remaining == 0 {
                    Ok(seq)
                } else {
                    Err(serde::de::Error::invalid_length(len, &"fewer elements in sequence"))
                }
            }
            Value::Mapping(v) => {
                let len = v.len();
                let mut deserializer = MapDeserializer::new(v);
                let map = visitor.visit_map(&mut deserializer)?;
                let remaining = deserializer.iter.len();
                if remaining == 0 {
                    Ok(map)
                } else {
                    Err(serde::de::Error::invalid_length(len, &"fewer elements in map"))
                }
            }
        }
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Error>
        where V: Visitor<'de>
    {
        match self {
            Value::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    #[inline]
    fn deserialize_enum<V>(self,
                           _name: &str,
                           _variants: &'static [&'static str],
                           visitor: V)
                           -> Result<V::Value, Error>
        where V: Visitor<'de>
    {
        let (variant, value) = match self {
            Value::Mapping(value) => {
                let mut iter = value.into_iter();
                let (variant, value) = match iter.next() {
                    Some(v) => v,
                    None => {
                        return Err(serde::de::Error::invalid_value(Unexpected::Map,
                                                                   &"map with a single key"));
                    }
                };
                // enums are encoded in json as maps with a single key:value pair
                if iter.next().is_some() {
                    return Err(serde::de::Error::invalid_value(Unexpected::Map,
                                                               &"map with a single key"));
                }
                (variant, Some(value))
            }
            Value::String(variant) => (Value::String(variant), None),
            other => {
                return Err(serde::de::Error::invalid_type(other.unexpected(), &"string or map"));
            }
        };

        visitor.visit_enum(EnumDeserializer {
                               variant: variant,
                               value: value,
                           })
    }

    #[inline]
    fn deserialize_newtype_struct<V>(self,
                                     _name: &'static str,
                                     visitor: V)
                                     -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        visitor.visit_newtype_struct(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf unit unit_struct seq tuple tuple_struct map struct identifier
        ignored_any
    }
}

struct EnumDeserializer {
    variant: Value,
    value: Option<Value>,
}

impl<'de> serde::de::EnumAccess<'de> for EnumDeserializer {
    type Error = Error;
    type Variant = VariantDeserializer;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, VariantDeserializer), Error>
        where V: serde::de::DeserializeSeed<'de>
    {
        let visitor = VariantDeserializer { value: self.value };
        seed.deserialize(self.variant).map(|v| (v, visitor))
    }
}

struct VariantDeserializer {
    value: Option<Value>,
}

impl<'de> serde::de::VariantAccess<'de> for VariantDeserializer {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Error> {
        match self.value {
            Some(value) => serde::de::Deserialize::deserialize(value),
            None => Ok(()),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Error>
        where T: serde::de::DeserializeSeed<'de>
    {
        match self.value {
            Some(value) => seed.deserialize(value),
            None => {
                Err(serde::de::Error::invalid_type(Unexpected::UnitVariant, &"newtype variant"))
            }
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Error>
        where V: Visitor<'de>
    {
        match self.value {
            Some(Value::Sequence(v)) => {
                serde::de::Deserializer::deserialize_any(SeqDeserializer::new(v), visitor)
            }
            Some(other) => {
                Err(serde::de::Error::invalid_type(other.unexpected(), &"tuple variant"))
            }
            None => Err(serde::de::Error::invalid_type(Unexpected::UnitVariant, &"tuple variant")),
        }
    }

    fn struct_variant<V>(self,
                       _fields: &'static [&'static str],
                       visitor: V)
                       -> Result<V::Value, Error>
        where V: Visitor<'de>
    {
        match self.value {
            Some(Value::Mapping(v)) => {
                serde::de::Deserializer::deserialize_any(MapDeserializer::new(v), visitor)
            }
            Some(other) => {
                Err(serde::de::Error::invalid_type(other.unexpected(), &"struct variant"))
            }
            _ => Err(serde::de::Error::invalid_type(Unexpected::UnitVariant, &"struct variant")),
        }
    }
}

struct SeqDeserializer {
    iter: vec::IntoIter<Value>,
}

impl SeqDeserializer {
    fn new(vec: Vec<Value>) -> Self {
        SeqDeserializer { iter: vec.into_iter() }
    }
}

impl<'de> serde::de::Deserializer<'de> for SeqDeserializer {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where V: Visitor<'de>
    {
        let len = self.iter.len();
        if len == 0 {
            visitor.visit_unit()
        } else {
            let ret = visitor.visit_seq(&mut self)?;
            let remaining = self.iter.len();
            if remaining == 0 {
                Ok(ret)
            } else {
                Err(serde::de::Error::invalid_length(len, &"fewer elements in sequence"))
            }
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct
        map struct enum identifier ignored_any
    }
}

impl<'de> serde::de::SeqAccess<'de> for SeqDeserializer {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
        where T: serde::de::DeserializeSeed<'de>
    {
        match self.iter.next() {
            Some(value) => seed.deserialize(value).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

struct MapDeserializer {
    iter: <Mapping as IntoIterator>::IntoIter,
    value: Option<Value>,
}

impl MapDeserializer {
    fn new(map: Mapping) -> Self {
        MapDeserializer {
            iter: map.into_iter(),
            value: None,
        }
    }
}

impl<'de> serde::de::MapAccess<'de> for MapDeserializer {
    type Error = Error;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
        where T: serde::de::DeserializeSeed<'de>
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                seed.deserialize(key).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Error>
        where T: serde::de::DeserializeSeed<'de>
    {
        match self.value.take() {
            Some(value) => seed.deserialize(value),
            None => panic!("visit_value called before visit_key"),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

impl<'de> serde::de::Deserializer<'de> for MapDeserializer {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
        where V: Visitor<'de>
    {
        visitor.visit_map(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct
        map struct enum identifier ignored_any
    }
}

impl Value {
    fn unexpected(&self) -> Unexpected {
        match *self {
            Value::Null => Unexpected::Unit,
            Value::Bool(b) => Unexpected::Bool(b),
            Value::I64(i) => Unexpected::Signed(i),
            Value::F64(f) => Unexpected::Float(f),
            Value::String(ref s) => Unexpected::Str(s),
            Value::Sequence(_) => Unexpected::Seq,
            Value::Mapping(_) => Unexpected::Map,
        }
    }
}

// - - -
//
// The original implementation of the indexing tricks below is from `serde_json`.

/// A type that can be used to index into a `serde_yaml::Value`. See the `get`
/// and `get_mut` methods of `Value`.
///
/// This trait is sealed and cannot be implemented for types outside of
/// `serde_yaml`.
pub trait Index: private::Sealed {
    /// Return None if the key is not already in the sequence or object.
    #[doc(hidden)]
    fn index_into<'v>(&self, v: &'v Value) -> Option<&'v Value>;

    /// Return None if the key is not already in the sequence or object.
    #[doc(hidden)]
    fn index_into_mut<'v>(&self, v: &'v mut Value) -> Option<&'v mut Value>;

    /// Panic if sequence index out of bounds. If key is not already in the object,
    /// insert it with a value of null. Panic if Value is a type that cannot be
    /// indexed into, except if Value is null then it can be treated as an empty
    /// object.
    #[doc(hidden)]
    fn index_or_insert<'v>(&self, v: &'v mut Value) -> &'v mut Value;
}

impl Index for usize {
    fn index_into<'v>(&self, v: &'v Value) -> Option<&'v Value> {
        match *v {
            Value::Sequence(ref vec) => vec.get(*self),
            Value::Mapping(ref vec) => vec.get(&Value::I64(*self as i64)),
            _ => None,
        }
    }
    fn index_into_mut<'v>(&self, v: &'v mut Value) -> Option<&'v mut Value> {
        match *v {
            Value::Sequence(ref mut vec) => vec.get_mut(*self),
            Value::Mapping(ref mut vec) => vec.get_mut(&Value::I64(*self as i64)),
            _ => None,
        }
    }
    fn index_or_insert<'v>(&self, v: &'v mut Value) -> &'v mut Value {
        match *v {
            Value::Sequence(ref mut vec) => {
                let len = vec.len();
                vec.get_mut(*self)
                    .unwrap_or_else(
                        || {
                            panic!(
                                "cannot access index {} of YAML sequence of length {}",
                                self,
                                len
                            )
                        },
                    )
            },
            Value::Mapping(ref mut map) => {
                // TODO: use entry() once LinkedHashMap supports entry()
                // https://github.com/contain-rs/linked-hash-map/issues/5
                if !map.contains_key(&Value::I64(*self as i64)) {
                    map.insert(Value::I64(*self as i64), Value::Null);
                }
                map.get_mut(&Value::I64(*self as i64)).unwrap()
            },
            _ => panic!("cannot access index {} of YAML {}", self, Type(v)),
        }
    }
}

impl Index for Value {
    fn index_into<'v>(&self, v: &'v Value) -> Option<&'v Value> {
        match *v {
            Value::Mapping(ref map) => map.get(self),
            _ => None,
        }
    }
    fn index_into_mut<'v>(&self, v: &'v mut Value) -> Option<&'v mut Value> {
        match *v {
            Value::Mapping(ref mut map) => map.get_mut(self),
            _ => None,
        }
    }
    fn index_or_insert<'v>(&self, v: &'v mut Value) -> &'v mut Value {
        if let Value::Null = *v {
            let mut map = Mapping::new();
            map.insert(self.clone(), Value::Null);
            *v = Value::Mapping(map);
        }
        match *v {
            Value::Mapping(ref mut map) => {
                // TODO: use entry() once LinkedHashMap supports entry()
                // https://github.com/contain-rs/linked-hash-map/issues/5
                if !map.contains_key(self) {
                    map.insert(self.clone(), Value::Null);
                }
                map.get_mut(self).unwrap()
            }
            _ => panic!("cannot access key {:?} in YAML {}", self, Type(v)),
        }
    }
}

impl Index for str {
    fn index_into<'v>(&self, v: &'v Value) -> Option<&'v Value> {
        Value::String(self.into()).index_into(v)
    }
    fn index_into_mut<'v>(&self, v: &'v mut Value) -> Option<&'v mut Value> {
        Value::String(self.into()).index_into_mut(v)
    }
    fn index_or_insert<'v>(&self, v: &'v mut Value) -> &'v mut Value {
        Value::String(self.into()).index_or_insert(v)
    }
}

impl Index for String {
    fn index_into<'v>(&self, v: &'v Value) -> Option<&'v Value> {
        Value::String(self.clone()).index_into(v)
    }
    fn index_into_mut<'v>(&self, v: &'v mut Value) -> Option<&'v mut Value> {
        Value::String(self.clone()).index_into_mut(v)
    }
    fn index_or_insert<'v>(&self, v: &'v mut Value) -> &'v mut Value {
        Value::String(self.clone()).index_or_insert(v)
    }
}

impl<'a, T: ?Sized> Index for &'a T
where
    T: Index,
{
    fn index_into<'v>(&self, v: &'v Value) -> Option<&'v Value> {
        (**self).index_into(v)
    }
    fn index_into_mut<'v>(&self, v: &'v mut Value) -> Option<&'v mut Value> {
        (**self).index_into_mut(v)
    }
    fn index_or_insert<'v>(&self, v: &'v mut Value) -> &'v mut Value {
        (**self).index_or_insert(v)
    }
}

// Prevent users from implementing the Index trait.
mod private {
    pub trait Sealed {}
    impl Sealed for usize {}
    impl Sealed for str {}
    impl Sealed for String {}
    impl Sealed for super::Value {}
    impl<'a, T: ?Sized> Sealed for &'a T
    where
        T: Sealed,
    {
    }
}

/// Used in panic messages.
struct Type<'a>(&'a Value);

impl<'a> fmt::Display for Type<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self.0 {
            Value::Null => formatter.write_str("null"),
            Value::Bool(_) => formatter.write_str("boolean"),
            Value::I64(_) => formatter.write_str("integer"),
            Value::F64(_) => formatter.write_str("float"),
            Value::String(_) => formatter.write_str("string"),
            Value::Sequence(_) => formatter.write_str("sequence"),
            Value::Mapping(_) => formatter.write_str("mapping"),
        }
    }
}

// The usual semantics of Index is to panic on invalid indexing.
//
// That said, the usual semantics are for things like `Vec` and `BTreeMap` which
// have different use cases than Value. If you are working with a Vec, you know
// that you are working with a Vec and you can get the len of the Vec and make
// sure your indices are within bounds. The Value use cases are more
// loosey-goosey. You got some YAML from an endpoint and you want to pull values
// out of it. Outside of this Index impl, you already have the option of using
// `value.as_sequence()` and working with the Vec directly, or matching on
// `Value::Sequence` and getting the Vec directly. The Index impl means you can
// skip that and index directly into the thing using a concise syntax. You don't
// have to check the type, you don't have to check the len, it is all about what
// you expect the Value to look like.
//
// Basically the use cases that would be well served by panicking here are
// better served by using one of the other approaches: `get` and `get_mut`,
// `as_sequence`, or match. The value of this impl is that it adds a way of
// working with Value that is not well served by the existing approaches:
// concise and careless and sometimes that is exactly what you want.
impl<I> ops::Index<I> for Value
where
    I: Index,
{
    type Output = Value;

    /// Index into a `serde_yaml::Value` using the syntax `value[0]` or
    /// `value["k"]`.
    ///
    /// Returns `Value::Null` if the type of `self` does not match the type of
    /// the index, for example if the index is a string and `self` is a sequence
    /// or a number. Also returns `Value::Null` if the given key does not exist
    /// in the map or the given index is not within the bounds of the sequence.
    ///
    /// For retrieving deeply nested values, you should have a look at the
    /// `Value::pointer` method.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate serde_yaml;
    /// #
    /// # fn yaml(i: &str) -> serde_yaml::Value { serde_yaml::from_str(i).unwrap() }
    /// # fn main() {
    /// let data = yaml(r#"{ x: { y: [z, zz] } }"#);
    ///
    /// assert_eq!(data["x"]["y"], yaml(r#"["z", "zz"]"#));
    /// assert_eq!(data["x"]["y"][0], yaml(r#""z""#));
    ///
    /// assert_eq!(data["a"], yaml(r#"null"#)); // returns null for undefined values
    /// assert_eq!(data["a"]["b"], yaml(r#"null"#)); // does not panic
    /// # }
    /// ```
    fn index(&self, index: I) -> &Value {
        static NULL: Value = Value::Null;
        index.index_into(self).unwrap_or(&NULL)
    }
}

impl<I> ops::IndexMut<I> for Value
where
    I: Index,
{
    /// Write into a `serde_yaml::Value` using the syntax `value[0] = ...` or
    /// `value["k"] = ...`.
    ///
    /// If the index is a number, the value must be a sequence of length bigger
    /// than the index. Indexing into a value that is not a sequence or a
    /// sequence that is too small will panic.
    ///
    /// If the index is a string, the value must be an object or null which is
    /// treated like an empty object. If the key is not already present in the
    /// object, it will be inserted with a value of null. Indexing into a value
    /// that is neither an object nor null will panic.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate serde_yaml;
    /// #
    /// # fn yaml(i: &str) -> serde_yaml::Value { serde_yaml::from_str(i).unwrap() }
    /// # fn main() {
    /// let mut data = yaml(r#"{x: 0}"#);
    ///
    /// // replace an existing key
    /// data["x"] = yaml(r#"1"#);
    ///
    /// // insert a new key
    /// data["y"] = yaml(r#"[false, false, false]"#);
    ///
    /// // replace a value in a sequence
    /// data["y"][0] = yaml(r#"true"#);
    ///
    /// // inserted a deeply nested key
    /// data["a"]["b"]["c"]["d"] = yaml(r#"true"#);
    ///
    /// println!("{:?}", data);
    /// # }
    /// ```
    fn index_mut(&mut self, index: I) -> &mut Value {
        index.index_or_insert(self)
    }
}

// - - -
//
// Implement a bunch of conversion to make it easier to create YAML values
// on the fly.

macro_rules! from_integer {
    ($($ty:ident)*) => {
        $(
            impl From<$ty> for Value {
                fn from(n: $ty) -> Self {
                    Value::I64(n as i64)
                }
            }
        )*
    };
}

from_integer! {
    i8 i16 i32 i64 isize
    u8 u16 u32 usize
}

impl From<f32> for Value {
    /// Convert 32-bit floating point number to `Value`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate serde_yaml;
    /// #
    /// # fn main() {
    /// use serde_yaml::Value;
    ///
    /// let f: f32 = 13.37;
    /// let x: Value = f.into();
    /// # }
    /// ```
    fn from(f: f32) -> Self {
        From::from(f as f64)
    }
}

impl From<f64> for Value {
    /// Convert 64-bit floating point number to `Value`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate serde_yaml;
    /// #
    /// # fn main() {
    /// use serde_yaml::Value;
    ///
    /// let f: f64 = 13.37;
    /// let x: Value = f.into();
    /// # }
    /// ```
    fn from(f: f64) -> Self {
        Value::F64(f)
    }
}

impl From<bool> for Value {
    /// Convert boolean to `Value`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate serde_yaml;
    /// #
    /// # fn main() {
    /// use serde_yaml::Value;
    ///
    /// let b = false;
    /// let x: Value = b.into();
    /// # }
    /// ```
    fn from(f: bool) -> Self {
        Value::Bool(f)
    }
}

impl From<String> for Value {
    /// Convert `String` to `Value`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate serde_yaml;
    /// #
    /// # fn main() {
    /// use serde_yaml::Value;
    ///
    /// let s: String = "lorem".to_string();
    /// let x: Value = s.into();
    /// # }
    /// ```
    fn from(f: String) -> Self {
        Value::String(f)
    }
}

impl<'a> From<&'a str> for Value {
    /// Convert string slice to `Value`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate serde_yaml;
    /// #
    /// # fn main() {
    /// use serde_yaml::Value;
    ///
    /// let s: &str = "lorem";
    /// let x: Value = s.into();
    /// # }
    /// ```
    fn from(f: &str) -> Self {
        Value::String(f.to_string())
    }
}

use std::borrow::Cow;

impl<'a> From<Cow<'a, str>> for Value {
    /// Convert copy-on-write string to `Value`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate serde_yaml;
    /// #
    /// # fn main() {
    /// use serde_yaml::Value;
    /// use std::borrow::Cow;
    ///
    /// let s: Cow<str> = Cow::Borrowed("lorem");
    /// let x: Value = s.into();
    /// # }
    /// ```
    ///
    /// ```rust
    /// # extern crate serde_yaml;
    /// #
    /// # fn main() {
    /// use serde_yaml::Value;
    /// use std::borrow::Cow;
    ///
    /// let s: Cow<str> = Cow::Owned("lorem".to_string());
    /// let x: Value = s.into();
    /// # }
    /// ```
    fn from(f: Cow<'a, str>) -> Self {
        Value::String(f.to_string())
    }
}

impl From<Mapping> for Value {
    /// Convert map (with string keys) to `Value`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate serde_yaml;
    /// #
    /// # fn main() {
    /// use serde_yaml::{Mapping, Value};
    ///
    /// let mut m = Mapping::new();
    /// m.insert("Lorem".into(), "ipsum".into());
    /// let x: Value = m.into();
    /// # }
    /// ```
    fn from(f: Mapping) -> Self {
        Value::Mapping(f)
    }
}

impl<T: Into<Value>> From<Vec<T>> for Value {
    /// Convert a `Vec` to `Value`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate serde_yaml;
    /// #
    /// # fn main() {
    /// use serde_yaml::Value;
    ///
    /// let v = vec!["lorem", "ipsum", "dolor"];
    /// let x: Value = v.into();
    /// # }
    /// ```
    fn from(f: Vec<T>) -> Self {
        Value::Sequence(f.into_iter().map(Into::into).collect())
    }
}

impl<'a, T: Clone + Into<Value>> From<&'a [T]> for Value {
    /// Convert a slice to `Value`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate serde_yaml;
    /// #
    /// # fn main() {
    /// use serde_yaml::Value;
    ///
    /// let v: &[&str] = &["lorem", "ipsum", "dolor"];
    /// let x: Value = v.into();
    /// # }
    /// ```
    fn from(f: &'a [T]) -> Self {
        Value::Sequence(f.into_iter().cloned().map(Into::into).collect())
    }
}

use std::iter::FromIterator;

impl<T: Into<Value>> FromIterator<T> for Value {
    /// Convert an iteratable type to a YAML sequence
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate serde_yaml;
    /// #
    /// # fn main() {
    /// use serde_yaml::Value;
    ///
    /// let v = std::iter::repeat(42).take(5);
    /// let x: Value = v.collect();
    /// # }
    /// ```
    ///
    /// ```rust
    /// # extern crate serde_yaml;
    /// #
    /// # fn main() {
    /// use serde_yaml::Value;
    ///
    /// let v: Vec<_> = vec!["lorem", "ipsum", "dolor"];
    /// let x: Value = v.into_iter().collect();
    /// # }
    /// ```
    ///
    /// ```rust
    /// # extern crate serde_yaml;
    /// #
    /// # fn main() {
    /// use std::iter::FromIterator;
    /// use serde_yaml::Value;
    ///
    /// let x: Value = Value::from_iter(vec!["lorem", "ipsum", "dolor"]);
    /// # }
    /// ```
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let vec: Vec<Value> = iter.into_iter().map(|x| x.into()).collect();

        Value::Sequence(vec)
    }
}
