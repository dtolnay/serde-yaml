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
use std::fmt;
use std::io;
use std::str;

use yaml_rust::parser::{Parser, MarkedEventReceiver, Event as YamlEvent};
use yaml_rust::scanner::{Marker, TokenType, TScalarStyle};

use serde::de::{self, Deserialize, DeserializeSeed, Expected, Unexpected};
use serde::de::impls::IgnoredAny as Ignore;

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

#[derive(Debug, PartialEq)]
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

struct CollectionVisitor<'a: 'r, 'r> {
    de: &'r mut Deserializer<'a>,
    len: usize,
}

impl<'a, 'r> de::SeqVisitor for CollectionVisitor<'a, 'r> {
    type Error = Error;

    fn visit_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
        where T: DeserializeSeed
    {
        match *self.de.peek()? {
            Event::SequenceEnd => Ok(None),
            _ => {
                self.len += 1;
                seed.deserialize(&mut *self.de).map(Some)
            }
        }
    }
}

impl<'a, 'r> de::MapVisitor for CollectionVisitor<'a, 'r> {
    type Error = Error;

    fn visit_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
        where K: DeserializeSeed
    {
        match *self.de.peek()? {
            Event::MappingEnd => Ok(None),
            _ => {
                self.len += 1;
                seed.deserialize(&mut *self.de).map(Some)
            }
        }
    }

    fn visit_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
        where V: DeserializeSeed
    {
        seed.deserialize(&mut *self.de)
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
        Ok((seed.deserialize(&mut *self.de)?, self))
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
        Ok((seed.deserialize(&mut *self.de)?, self))
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

fn visit_untagged_str<V>(visitor: V, v: &str) -> Result<V::Value>
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
                                    Err(_) => Err(de::Error::invalid_value(Unexpected::Str(v), &"a boolean")),
                                }
                            },
                            "int" => {
                                match v.parse::<i64>() {
                                    Ok(v) => visitor.visit_i64(v),
                                    Err(_) => Err(de::Error::invalid_value(Unexpected::Str(v), &"an integer")),
                                }
                            },
                            "float" => {
                                match v.parse::<f64>() {
                                    Ok(v) => visitor.visit_f64(v),
                                    Err(_) => Err(de::Error::invalid_value(Unexpected::Str(v), &"a float")),
                                }
                            },
                            "null" => {
                                match v.as_ref() {
                                    "~" | "null" => visitor.visit_unit(),
                                    _ => Err(de::Error::invalid_value(Unexpected::Str(v), &"null")),
                                }
                            }
                            _  => visitor.visit_str(v),
                        }
                    } else {
                        visitor.visit_str(v)
                    }
                } else {
                    visit_untagged_str(visitor, v)
                }
            }
            Event::SequenceStart => {
                let (value, len, remaining) = {
                    let mut seq = CollectionVisitor { de: self, len: 0 };
                    let value = visitor.visit_seq(&mut seq)?;
                    let mut remaining = 0;
                    while de::SeqVisitor::visit::<Ignore>(&mut seq)?.is_some() {
                        remaining += 1;
                    }
                    (value, seq.len, remaining)
                };
                assert_eq!(Event::SequenceEnd, *self.next()?);
                if remaining == 0 {
                    Ok(value)
                } else {
                    struct ExpectedSeq(usize);
                    impl Expected for ExpectedSeq {
                        fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                            write!(formatter, "sequence of {} elements", self.0)
                        }
                    }
                    Err(de::Error::invalid_length(len + remaining, &ExpectedSeq(len)))
                }
            }
            Event::MappingStart => {
                let (value, len, remaining) = {
                    let mut map = CollectionVisitor { de: self, len: 0 };
                    let value = visitor.visit_map(&mut map)?;
                    let mut remaining = 0;
                    while de::MapVisitor::visit::<Ignore, Ignore>(&mut map)?.is_some() {
                        remaining += 1;
                    }
                    (value, map.len, remaining)
                };
                assert_eq!(Event::MappingEnd, *self.next()?);
                if remaining == 0 {
                    Ok(value)
                } else {
                    struct ExpectedMap(usize);
                    impl Expected for ExpectedMap {
                        fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                            write!(formatter, "map containing {} entries", self.0)
                        }
                    }
                    Err(de::Error::invalid_length(len + remaining, &ExpectedMap(len)))
                }
            }
            Event::SequenceEnd => panic!("unexpected end of sequence"),
            Event::MappingEnd => panic!("unexpected end of mapping"),
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
            Event::SequenceEnd => panic!("unexpected end of sequence"),
            Event::MappingEnd => panic!("unexpected end of mapping"),
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
    parser.load(&mut loader, true)?;
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
    from_str(str::from_utf8(&bytes)?)
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
