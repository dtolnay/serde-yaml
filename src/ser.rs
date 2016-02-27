// Copyright 2016 Serde YAML Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.extern crate serde;

//! YAML Serialization
//!
//! This module provides YAML serialization with the type `Serializer`.

use std::fmt;
use std::io;

use yaml_rust::{Yaml, YamlEmitter};
use yaml_rust::yaml;

use serde::ser;

use super::error::{Error, Result};

/// A structure for serializing a Rust value into a YAML value.
pub struct Serializer<'a> {
    /// The YAML value to hold the result.
    doc: &'a mut Yaml,
}

impl<'a> Serializer<'a> {
    pub fn new(doc: &'a mut Yaml) -> Self {
        Serializer{
            doc: doc,
        }
    }
}

impl<'a> ser::Serializer for Serializer<'a> {
    type Error = Error;

    fn serialize_bool(&mut self, v: bool) -> Result<()> {
        *self.doc = Yaml::Boolean(v);
        Ok(())
    }

    fn serialize_i64(&mut self, v: i64) -> Result<()> {
        *self.doc = Yaml::Integer(v);
        Ok(())
    }

    fn serialize_u64(&mut self, v: u64) -> Result<()> {
        self.serialize_i64(v as i64)
    }

    fn serialize_f64(&mut self, v: f64) -> Result<()> {
        *self.doc = Yaml::Real(v.to_string());
        Ok(())
    }

    fn serialize_str(&mut self, value: &str) -> Result<()> {
        *self.doc = Yaml::String(String::from(value));
        Ok(())
    }

    fn serialize_unit(&mut self) -> Result<()> {
        *self.doc = Yaml::Null;
        Ok(())
    }

    fn serialize_none(&mut self) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_some<V>(&mut self, value: V) -> Result<()>
        where V: ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq<V>(&mut self, mut visitor: V) -> Result<()>
        where V: ser::SeqVisitor,
    {
        let vec = match visitor.len() {
            None => yaml::Array::new(),
            Some(len) => yaml::Array::with_capacity(len),
        };
        *self.doc = Yaml::Array(vec);
        let mut elt_ser = Serializer::new(self.doc);
        while try!(visitor.visit(&mut elt_ser)).is_some() { }
        Ok(())
    }

    fn serialize_seq_elt<T>(&mut self, elem: T) -> Result<()>
        where T: ser::Serialize,
    {
        match *self.doc {
            Yaml::Array(ref mut vec) => {
                vec.push(try!(to_yaml(elem)));
            },
            _ => panic!("bad call to serialize_seq_elt"),
        }
        Ok(())
    }

    fn serialize_map<V>(&mut self, mut visitor: V) -> Result<()>
        where V: ser::MapVisitor,
    {
        *self.doc = Yaml::Hash(yaml::Hash::new());
        let mut elt_ser = Serializer::new(self.doc);
        while try!(visitor.visit(&mut elt_ser)).is_some() { }
        Ok(())
    }

    fn serialize_map_elt<K, V>(&mut self, key: K, value: V) -> Result<()>
        where K: ser::Serialize,
              V: ser::Serialize,
    {
        match *self.doc {
            Yaml::Hash(ref mut map) => {
                map.insert(try!(to_yaml(key)), try!(to_yaml(value)));
            },
            _ => panic!("bad call to serialize_map_elt"),
        }
        Ok(())
    }

    fn serialize_unit_variant(&mut self,
                          _name: &str,
                          _variant_index: usize,
                          variant: &str) -> Result<()> {
        *self.doc = singleton_hash(
            try!(to_yaml(variant)),
            Yaml::Array(yaml::Array::new()));
        Ok(())
    }

    /// Override `serialize_newtype_struct` to serialize newtypes without an object wrapper.
    fn serialize_newtype_struct<T>(&mut self,
                               _name: &'static str,
                               value: T) -> Result<()>
        where T: ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(&mut self,
                                _name: &str,
                                _variant_index: usize,
                                variant: &str,
                                value: T) -> Result<()>
        where T: ser::Serialize,
    {
        *self.doc = singleton_hash(
            try!(to_yaml(variant)),
            try!(to_yaml(value)));
        Ok(())
    }

    fn serialize_tuple_variant<V>(&mut self,
                              _name: &str,
                              _variant_index: usize,
                              variant: &str,
                              visitor: V) -> Result<()>
        where V: ser::SeqVisitor,
    {
        let mut values = Yaml::Null;
        try!(Serializer::new(&mut values).serialize_seq(visitor));
        *self.doc = singleton_hash(
            try!(to_yaml(variant)),
            values);
        Ok(())
    }

    fn serialize_struct_variant<V>(&mut self,
                               _name: &str,
                               _variant_index: usize,
                               variant: &str,
                               visitor: V) -> Result<()>
        where V: ser::MapVisitor,
    {
        let mut values = Yaml::Null;
        try!(Serializer::new(&mut values).serialize_map(visitor));
        *self.doc = singleton_hash(
            try!(to_yaml(variant)),
            values);
        Ok(())
    }
}

pub fn to_writer<W, T>(writer: &mut W, value: &T) -> Result<()>
    where W: io::Write,
          T: ser::Serialize,
{
    let doc = try!(to_yaml(value));
    let mut writer_adapter = FmtToIoWriter{writer: writer};
    try!(YamlEmitter::new(&mut writer_adapter).dump(&doc));
    Ok(())
}

pub fn to_string<T>(value: &T) -> Result<String>
    where T: ser::Serialize
{
    let mut vec = Vec::with_capacity(128);
    try!(to_writer(&mut vec, value));
    Ok(try!(String::from_utf8(vec)))
}

/// The yaml-rust library uses `fmt.Write` intead of `io.Write` so this is a
/// simple adapter.
struct FmtToIoWriter<'a, W>
    where W: io::Write + 'a,
{
    writer: &'a mut W,
}

impl<'a, W> fmt::Write for FmtToIoWriter<'a, W>
    where W: io::Write + 'a,
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if self.writer.write(s.as_bytes()).is_err() {
            return Err(fmt::Error);
        }
        if self.writer.flush().is_err() {
            return Err(fmt::Error);
        }
        Ok(())
    }
}

fn to_yaml<T>(elem: T) -> Result<Yaml>
    where T: ser::Serialize
{
    let mut result = Yaml::Null;
    try!(elem.serialize(&mut Serializer::new(&mut result)));
    Ok(result)
}

fn singleton_hash(k: Yaml, v: Yaml) -> Yaml {
    let mut hash = yaml::Hash::new();
    hash.insert(k, v);
    Yaml::Hash(hash)
}
