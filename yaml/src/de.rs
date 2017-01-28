// Copyright 2016 Serde YAML Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! YAML Deserialization
//!
//! This module provides YAML deserialization with the type `Deserializer`.

use std::io;
use std::iter;
use std::str;

use yaml_rust::{Yaml, YamlLoader};
use yaml_rust::yaml;

use serde::de::{self, Deserialize, DeserializeSeed};

use super::error::{Error, Result};

pub struct Deserializer {
    doc: Yaml,
}

impl Deserializer {
    pub fn new(doc: Yaml) -> Self {
        Deserializer {
            doc: doc,
        }
    }
}

struct SeqVisitor {
    /// Iterator over the YAML array being visited.
    iter: <yaml::Array as iter::IntoIterator>::IntoIter,
}

impl SeqVisitor {
    fn new(seq: yaml::Array) -> Self {
        SeqVisitor {
            iter: seq.into_iter(),
        }
    }
}

impl de::SeqVisitor for SeqVisitor {
    type Error = Error;

    fn visit_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
        where T: DeserializeSeed
    {
        match self.iter.next() {
            None => Ok(None),
            Some(t) => seed.deserialize(Deserializer::new(t)).map(Some),
        }
    }
}

struct MapVisitor {
    /// Iterator over the YAML hash being visited.
    iter: <yaml::Hash as iter::IntoIterator>::IntoIter,
    /// Value associated with the most recently visited key.
    v: Option<Yaml>,
}

impl MapVisitor {
    fn new(hash: yaml::Hash) -> Self {
        MapVisitor {
            iter: hash.into_iter(),
            v: None,
        }
    }
}

impl de::MapVisitor for MapVisitor {
    type Error = Error;

    fn visit_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
        where K: DeserializeSeed
    {
        match self.iter.next() {
            None => Ok(None),
            Some((k, v)) => {
                self.v = Some(v);
                seed.deserialize(Deserializer::new(k)).map(Some)
            }
        }
    }

    fn visit_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
        where V: DeserializeSeed
    {
        match self.v.take() {
            Some(v) => seed.deserialize(Deserializer::new(v)),
            None => panic!("visit_value called before visit_key"),
        }
    }
}

struct EnumVisitor {
    variant: Yaml,
    content: Yaml,
}

impl EnumVisitor {
    fn new(variant: Yaml, content: Yaml) -> Self {
        EnumVisitor {
            variant: variant,
            content: content,
        }
    }
}

impl de::EnumVisitor for EnumVisitor {
    type Error = Error;
    type Variant = VariantVisitor;

    fn visit_variant_seed<V>(
        self,
        seed: V,
    ) -> Result<(V::Value, VariantVisitor)>
        where V: DeserializeSeed
    {
        let variant = try!(seed.deserialize(Deserializer::new(self.variant)));
        Ok((variant, VariantVisitor::new(self.content)))
    }
}

struct VariantVisitor {
    content: Yaml,
}

impl VariantVisitor {
    fn new(content: Yaml) -> Self {
        VariantVisitor {
            content: content,
        }
    }
}

impl de::VariantVisitor for VariantVisitor {
    type Error = Error;

    fn visit_unit(self) -> Result<()> {
        Ok(())
    }

    fn visit_newtype_seed<T>(self, seed: T) -> Result<T::Value>
        where T: DeserializeSeed
    {
        seed.deserialize(Deserializer::new(self.content))
    }

    fn visit_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        de::Deserializer::deserialize(Deserializer::new(self.content), visitor)
    }

    fn visit_struct<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V
    ) -> Result<V::Value>
        where V: de::Visitor
    {
        de::Deserializer::deserialize(Deserializer::new(self.content), visitor)
    }
}

impl de::Deserializer for Deserializer {
    type Error = Error;

    fn deserialize<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        match self.doc {
            Yaml::Real(ref s) => {
                match s.parse() {
                    Ok(f) => visitor.visit_f64(f),
                    Err(_) => visitor.visit_str(s),
                }
            }
            Yaml::Integer(i) => visitor.visit_i64(i),
            Yaml::String(s) => visitor.visit_string(s),
            Yaml::Boolean(b) => visitor.visit_bool(b),
            Yaml::Array(seq) => visitor.visit_seq(SeqVisitor::new(seq)),
            Yaml::Hash(hash) => visitor.visit_map(MapVisitor::new(hash)),
            Yaml::Alias(_) => Err(Error::AliasUnsupported),
            Yaml::Null => visitor.visit_unit(),
            Yaml::BadValue => {
                // The yaml-rust library produces BadValue when a nonexistent
                // node is accessed by the Index trait, and when a type
                // conversion is invalid. Both of these are unexpected in our
                // usage.
                panic!("bad value")
            }
        }
    }

    /// Parses `null` as None and any other values as `Some(...)`.
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        match self.doc {
            Yaml::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    /// Parses a newtype struct as the underlying value.
    fn deserialize_newtype_struct<V>(
        self,
        _name: &str,
        visitor: V
    ) -> Result<V::Value>
        where V: de::Visitor
    {
        visitor.visit_newtype_struct(self)
    }

    /// Parses an enum as a single key:value pair where the key identifies the
    /// variant and the value gives the content. A String will also parse correctly
    /// to a unit enum value.
    fn deserialize_enum<V>(
        self,
        name: &str,
        _variants: &'static [&'static str],
        visitor: V
    ) -> Result<V::Value>
        where V: de::Visitor
    {
        match self.doc {
            Yaml::Hash(hash) => {
                let len = hash.len();
                let mut iter = hash.into_iter();
                if let (Some(entry), None) = (iter.next(), iter.next()) {
                    let (variant, content) = entry;
                    visitor.visit_enum(EnumVisitor::new(variant, content))
                } else {
                    Err(Error::VariantMapWrongSize(String::from(name), len))
                }
            }
            ystr @ Yaml::String(_) => {
                visitor.visit_enum(EnumVisitor::new(ystr, Yaml::Null))
            }
            _ => Err(Error::VariantNotAMapOrString(String::from(name))),
        }
    }

    forward_to_deserialize!{
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit seq
        seq_fixed_size bytes byte_buf map unit_struct tuple_struct struct
        struct_field tuple ignored_any
    }
}

/// Decodes a YAML value from a `&str`.
pub fn from_str<T>(s: &str) -> Result<T>
    where T: Deserialize
{
    let docs = try!(YamlLoader::load_from_str(s));
    match docs.len() {
        0 => Err(Error::EndOfStream),
        1 => {
            let doc = docs.into_iter().next().unwrap();
            Deserialize::deserialize(Deserializer::new(doc))
        }
        n => Err(Error::TooManyDocuments(n)),
    }
}

pub fn from_iter<I, T>(iter: I) -> Result<T>
    where I: Iterator<Item = io::Result<u8>>,
          T: Deserialize
{
    let bytes: Vec<u8> = try!(iter.collect());
    from_str(try!(str::from_utf8(&bytes)))
}

pub fn from_reader<R, T>(rdr: R) -> Result<T>
    where R: io::Read,
          T: Deserialize
{
    from_iter(rdr.bytes())
}

pub fn from_slice<T>(v: &[u8]) -> Result<T>
    where T: Deserialize
{
    from_iter(v.iter().map(|byte| Ok(*byte)))
}
