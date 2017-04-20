// Copyright 2016 Serde YAML Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt;
use std::hash::{Hash, Hasher};
use std::mem;
use std::vec;

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
    pub fn is_null(&self) -> bool {
        if let Value::Null = *self { true } else { false }
    }

    pub fn is_bool(&self) -> bool {
        self.as_bool().is_some()
    }

    pub fn as_bool(&self) -> Option<bool> {
        match *self {
            Value::Bool(b) => Some(b),
            _ => None,
        }
    }

    pub fn is_i64(&self) -> bool {
        self.as_i64().is_some()
    }

    pub fn as_i64(&self) -> Option<i64> {
        match *self {
            Value::I64(i) => Some(i),
            _ => None,
        }
    }

    pub fn is_f64(&self) -> bool {
        self.as_f64().is_some()
    }

    pub fn as_f64(&self) -> Option<f64> {
        match *self {
            Value::F64(i) => Some(i),
            _ => None,
        }
    }

    pub fn is_string(&self) -> bool {
        self.as_str().is_some()
    }

    pub fn as_str(&self) -> Option<&str> {
        match *self {
            Value::String(ref s) => Some(s),
            _ => None,
        }
    }

    pub fn is_sequence(&self) -> bool {
        self.as_sequence().is_some()
    }

    pub fn as_sequence(&self) -> Option<&Sequence> {
        match *self {
            Value::Sequence(ref seq) => Some(seq),
            _ => None,
        }
    }

    pub fn as_sequence_mut(&mut self) -> Option<&mut Sequence> {
        match *self {
            Value::Sequence(ref mut seq) => Some(seq),
            _ => None,
        }
    }

    pub fn is_mapping(&self) -> bool {
        self.as_mapping().is_some()
    }

    pub fn as_mapping(&self) -> Option<&Mapping> {
        match *self {
            Value::Mapping(ref map) => Some(map),
            _ => None,
        }
    }

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
        Yaml::Array(array) => Value::Sequence(array.into_iter().map(yaml_to_value).collect()),
        Yaml::Hash(hash) => {
            Value::Mapping(hash.into_iter().map(|(k, v)| (yaml_to_value(k), yaml_to_value(v))).collect())
        }
        Yaml::Alias(_) => panic!("alias unsupported"),
        Yaml::Null => Value::Null,
        Yaml::BadValue => panic!("bad value"),
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
                    Err(serde::de::Error::invalid_length(len, &"fewer elements in array"))
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
                Err(serde::de::Error::invalid_length(len, &"fewer elements in array"))
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
