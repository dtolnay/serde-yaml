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
use std::slice;
use std::str;

use yaml_rust::{Yaml, YamlLoader};
use yaml_rust::yaml;

use serde::de::{self, Deserialize};

use super::error::{Error, Result};

/// A structure for deserializing a YAML value into a Rust value.
pub struct Deserializer<'a> {
    /// YAML value being deserialized.
    doc: &'a Yaml,
}

impl<'a> Deserializer<'a> {
    /// Creates the YAML deserializer from an in-memory `Yaml`.
    pub fn new(doc: &'a Yaml) -> Self {
        Deserializer {
            doc: doc,
        }
    }
}

struct SeqVisitor<'a> {
    /// Iterator over the YAML array being visited.
    iter: slice::Iter<'a, Yaml>,
}

impl<'a> SeqVisitor<'a> {
    fn new(seq: &'a [Yaml]) -> Self {
        SeqVisitor {
            iter: seq.iter(),
        }
    }
}

impl<'a> de::SeqVisitor for SeqVisitor<'a> {
    type Error = Error;

    fn visit<T>(&mut self) -> Result<Option<T>>
        where T: Deserialize,
    {
        match self.iter.next() {
            None => Ok(None),
            Some(ref t) => {
                Deserialize::deserialize(&mut Deserializer::new(t)).map(Some)
            },
        }
    }

    fn end(&mut self) -> Result<()> {
        Ok(())
    }
}

struct MapVisitor<'a> {
    /// Iterator over the YAML hash being visited.
    iter: <&'a yaml::Hash as iter::IntoIterator>::IntoIter,
    /// Value associated with the most recently visited key.
    v: Option<&'a Yaml>,
}

impl<'a> MapVisitor<'a> {
    fn new(hash: &'a yaml::Hash) -> Self {
        MapVisitor {
            iter: hash.into_iter(),
            v: None,
        }
    }
}

impl<'a> de::MapVisitor for MapVisitor<'a> {
    type Error = Error;
    
    fn visit_key<K>(&mut self) -> Result<Option<K>>
        where K: Deserialize,
    {
        match self.iter.next() {
            None => Ok(None),
            Some((ref k, ref v)) => {
                self.v = Some(v);
                Deserialize::deserialize(&mut Deserializer::new(k)).map(Some)
            },
        }
    }

    fn visit_value<V>(&mut self) -> Result<V>
        where V: Deserialize,
    {
        if let Some(v) = self.v {
            Deserialize::deserialize(&mut Deserializer::new(v))
        } else {
            panic!("must call visit_key before visit_value")
        }
    }

    fn end(&mut self) -> Result<()> {
        Ok(())
    }

    fn missing_field<V>(&mut self, field: &'static str) -> Result<V>
        where V: de::Deserialize,
    {
        struct MissingFieldDeserializer(&'static str);

        impl de::Deserializer for MissingFieldDeserializer {
            type Error = Error;

            fn deserialize<V>(&mut self, _visitor: V) -> Result<V::Value>
                where V: de::Visitor,
            {
                Err(de::Error::missing_field(self.0))
            }

            fn deserialize_option<V>(&mut self,
                                     mut visitor: V) -> Result<V::Value>
                where V: de::Visitor,
            {
                visitor.visit_none()
            }
        }

        let mut de = MissingFieldDeserializer(field);
        Ok(try!(de::Deserialize::deserialize(&mut de)))
    }
}

struct VariantVisitor<'a> {
    /// Representation of which variant it is.
    variant: &'a Yaml,
    /// Representation of the content of the variant.
    content: &'a Yaml,
}

impl<'a> VariantVisitor<'a> {
    fn new(variant: &'a Yaml, content: &'a Yaml) -> Self {
        VariantVisitor {
            variant: variant,
            content: content,
        }
    }
}

impl <'a> de::VariantVisitor for VariantVisitor<'a> {
    type Error = Error;

    fn visit_variant<V>(&mut self) -> Result<V>
        where V: Deserialize
    {
        Deserialize::deserialize(&mut Deserializer::new(self.variant))
    }

    fn visit_unit(&mut self) -> Result<()> {
        Ok(())
    }

    fn visit_newtype<T>(&mut self) -> Result<T>
        where T: Deserialize,
    {
        Deserialize::deserialize(&mut Deserializer::new(self.content))
    }

    fn visit_tuple<V>(&mut self,
                      _len: usize,
                      visitor: V) -> Result<V::Value>
        where V: de::Visitor,
    {
        de::Deserializer::deserialize(&mut Deserializer::new(self.content), visitor)
    }

    fn visit_struct<V>(&mut self,
                       _fields: &'static [&'static str],
                       visitor: V) -> Result<V::Value>
        where V: de::Visitor,
    {
        de::Deserializer::deserialize(&mut Deserializer::new(self.content), visitor)
    }
}

impl<'a> de::Deserializer for Deserializer<'a> {
    type Error = Error;

    fn deserialize<V>(&mut self, mut visitor: V) -> Result<V::Value>
        where V: de::Visitor,
    {
        match *self.doc {
            Yaml::Integer(i) => visitor.visit_i64(i),
            Yaml::Real(ref s) | Yaml::String(ref s) => visitor.visit_str(s),
            Yaml::Boolean(b) => visitor.visit_bool(b),
            Yaml::Array(ref seq) => visitor.visit_seq(SeqVisitor::new(seq)),
            Yaml::Hash(ref hash) => visitor.visit_map(MapVisitor::new(hash)),
            Yaml::Alias(_) => Err(Error::AliasUnsupported),
            Yaml::Null => visitor.visit_unit(),
            Yaml::BadValue => {
                // The yaml-rust library produces BadValue when a nonexistent
                // node is accessed by the Index trait, and when a type
                // conversion is invalid. Both of these are unexpected in our
                // usage.
                panic!("bad value")
            },
        }
    }

    /// Parses `null` as None and any other values as `Some(...)`.
    fn deserialize_option<V>(&mut self, mut visitor: V) -> Result<V::Value>
        where V: de::Visitor,
    {
        match *self.doc {
            Yaml::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    /// Parses a newtype struct as the underlying value.
    fn deserialize_newtype_struct<V>(&mut self,
                               _name: &str,
                               mut visitor: V) -> Result<V::Value>
        where V: de::Visitor,
    {
        visitor.visit_newtype_struct(self)
    }

    /// Parses an enum as a single key:value pair where the key identifies the
    /// variant and the value gives the content.
    fn deserialize_enum<V>(&mut self,
                     name: &str,
                     _variants: &'static [&'static str],
                     mut visitor: V) -> Result<V::Value>
        where V: de::EnumVisitor,
    {
        if let Yaml::Hash(ref hash) = *self.doc {
            let mut iter = hash.iter();
            if let (Some(entry), None) = (iter.next(), iter.next()) {
                let (variant, content) = entry;
                visitor.visit(VariantVisitor::new(variant, content))
            } else {
                Err(Error::VariantMapWrongSize(String::from(name), hash.len()))
            }
        } else {
            Err(Error::VariantNotAMap(String::from(name)))
        }
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
            let doc = &docs[0];
            Deserialize::deserialize(&mut Deserializer::new(doc))
        },
        n => Err(Error::TooManyDocuments(n)),
    }
}

pub fn from_iter<I, T>(iter: I) -> Result<T>
    where I: Iterator<Item=io::Result<u8>>,
          T: Deserialize,
{
    let bytes: Vec<u8> = try!(iter.collect());
    from_str(try!(str::from_utf8(&bytes)))
}

pub fn from_reader<R, T>(rdr: R) -> Result<T>
    where R: io::Read,
          T: Deserialize,
{
    from_iter(rdr.bytes())
}

pub fn from_slice<T>(v: &[u8]) -> Result<T>
    where T: Deserialize
{
    from_iter(v.iter().map(|byte| Ok(*byte)))
}
