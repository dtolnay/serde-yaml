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
use std::f64;
use std::fmt;
use std::io;
use std::str;

use yaml_rust::parser::{Event as YamlEvent, MarkedEventReceiver, Parser};
use yaml_rust::scanner::{Marker, TScalarStyle, TokenType};

use serde::de::IgnoredAny as Ignore;
use serde::de::{
    self, Deserialize, DeserializeOwned, DeserializeSeed, Expected, IntoDeserializer, Unexpected,
};

use error::{Error, Result};
use path::Path;

pub struct Loader {
    events: Vec<(Event, Marker)>,
    /// Map from alias id to index in events.
    aliases: BTreeMap<usize, usize>,
}

impl MarkedEventReceiver for Loader {
    fn on_event(&mut self, event: YamlEvent, marker: Marker) {
        let event = match event {
            YamlEvent::Nothing
            | YamlEvent::StreamStart
            | YamlEvent::StreamEnd
            | YamlEvent::DocumentStart
            | YamlEvent::DocumentEnd => return,

            YamlEvent::Alias(id) => Event::Alias(id),
            YamlEvent::Scalar(value, style, id, tag) => {
                self.aliases.insert(id, self.events.len());
                Event::Scalar(value, style, tag)
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
    pos: &'a mut usize,
    path: Path<'a>,
}

impl<'a> Deserializer<'a> {
    fn peek(&self) -> Result<(&'a Event, Marker)> {
        match self.events.get(*self.pos) {
            Some(event) => Ok((&event.0, event.1)),
            None => Err(Error::end_of_stream()),
        }
    }

    fn next(&mut self) -> Result<(&'a Event, Marker)> {
        self.opt_next().ok_or_else(Error::end_of_stream)
    }

    fn opt_next(&mut self) -> Option<(&'a Event, Marker)> {
        self.events.get(*self.pos).map(|event| {
            *self.pos += 1;
            (&event.0, event.1)
        })
    }

    fn jump(&'a self, pos: &'a mut usize) -> Result<Deserializer<'a>> {
        match self.aliases.get(pos) {
            Some(&found) => {
                *pos = found;
                Ok(Deserializer {
                    events: self.events,
                    aliases: self.aliases,
                    pos: pos,
                    path: Path::Alias { parent: &self.path },
                })
            }
            None => panic!("unresolved alias: {}", *pos),
        }
    }

    fn visit<'de, V>(&mut self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match *self.next()?.0 {
            Event::Alias(i) => {
                let mut pos = i;
                de::Deserializer::deserialize_any(&mut self.jump(&mut pos)?, visitor)
            }
            Event::Scalar(ref v, style, ref tag) => {
                if style != TScalarStyle::Plain {
                    visitor.visit_str(v)
                } else if let Some(TokenType::Tag(ref handle, ref suffix)) = *tag {
                    if handle == "!!" {
                        match suffix.as_ref() {
                            "bool" => match v.parse::<bool>() {
                                Ok(v) => visitor.visit_bool(v),
                                Err(_) => {
                                    Err(de::Error::invalid_value(Unexpected::Str(v), &"a boolean"))
                                }
                            },
                            "int" => match v.parse::<i64>() {
                                Ok(v) => visitor.visit_i64(v),
                                Err(_) => {
                                    Err(de::Error::invalid_value(Unexpected::Str(v), &"an integer"))
                                }
                            },
                            "float" => match v.parse::<f64>() {
                                Ok(v) => visitor.visit_f64(v),
                                Err(_) => {
                                    Err(de::Error::invalid_value(Unexpected::Str(v), &"a float"))
                                }
                            },
                            "null" => match v.as_ref() {
                                "~" | "null" => visitor.visit_unit(),
                                _ => Err(de::Error::invalid_value(Unexpected::Str(v), &"null")),
                            },
                            _ => visitor.visit_str(v),
                        }
                    } else {
                        visitor.visit_str(v)
                    }
                } else {
                    visit_untagged_str(visitor, v)
                }
            }
            Event::SequenceStart => {
                let (value, len) = {
                    let mut seq = SeqAccess { de: self, len: 0 };
                    let value = visitor.visit_seq(&mut seq)?;
                    (value, seq.len)
                };
                self.end_sequence(len)?;
                Ok(value)
            }
            Event::MappingStart => {
                let (value, len) = {
                    let mut map = MapAccess {
                        de: &mut *self,
                        len: 0,
                        key: None,
                    };
                    let value = visitor.visit_map(&mut map)?;
                    (value, map.len)
                };
                self.end_mapping(len)?;
                Ok(value)
            }
            Event::SequenceEnd => panic!("unexpected end of sequence"),
            Event::MappingEnd => panic!("unexpected end of mapping"),
        }
    }

    fn ignore_any(&mut self) -> Result<()> {
        enum Nest {
            Sequence,
            Mapping,
        }

        let mut stack = Vec::new();

        while let Some((event, _)) = self.opt_next() {
            match *event {
                Event::Alias(_) | Event::Scalar(_, _, _) => {}
                Event::SequenceStart => {
                    stack.push(Nest::Sequence);
                }
                Event::MappingStart => {
                    stack.push(Nest::Mapping);
                }
                Event::SequenceEnd => match stack.pop() {
                    Some(Nest::Sequence) => {}
                    None | Some(Nest::Mapping) => {
                        panic!("unexpected end of sequence");
                    }
                },
                Event::MappingEnd => match stack.pop() {
                    Some(Nest::Mapping) => {}
                    None | Some(Nest::Sequence) => {
                        panic!("unexpected end of mapping");
                    }
                },
            }
            if stack.is_empty() {
                return Ok(());
            }
        }

        if !stack.is_empty() {
            panic!("missing end event");
        }

        Ok(())
    }

    fn end_sequence(&mut self, len: usize) -> Result<()> {
        let total = {
            let mut seq = SeqAccess { de: self, len: len };
            while de::SeqAccess::next_element::<Ignore>(&mut seq)?.is_some() {}
            seq.len
        };
        assert_eq!(Event::SequenceEnd, *self.next()?.0);
        if total == len {
            Ok(())
        } else {
            struct ExpectedSeq(usize);
            impl Expected for ExpectedSeq {
                fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    if self.0 == 1 {
                        write!(formatter, "sequence of 1 element")
                    } else {
                        write!(formatter, "sequence of {} elements", self.0)
                    }
                }
            }
            Err(de::Error::invalid_length(total, &ExpectedSeq(len)))
        }
    }

    fn end_mapping(&mut self, len: usize) -> Result<()> {
        let total = {
            let mut map = MapAccess {
                de: self,
                len: len,
                key: None,
            };
            while de::MapAccess::next_entry::<Ignore, Ignore>(&mut map)?.is_some() {}
            map.len
        };
        assert_eq!(Event::MappingEnd, *self.next()?.0);
        if total == len {
            Ok(())
        } else {
            struct ExpectedMap(usize);
            impl Expected for ExpectedMap {
                fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    if self.0 == 1 {
                        write!(formatter, "map containing 1 entry")
                    } else {
                        write!(formatter, "map containing {} entries", self.0)
                    }
                }
            }
            Err(de::Error::invalid_length(total, &ExpectedMap(len)))
        }
    }
}

struct SeqAccess<'a: 'r, 'r> {
    de: &'r mut Deserializer<'a>,
    len: usize,
}

impl<'de, 'a, 'r> de::SeqAccess<'de> for SeqAccess<'a, 'r> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        match *self.de.peek()?.0 {
            Event::SequenceEnd => Ok(None),
            _ => {
                let mut element_de = Deserializer {
                    events: self.de.events,
                    aliases: self.de.aliases,
                    pos: self.de.pos,
                    path: Path::Seq {
                        parent: &self.de.path,
                        index: self.len,
                    },
                };
                self.len += 1;
                seed.deserialize(&mut element_de).map(Some)
            }
        }
    }
}

struct MapAccess<'a: 'r, 'r> {
    de: &'r mut Deserializer<'a>,
    len: usize,
    key: Option<&'a str>,
}

impl<'de, 'a, 'r> de::MapAccess<'de> for MapAccess<'a, 'r> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        match *self.de.peek()?.0 {
            Event::MappingEnd => Ok(None),
            Event::Scalar(ref key, _, _) => {
                self.len += 1;
                self.key = Some(key);
                seed.deserialize(&mut *self.de).map(Some)
            }
            _ => {
                self.len += 1;
                self.key = None;
                seed.deserialize(&mut *self.de).map(Some)
            }
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        let mut value_de = Deserializer {
            events: self.de.events,
            aliases: self.de.aliases,
            pos: self.de.pos,
            path: if let Some(key) = self.key {
                Path::Map {
                    parent: &self.de.path,
                    key: key,
                }
            } else {
                Path::Unknown {
                    parent: &self.de.path,
                }
            },
        };
        seed.deserialize(&mut value_de)
    }
}

struct EnumAccess<'a: 'r, 'r> {
    de: &'r mut Deserializer<'a>,
    name: &'static str,
}

impl<'de, 'a, 'r> de::EnumAccess<'de> for EnumAccess<'a, 'r> {
    type Error = Error;
    type Variant = Deserializer<'r>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        #[derive(Debug)]
        enum Nope {}

        struct BadKey {
            name: &'static str,
        }

        impl<'de> de::Visitor<'de> for BadKey {
            type Value = Nope;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "variant of enum `{}`", self.name)
            }
        }

        let variant = match *self.de.next()?.0 {
            Event::Scalar(ref s, _, _) => &**s,
            _ => {
                *self.de.pos -= 1;
                let bad = BadKey { name: self.name };
                return Err(de::Deserializer::deserialize_any(&mut *self.de, bad).unwrap_err());
            }
        };

        let str_de = IntoDeserializer::<Error>::into_deserializer(variant);
        let ret = seed.deserialize(str_de)?;
        let variant_visitor = Deserializer {
            events: self.de.events,
            aliases: self.de.aliases,
            pos: self.de.pos,
            path: Path::Map {
                parent: &self.de.path,
                key: variant,
            },
        };
        Ok((ret, variant_visitor))
    }
}

impl<'de, 'a> de::VariantAccess<'de> for Deserializer<'a> {
    type Error = Error;

    fn unit_variant(mut self) -> Result<()> {
        Deserialize::deserialize(&mut self)
    }

    fn newtype_variant_seed<T>(mut self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut self)
    }

    fn tuple_variant<V>(mut self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        de::Deserializer::deserialize_any(&mut self, visitor)
    }

    fn struct_variant<V>(mut self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        de::Deserializer::deserialize_any(&mut self, visitor)
    }
}

struct UnitVariantAccess<'a: 'r, 'r> {
    de: &'r mut Deserializer<'a>,
}

impl<'de, 'a, 'r> de::EnumAccess<'de> for UnitVariantAccess<'a, 'r> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        Ok((seed.deserialize(&mut *self.de)?, self))
    }
}

impl<'de, 'a, 'r> de::VariantAccess<'de> for UnitVariantAccess<'a, 'r> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        Err(de::Error::invalid_type(
            Unexpected::UnitVariant,
            &"newtype variant",
        ))
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(de::Error::invalid_type(
            Unexpected::UnitVariant,
            &"tuple variant",
        ))
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(de::Error::invalid_type(
            Unexpected::UnitVariant,
            &"struct variant",
        ))
    }
}

fn visit_untagged_str<'de, V>(visitor: V, v: &str) -> Result<V::Value>
where
    V: de::Visitor<'de>,
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
        if let Ok(n) = u64::from_str_radix(&v[2..], 16) {
            return visitor.visit_u64(n);
        }
        if let Ok(n) = i64::from_str_radix(&v[2..], 16) {
            return visitor.visit_i64(n);
        }
    }
    if v.starts_with("0o") {
        if let Ok(n) = u64::from_str_radix(&v[2..], 8) {
            return visitor.visit_u64(n);
        }
        if let Ok(n) = i64::from_str_radix(&v[2..], 8) {
            return visitor.visit_i64(n);
        }
    }
    if v.starts_with('+') {
        if let Ok(n) = v.parse() {
            return visitor.visit_u64(n);
        }
        if let Ok(n) = v[1..].parse() {
            return visitor.visit_i64(n);
        }
    }
    if let Ok(n) = v.parse() {
        return visitor.visit_u64(n);
    }
    if let Ok(n) = v.parse() {
        return visitor.visit_i64(n);
    }
    match v.trim_left_matches('+') {
        ".inf" | ".Inf" | ".INF" => return visitor.visit_f64(f64::INFINITY),
        _ => (),
    }
    if v == "-.inf" || v == "-.Inf" || v == "-.INF" {
        return visitor.visit_f64(f64::NEG_INFINITY);
    }
    if v == ".nan" || v == ".NaN" || v == ".NAN" {
        return visitor.visit_f64(f64::NAN);
    }
    if let Ok(n) = v.parse() {
        return visitor.visit_f64(n);
    }
    visitor.visit_str(v)
}

impl<'de, 'a, 'r> de::Deserializer<'de> for &'r mut Deserializer<'a> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let marker = self.peek()?.1;
        // The de::Error impl creates errors with unknown line and column. Fill
        // in the position here by looking at the current index in the input.
        self.visit(visitor)
            .map_err(|err| err.fix_marker(marker, self.path))
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let (next, marker) = self.peek()?;
        match *next {
            Event::Scalar(ref v, _, _) => {
                *self.pos += 1;
                visitor
                    .visit_str(v)
                    .map_err(|err: Error| err.fix_marker(marker, self.path))
            }
            Event::Alias(i) => {
                *self.pos += 1;
                let mut pos = i;
                self.jump(&mut pos)?.deserialize_str(visitor)
            }
            _ => self.deserialize_any(visitor),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    /// Parses `null` as None and any other values as `Some(...)`.
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let is_some = match *self.peek()?.0 {
            Event::Alias(i) => {
                *self.pos += 1;
                let mut pos = i;
                return self.jump(&mut pos)?.deserialize_option(visitor);
            }
            Event::Scalar(ref v, style, ref tag) => {
                if style != TScalarStyle::Plain {
                    true
                } else if let Some(TokenType::Tag(ref handle, ref suffix)) = *tag {
                    if handle == "!!" && suffix == "null" {
                        if v == "~" || v == "null" {
                            false
                        } else {
                            return Err(de::Error::invalid_value(Unexpected::Str(v), &"null"));
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
            *self.pos += 1;
            visitor.visit_none()
        }
    }

    /// Parses a newtype struct as the underlying value.
    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    /// Parses an enum as a single key:value pair where the key identifies the
    /// variant and the value gives the content. A String will also parse correctly
    /// to a unit enum value.
    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let (next, marker) = self.peek()?;
        match *next {
            Event::Alias(i) => {
                *self.pos += 1;
                let mut pos = i;
                self.jump(&mut pos)?
                    .deserialize_enum(name, variants, visitor)
            }
            Event::Scalar(_, _, _) => visitor.visit_enum(UnitVariantAccess { de: self }),
            Event::MappingStart => {
                *self.pos += 1;
                let value = visitor.visit_enum(EnumAccess {
                    de: self,
                    name: name,
                })?;
                self.end_mapping(1)?;
                Ok(value)
            }
            Event::SequenceStart => {
                let err = de::Error::invalid_type(Unexpected::Seq, &"string or singleton map");
                Err(Error::fix_marker(err, marker, self.path))
            }
            Event::SequenceEnd => panic!("unexpected end of sequence"),
            Event::MappingEnd => panic!("unexpected end of mapping"),
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.ignore_any()?;
        visitor.visit_unit()
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char bytes byte_buf unit
        unit_struct seq tuple tuple_struct map struct identifier
    }
}

/// Deserialize an instance of type `T` from a string of YAML text.
///
/// This conversion can fail if the structure of the Value does not match the
/// structure expected by `T`, for example if `T` is a struct type but the Value
/// contains something other than a YAML map. It can also fail if the structure
/// is correct but `T`'s implementation of `Deserialize` decides that something
/// is wrong with the data, for example required struct fields are missing from
/// the YAML map or some number is too big to fit in the expected primitive
/// type.
///
/// YAML currently does not support zero-copy deserialization.
pub fn from_str<T>(s: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    let mut parser = Parser::new(s.chars());
    let mut loader = Loader {
        events: Vec::new(),
        aliases: BTreeMap::new(),
    };
    parser.load(&mut loader, true).map_err(Error::scanner)?;
    if loader.events.is_empty() {
        Err(Error::end_of_stream())
    } else {
        let mut pos = 0;
        let t = Deserialize::deserialize(&mut Deserializer {
            events: &loader.events,
            aliases: &loader.aliases,
            pos: &mut pos,
            path: Path::Root,
        })?;
        if pos == loader.events.len() {
            Ok(t)
        } else {
            Err(Error::more_than_one_document())
        }
    }
}

/// Deserialize an instance of type `T` from an IO stream of YAML.
///
/// This conversion can fail if the structure of the Value does not match the
/// structure expected by `T`, for example if `T` is a struct type but the Value
/// contains something other than a YAML map. It can also fail if the structure
/// is correct but `T`'s implementation of `Deserialize` decides that something
/// is wrong with the data, for example required struct fields are missing from
/// the YAML map or some number is too big to fit in the expected primitive
/// type.
pub fn from_reader<R, T>(mut rdr: R) -> Result<T>
where
    R: io::Read,
    T: DeserializeOwned,
{
    let mut bytes = Vec::new();
    rdr.read_to_end(&mut bytes).map_err(Error::io)?;
    let s = str::from_utf8(&bytes).map_err(Error::str_utf8)?;
    from_str(s)
}

/// Deserialize an instance of type `T` from bytes of YAML text.
///
/// This conversion can fail if the structure of the Value does not match the
/// structure expected by `T`, for example if `T` is a struct type but the Value
/// contains something other than a YAML map. It can also fail if the structure
/// is correct but `T`'s implementation of `Deserialize` decides that something
/// is wrong with the data, for example required struct fields are missing from
/// the YAML map or some number is too big to fit in the expected primitive
/// type.
///
/// YAML currently does not support zero-copy deserialization.
pub fn from_slice<T>(v: &[u8]) -> Result<T>
where
    T: DeserializeOwned,
{
    let s = str::from_utf8(v).map_err(Error::str_utf8)?;
    from_str(s)
}
