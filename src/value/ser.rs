// Copyright 2016 Serde YAML Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::num;

use serde::ser;
use yaml_rust::yaml::{self, Hash, Yaml};

use super::{Error, Value};

impl ser::Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        match *self {
            Value::Null => serializer.serialize_unit(),
            Value::Bool(b) => serializer.serialize_bool(b),
            Value::Number(ref n) => n.serialize(serializer),
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

// Inner Serializer without writer
pub struct Serializer;

impl ser::Serializer for Serializer {
    type Ok = Yaml;
    type Error = Error;

    type SerializeSeq = SerializeVec;
    type SerializeTuple = SerializeVec;
    type SerializeTupleStruct = SerializeVec;
    type SerializeTupleVariant = SerializeTupleVariant;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeStruct;
    type SerializeStructVariant = SerializeStructVariant;

    #[inline]
    fn serialize_bool(self, value: bool) -> Result<Yaml, Error> {
        Ok(Yaml::Boolean(value))
    }

    #[inline]
    fn serialize_i8(self, value: i8) -> Result<Yaml, Error> {
        self.serialize_i64(value as i64)
    }

    #[inline]
    fn serialize_i16(self, value: i16) -> Result<Yaml, Error> {
        self.serialize_i64(value as i64)
    }

    #[inline]
    fn serialize_i32(self, value: i32) -> Result<Yaml, Error> {
        self.serialize_i64(value as i64)
    }

    // XXX: should we have inline here?
    fn serialize_i64(self, value: i64) -> Result<Yaml, Error> {
        Ok(Yaml::Integer(value))
    }

    serde_if_integer128! {
        #[cfg_attr(feature = "cargo-clippy", allow(cast_possible_truncation))]
        // XXX: should we have inline here?
        fn serialize_i128(self, value: i128) -> Result<Yaml, Error> {
            if value <= i64::max_value() as i128 && value >= i64::min_value() as i128 {
                self.serialize_i64(value as i64)
            } else {
                Ok(Yaml::Real(value.to_string()))
            }
        }
    }

    #[inline]
    fn serialize_u8(self, value: u8) -> Result<Yaml, Error> {
        self.serialize_i64(value as i64)
    }

    #[inline]
    fn serialize_u16(self, value: u16) -> Result<Yaml, Error> {
        self.serialize_i64(value as i64)
    }

    #[inline]
    fn serialize_u32(self, value: u32) -> Result<Yaml, Error> {
        self.serialize_i64(value as i64)
    }

    // XXX: should we have inline here?
    fn serialize_u64(self, value: u64) -> Result<Yaml, Error> {
        if value <= i64::max_value() as u64 {
            self.serialize_i64(value as i64)
        } else {
            Ok(Yaml::Real(value.to_string()))
        }
    }

    #[inline]
    fn serialize_f32(self, value: f32) -> Result<Yaml, Error> {
        self.serialize_f64(value as f64)
    }

    #[inline]
    fn serialize_f64(self, value: f64) -> Result<Yaml, Error> {
        Ok(Yaml::Real(match value.classify() {
            num::FpCategory::Infinite if value.is_sign_positive() => ".inf".into(),
            num::FpCategory::Infinite => "-.inf".into(),
            num::FpCategory::Nan => ".nan".into(),
            _ => {
                let mut buf = vec![];
                dtoa::write(&mut buf, value).unwrap();
                std::str::from_utf8(&buf).unwrap().into()
            }
        }))
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<Yaml, Error> {
        Ok(Yaml::String(value.to_string()))
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<Yaml, Error> {
        Ok(Yaml::String(value.to_owned()))
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Yaml, Error> {
        let vec = value.iter().map(|&b| Yaml::Integer(b.into())).collect();
        Ok(Yaml::Array(vec))
    }

    #[inline]
    fn serialize_unit(self) -> Result<Yaml, Error> {
        Ok(Yaml::Null)
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Yaml, Error> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Yaml, Error> {
        self.serialize_str(variant)
    }

    #[inline]
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Yaml, Error>
    where
        T: ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Yaml, Error>
    where
        T: ser::Serialize,
    {
        Ok(singleton_hash(to_yaml(variant)?, to_yaml(value)?))
    }

    #[inline]
    fn serialize_none(self) -> Result<Yaml, Error> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Yaml, Error>
    where
        T: ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Error> {
        Ok(SerializeVec {
            array: yaml::Array::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTuple, Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize
    ) -> Result<Self::SerializeTupleVariant, Error> {
        Ok(SerializeTupleVariant {
            name: variant,
            array: yaml::Array::with_capacity(len),
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Error> {
        Ok(SerializeMap {
            hash: yaml::Hash::new(),
            next_key: None,
        })
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct, Error> {
        Ok(SerializeStruct {
            hash: yaml::Hash::new(),
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Error> {
        Ok(SerializeStructVariant {
            name: variant,
            hash: yaml::Hash::new(),
        })
    }
}

#[doc(hidden)]
pub struct SerializeVec {
    array: yaml::Array,
}

#[doc(hidden)]
pub struct SerializeTupleVariant {
    name: &'static str,
    array: yaml::Array,
}

#[doc(hidden)]
pub struct SerializeMap {
    hash: yaml::Hash,
    next_key: Option<Yaml>,
}

#[doc(hidden)]
pub struct SerializeStruct {
    hash: yaml::Hash,
}

#[doc(hidden)]
pub struct SerializeStructVariant {
    name: &'static str,
    hash: yaml::Hash,
}

impl ser::SerializeSeq for SerializeVec {
    type Ok = Yaml;
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ser::Serialize,
    {
        self.array.push(to_yaml(value)?);
        Ok(())
    }

    fn end(self) -> Result<Yaml, Error> {
        Ok(Yaml::Array(self.array))
    }
}

impl ser::SerializeTuple for SerializeVec {
    type Ok = Yaml;
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ser::Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Yaml, Error> {
        ser::SerializeSeq::end(self)
    }
}

impl ser::SerializeTupleStruct for SerializeVec {
    type Ok = Yaml;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ser::Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Yaml, Error> {
        ser::SerializeSeq::end(self)
    }
}

impl ser::SerializeTupleVariant for SerializeTupleVariant {
    type Ok = Yaml;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ser::Serialize,
    {
        self.array.push(to_yaml(value)?);
        Ok(())
    }

    fn end(self) -> Result<Yaml, Error> {
        Ok(singleton_hash(to_yaml(self.name)?, Yaml::Array(self.array)))
    }
}

impl ser::SerializeMap for SerializeMap {
    type Ok = Yaml;
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Error>
    where
        T: ser::Serialize,
    {
        self.next_key = Some(to_yaml(key)?);
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ser::Serialize,
    {
        let key = self.next_key.take();
        // Panic because this indicates a bug in the program rather than an
        // expected failure.
        let key = key.expect("serialize_value called before serialize_key");
        self.hash.insert(key, to_yaml(&value)?);
        Ok(())
    }

    // XXX: does this make a difference?
    fn serialize_entry<K: ?Sized, V: ?Sized>(&mut self, key: &K, value: &V) -> Result<(), Error>
    where
        K: ser::Serialize,
        V: ser::Serialize,
    {
        self.hash.insert(to_yaml(key)?, to_yaml(value)?);
        Ok(())
    }

    fn end(self) -> Result<Yaml, Error> {
        Ok(Yaml::Hash(self.hash))
    }
}

impl ser::SerializeStruct for SerializeStruct {
    type Ok = Yaml;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), Error>
    where
        T: ser::Serialize,
    {
        self.hash.insert(to_yaml(key)?, to_yaml(value)?);
        Ok(())
    }

    fn end(self) -> Result<Yaml, Error> {
        Ok(Yaml::Hash(self.hash))
    }
}

impl ser::SerializeStructVariant for SerializeStructVariant {
    type Ok = Yaml;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), Error>
    where
        T: ser::Serialize,
    {
        self.hash.insert(to_yaml(key)?, to_yaml(value)?);
        Ok(())
    }

    fn end(self) -> Result<Yaml, Error> {
        Ok(singleton_hash(to_yaml(self.name)?, Yaml::Hash(self.hash)))
    }
}

// internal yaml_rust value conversion
// TODO: convert to rust value on the fly
fn to_yaml<T>(value: T) -> Result<Yaml, Error>
where
    T: ser::Serialize,
{
    value.serialize(Serializer)
}

// XXX: not sure why we need a singleton_hash
fn singleton_hash(k: Yaml, v: Yaml) -> Yaml {
    let mut hash = yaml::Hash::new();
    hash.insert(k, v);
    Yaml::Hash(hash)
}
