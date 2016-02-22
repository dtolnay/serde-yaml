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

    fn visit_bool(&mut self, v: bool) -> Result<()> {
        *self.doc = Yaml::Boolean(v);
        Ok(())
    }

    fn visit_i64(&mut self, v: i64) -> Result<()> {
        *self.doc = Yaml::Integer(v);
        Ok(())
    }

    fn visit_u64(&mut self, v: u64) -> Result<()> {
        self.visit_i64(v as i64)
    }

    fn visit_f64(&mut self, v: f64) -> Result<()> {
        *self.doc = Yaml::Real(v.to_string());
        Ok(())
    }

    fn visit_str(&mut self, value: &str) -> Result<()> {
        *self.doc = Yaml::String(String::from(value));
        Ok(())
    }

    fn visit_unit(&mut self) -> Result<()> {
        *self.doc = Yaml::Null;
        Ok(())
    }

    fn visit_none(&mut self) -> Result<()> {
        self.visit_unit()
    }

    fn visit_some<V>(&mut self, value: V) -> Result<()>
        where V: ser::Serialize,
    {
        value.serialize(self)
    }

    fn visit_seq<V>(&mut self, mut visitor: V) -> Result<()>
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

    fn visit_seq_elt<T>(&mut self, elem: T) -> Result<()>
        where T: ser::Serialize,
    {
        match *self.doc {
            Yaml::Array(ref mut vec) => {
                vec.push(try!(to_yaml(elem)));
            },
            _ => panic!("bad call to visit_seq_elt"),
        }
        Ok(())
    }

    fn visit_map<V>(&mut self, mut visitor: V) -> Result<()>
        where V: ser::MapVisitor,
    {
        *self.doc = Yaml::Hash(yaml::Hash::new());
        let mut elt_ser = Serializer::new(self.doc);
        while try!(visitor.visit(&mut elt_ser)).is_some() { }
        Ok(())
    }

    fn visit_map_elt<K, V>(&mut self, key: K, value: V) -> Result<()>
        where K: ser::Serialize,
              V: ser::Serialize,
    {
        match *self.doc {
            Yaml::Hash(ref mut map) => {
                map.insert(try!(to_yaml(key)), try!(to_yaml(value)));
            },
            _ => panic!("bad call to visit_map_elt"),
        }
        Ok(())
    }

    fn visit_unit_variant(&mut self,
                          _name: &str,
                          _variant_index: usize,
                          variant: &str) -> Result<()> {
        *self.doc = singleton_hash(
            try!(to_yaml(variant)),
            Yaml::Array(yaml::Array::new()));
        Ok(())
    }

    /// Override `visit_newtype_struct` to serialize newtypes without an object wrapper.
    fn visit_newtype_struct<T>(&mut self,
                               _name: &'static str,
                               value: T) -> Result<()>
        where T: ser::Serialize,
    {
        value.serialize(self)
    }

    fn visit_newtype_variant<T>(&mut self,
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

    fn visit_tuple_variant<V>(&mut self,
                              _name: &str,
                              _variant_index: usize,
                              variant: &str,
                              visitor: V) -> Result<()>
        where V: ser::SeqVisitor,
    {
        let mut values = Yaml::Null;
        try!(Serializer::new(&mut values).visit_seq(visitor));
        *self.doc = singleton_hash(
            try!(to_yaml(variant)),
            values);
        Ok(())
    }

    fn visit_struct_variant<V>(&mut self,
                               _name: &str,
                               _variant_index: usize,
                               variant: &str,
                               visitor: V) -> Result<()>
        where V: ser::MapVisitor,
    {
        let mut values = Yaml::Null;
        try!(Serializer::new(&mut values).visit_map(visitor));
        *self.doc = singleton_hash(
            try!(to_yaml(variant)),
            values);
        Ok(())
    }

    fn format() -> &'static str {
        "yaml"
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
