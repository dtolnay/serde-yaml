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

use std::collections::BTreeMap;
use std::io;
use std::str;

use yaml_rust::parser::{Parser, MarkedEventReceiver, Event as YamlEvent};
use yaml_rust::scanner::{Marker, TokenType, TScalarStyle};

use serde::de::{self, Deserialize, DeserializeSeed, Unexpected};

use super::error::{Error, Result};

pub struct Loader {
    events: Vec<(Event, Marker)>,
    /// Map from alias id to index in events.
    aliases: BTreeMap<usize, usize>,
}

impl MarkedEventReceiver for Loader {
    fn on_event(&mut self, event: &YamlEvent, marker: Marker) {
        let event = match *event {
            YamlEvent::Nothing
                | YamlEvent::StreamStart
                | YamlEvent::StreamEnd
                | YamlEvent::DocumentStart
                | YamlEvent::DocumentEnd => return,

            YamlEvent::Alias(id) => Event::Alias(id),
            YamlEvent::Scalar(ref value, style, id, ref tag) => {
                self.aliases.insert(id, self.events.len());
                Event::Scalar(value.clone(), style, tag.clone())
            }
            YamlEvent::SequenceStart(id) => {
                self.aliases.insert(id, self.events.len());
                Event::SequenceStart
            }
            YamlEvent::SequenceEnd => Event::SequenceEnd,
            YamlEvent::MappingStart(id) => {
                self.aliases.insert(id, self.events.len());
                Event::MappingStart
            }
            YamlEvent::MappingEnd => Event::MappingEnd,
        };
        self.events.push((event, marker));
    }
}

#[derive(Debug)]
enum Event {
    Alias(usize),
    Scalar(String, TScalarStyle, Option<TokenType>),
    SequenceStart,
    SequenceEnd,
    MappingStart,
    MappingEnd,
}

struct Deserializer<'a> {
    events: &'a [(Event, Marker)],
    /// Map from alias id to index in events.
    aliases: &'a BTreeMap<usize, usize>,
    pos: usize,
}

impl<'a> Deserializer<'a> {
    fn peek(&self) -> Result<&'a Event> {
        match self.events.get(self.pos) {
            Some(event) => Ok(&event.0),
            None => Err(Error::EndOfStream),
        }
    }

    fn next(&mut self) -> Result<&'a Event> {
        match self.events.get(self.pos) {
            Some(event) => {
                self.pos += 1;
                Ok(&event.0)
            }
            None => Err(Error::EndOfStream),
        }
    }

    fn jump(&self, id: usize) -> Result<Deserializer<'a>> {
        match self.aliases.get(&id) {
            Some(&pos) => {
                Ok(Deserializer {
                    events: self.events,
                    aliases: self.aliases,
                    pos: pos,
                })
            }
            None => Err(Error::AliasNotFound),
        }
    }
}

impl<'a> de::SeqVisitor for Deserializer<'a> {
    type Error = Error;

    fn visit_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
        where T: DeserializeSeed
    {
        match *self.peek()? {
            Event::SequenceEnd => Ok(None),
            _ => seed.deserialize(self).map(Some),
        }
    }
}

impl<'a> de::MapVisitor for Deserializer<'a> {
    type Error = Error;

    fn visit_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
        where K: DeserializeSeed
    {
        match *self.peek()? {
            Event::MappingEnd => Ok(None),
            _ => seed.deserialize(self).map(Some),
        }
    }

    fn visit_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
        where V: DeserializeSeed
    {
        seed.deserialize(self)
    }
}

struct VariantVisitor<'a: 'r, 'r> {
    de: &'r mut Deserializer<'a>,
}

impl<'a, 'r> de::EnumVisitor for VariantVisitor<'a, 'r> {
    type Error = Error;
    type Variant = VariantVisitor<'a, 'r>;

    fn visit_variant_seed<V>(
        self,
        seed: V,
    ) -> Result<(V::Value, Self::Variant)>
        where V: DeserializeSeed
    {
        Ok((try!(seed.deserialize(&mut *self.de)), self))
    }
}

impl<'a, 'r> de::VariantVisitor for VariantVisitor<'a, 'r> {
    type Error = Error;

    fn visit_unit(self) -> Result<()> {
        Deserialize::deserialize(self.de)
    }

    fn visit_newtype_seed<T>(self, seed: T) -> Result<T::Value>
        where T: DeserializeSeed
    {
        seed.deserialize(self.de)
    }

    fn visit_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        de::Deserializer::deserialize(self.de, visitor)
    }

    fn visit_struct<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V
    ) -> Result<V::Value>
        where V: de::Visitor
    {
        de::Deserializer::deserialize(self.de, visitor)
    }
}

struct UnitVariantVisitor<'a: 'r, 'r> {
    de: &'r mut Deserializer<'a>,
}

impl<'a, 'r> de::EnumVisitor for UnitVariantVisitor<'a, 'r> {
    type Error = Error;
    type Variant = Self;

    fn visit_variant_seed<V>(
        self,
        seed: V,
    ) -> Result<(V::Value, Self::Variant)>
        where V: DeserializeSeed
    {
        Ok((try!(seed.deserialize(&mut *self.de)), self))
    }
}

impl<'a, 'r> de::VariantVisitor for UnitVariantVisitor<'a, 'r> {
    type Error = Error;

    fn visit_unit(self) -> Result<()> {
        Ok(())
    }

    fn visit_newtype_seed<T>(self, _seed: T) -> Result<T::Value>
        where T: DeserializeSeed
    {
        Err(de::Error::invalid_type(Unexpected::UnitVariant, &"newtype variant"))
    }

    fn visit_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        Err(de::Error::invalid_type(Unexpected::UnitVariant, &"tuple variant"))
    }

    fn visit_struct<V>(
        self,
        _fields: &'static [&'static str],
        _visitor: V
    ) -> Result<V::Value>
        where V: de::Visitor
    {
        Err(de::Error::invalid_type(Unexpected::UnitVariant, &"struct variant"))
    }
}

fn visit_str<V>(visitor: V, v: &str) -> Result<V::Value>
    where V: de::Visitor
{
    if v == "~" || v == "null" {
        return visitor.visit_unit();
    }
    if v == "true" {
        return visitor.visit_bool(true);
    }
    if v == "false" {
        return visitor.visit_bool(false);
    }
    if v.starts_with("0x") {
        if let Ok(n) = i64::from_str_radix(&v[2..], 16) {
            return visitor.visit_i64(n);
        }
    }
    if v.starts_with("0o") {
        if let Ok(n) = i64::from_str_radix(&v[2..], 8) {
            return visitor.visit_i64(n);
        }
    }
    if v.starts_with('+') {
        if let Ok(n) = v[1..].parse() {
            return visitor.visit_i64(n);
        }
    }
    if let Ok(n) = v.parse() {
        return visitor.visit_i64(n);
    }
    if let Ok(n) = v.parse() {
        return visitor.visit_f64(n);
    }
    visitor.visit_str(v)
}

impl<'a, 'r> de::Deserializer for &'r mut Deserializer<'a> {
    type Error = Error;

    fn deserialize<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        match *self.next()? {
            Event::Alias(i) => self.jump(i)?.deserialize(visitor),
            Event::Scalar(ref v, style, ref tag) => {
                if style != TScalarStyle::Plain {
                    visitor.visit_str(v)
                } else if let Some(TokenType::Tag(ref handle, ref suffix)) = *tag {
                    if handle == "!!" {
                        match suffix.as_ref() {
                            "bool" => {
                                match v.parse::<bool>() {
                                    Ok(v) => visitor.visit_bool(v),
                                    Err(err) => Err(de::Error::custom(err)), // FIXME
                                }
                            },
                            "int" => {
                                match v.parse::<i64>() {
                                    Ok(v) => visitor.visit_i64(v),
                                    Err(err) => Err(de::Error::custom(err)), // FIXME
                                }
                            },
                            "float" => {
                                match v.parse::<f64>() {
                                    Ok(v) => visitor.visit_f64(v),
                                    Err(err) => Err(de::Error::custom(err)), // FIXME
                                }
                            },
                            "null" => {
                                match v.as_ref() {
                                    "~" | "null" => visitor.visit_unit(),
                                    _ => Err(de::Error::custom("failed to parse null")), // FIXME
                                }
                            }
                            _  => visitor.visit_str(v),
                        }
                    } else {
                        visitor.visit_str(v)
                    }
                } else {
                    visit_str(visitor, v)
                }
            }
            Event::SequenceStart => {
                let value = visitor.visit_seq(&mut *self)?;
                match *self.next()? {
                    Event::SequenceEnd => Ok(value),
                    _ => Err(de::Error::custom("remaining elements in sequence")), // FIXME
                }
            }
            Event::MappingStart => {
                let value = visitor.visit_map(&mut *self)?;
                match *self.next()? {
                    Event::MappingEnd => Ok(value),
                    _ => Err(de::Error::custom("remaining elements in map")), // FIXME
                }
            }
            Event::SequenceEnd | Event::MappingEnd => Err(Error::EndOfStream), // FIXME
        }
    }

    /// Parses `null` as None and any other values as `Some(...)`.
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        let is_some = match *self.peek()? {
            Event::Alias(i) => return self.jump(i)?.deserialize_option(visitor),
            Event::Scalar(ref v, style, ref tag) => {
                if style != TScalarStyle::Plain {
                    true
                } else if let Some(TokenType::Tag(ref handle, ref suffix)) = *tag {
                    if handle == "!!" && suffix == "null" {
                        if v == "~" || v == "null" {
                            false
                        } else {
                            return Err(de::Error::custom("failed to parse null")); // FIXME
                        }
                    } else {
                        true
                    }
                } else {
                    v != "~" && v != "null"
                }
            }
            Event::SequenceStart | Event::MappingStart => true,
            Event::SequenceEnd | Event::MappingEnd => return Err(Error::EndOfStream), // FIXME
        };
        if is_some {
            visitor.visit_some(self)
        } else {
            self.pos += 1;
            visitor.visit_none()
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
        match *self.peek()? {
            Event::MappingStart => {
                self.pos += 1;
                let value = visitor.visit_enum(VariantVisitor { de: self })?;
                match *self.next()? {
                    Event::MappingEnd => Ok(value),
                    _ => Err(Error::VariantMapWrongSize(name.to_owned(), 2)), // FIXME
                }
            }
            Event::Scalar(_, _, _) => {
                visitor.visit_enum(UnitVariantVisitor { de: self })
            }
            _ => Err(Error::VariantNotAMapOrString(name.to_owned())),
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
    let mut parser = Parser::new(s.chars());
    let mut loader = Loader {
        events: Vec::new(),
        aliases: BTreeMap::new(),
    };
    try!(parser.load(&mut loader, true));
    if loader.events.is_empty() {
        Err(Error::EndOfStream)
    } else {
        let mut deserializer = Deserializer {
            events: &loader.events,
            aliases: &loader.aliases,
            pos: 0,
        };
        let t = Deserialize::deserialize(&mut deserializer)?;
        if deserializer.pos == loader.events.len() {
            Ok(t)
        } else {
            Err(Error::MoreThanOneDocument)
        }
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
