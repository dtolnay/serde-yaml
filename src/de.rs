use crate::error::{self, Error, ErrorImpl};
use crate::libyaml::error::Mark;
use crate::libyaml::parser::{Scalar, ScalarStyle};
use crate::libyaml::tag::Tag;
use crate::loader::{Document, Loader};
use crate::path::Path;
use serde::de::{
    self, Deserialize, DeserializeOwned, DeserializeSeed, Expected, IgnoredAny as Ignore,
    IntoDeserializer, Unexpected, Visitor,
};
use std::f64;
use std::fmt;
use std::io;
use std::marker::PhantomData;
use std::mem;
use std::num::ParseIntError;
use std::str;
use std::sync::Arc;

type Result<T, E = Error> = std::result::Result<T, E>;

/// A structure that deserializes YAML into Rust values.
///
/// # Examples
///
/// Deserializing a single document:
///
/// ```
/// use anyhow::Result;
/// use serde::Deserialize;
/// use serde_yaml::Value;
///
/// fn main() -> Result<()> {
///     let input = "k: 107\n";
///     let de = serde_yaml::Deserializer::from_str(input);
///     let value = Value::deserialize(de)?;
///     println!("{:?}", value);
///     Ok(())
/// }
/// ```
///
/// Deserializing multi-doc YAML:
///
/// ```
/// use anyhow::Result;
/// use serde::Deserialize;
/// use serde_yaml::Value;
///
/// fn main() -> Result<()> {
///     let input = "---\nk: 107\n...\n---\nj: 106\n";
///
///     for document in serde_yaml::Deserializer::from_str(input) {
///         let value = Value::deserialize(document)?;
///         println!("{:?}", value);
///     }
///
///     Ok(())
/// }
/// ```
pub struct Deserializer<'a> {
    progress: Progress<'a>,
}

pub(crate) enum Progress<'a> {
    Str(&'a str),
    Slice(&'a [u8]),
    Read(Box<dyn io::Read + 'a>),
    Iterable(Loader<'a>),
    Document(Document),
    Fail(Arc<ErrorImpl>),
}

impl<'a> Deserializer<'a> {
    /// Creates a YAML deserializer from a `&str`.
    pub fn from_str(s: &'a str) -> Self {
        let progress = Progress::Str(s);
        Deserializer { progress }
    }

    /// Creates a YAML deserializer from a `&[u8]`.
    pub fn from_slice(v: &'a [u8]) -> Self {
        let progress = Progress::Slice(v);
        Deserializer { progress }
    }

    /// Creates a YAML deserializer from an `io::Read`.
    ///
    /// Reader-based deserializers do not support deserializing borrowed types
    /// like `&str`, since the `std::io::Read` trait has no non-copying methods
    /// -- everything it does involves copying bytes out of the data source.
    pub fn from_reader<R>(rdr: R) -> Self
    where
        R: io::Read + 'a,
    {
        let progress = Progress::Read(Box::new(rdr));
        Deserializer { progress }
    }

    fn de<T>(self, f: impl FnOnce(&mut DeserializerFromEvents) -> Result<T>) -> Result<T> {
        match &self.progress {
            Progress::Iterable(_) => return Err(error::more_than_one_document()),
            Progress::Document(document) => {
                let mut pos = 0;
                let t = f(&mut DeserializerFromEvents {
                    document,
                    pos: &mut pos,
                    path: Path::Root,
                    remaining_depth: 128,
                })?;
                return Ok(t);
            }
            _ => {}
        }

        let mut loader = Loader::new(self.progress)?;
        let document = loader.next_document().ok_or_else(error::end_of_stream)?;
        let mut pos = 0;
        let t = f(&mut DeserializerFromEvents {
            document: &document,
            pos: &mut pos,
            path: Path::Root,
            remaining_depth: 128,
        })?;
        if loader.next_document().is_none() {
            Ok(t)
        } else {
            Err(error::more_than_one_document())
        }
    }
}

impl<'de> Iterator for Deserializer<'de> {
    type Item = Self;

    fn next(&mut self) -> Option<Self> {
        match &mut self.progress {
            Progress::Iterable(loader) => {
                let document = loader.next_document()?;
                return Some(Deserializer {
                    progress: Progress::Document(document),
                });
            }
            Progress::Document(_) => return None,
            Progress::Fail(err) => {
                return Some(Deserializer {
                    progress: Progress::Fail(Arc::clone(err)),
                });
            }
            _ => {}
        }

        let dummy = Progress::Str("");
        let input = mem::replace(&mut self.progress, dummy);
        match Loader::new(input) {
            Ok(loader) => {
                self.progress = Progress::Iterable(loader);
                self.next()
            }
            Err(err) => {
                let fail = err.shared();
                self.progress = Progress::Fail(Arc::clone(&fail));
                Some(Deserializer {
                    progress: Progress::Fail(fail),
                })
            }
        }
    }
}

impl<'de> de::Deserializer<'de> for Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.de(|state| state.deserialize_any(visitor))
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.de(|state| state.deserialize_bool(visitor))
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.de(|state| state.deserialize_i8(visitor))
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.de(|state| state.deserialize_i16(visitor))
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.de(|state| state.deserialize_i32(visitor))
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.de(|state| state.deserialize_i64(visitor))
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.de(|state| state.deserialize_i128(visitor))
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.de(|state| state.deserialize_u8(visitor))
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.de(|state| state.deserialize_u16(visitor))
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.de(|state| state.deserialize_u32(visitor))
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.de(|state| state.deserialize_u64(visitor))
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.de(|state| state.deserialize_u128(visitor))
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.de(|state| state.deserialize_f32(visitor))
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.de(|state| state.deserialize_f64(visitor))
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.de(|state| state.deserialize_char(visitor))
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.de(|state| state.deserialize_str(visitor))
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.de(|state| state.deserialize_string(visitor))
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.de(|state| state.deserialize_bytes(visitor))
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.de(|state| state.deserialize_byte_buf(visitor))
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.de(|state| state.deserialize_option(visitor))
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.de(|state| state.deserialize_unit(visitor))
    }

    fn deserialize_unit_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.de(|state| state.deserialize_unit_struct(name, visitor))
    }

    fn deserialize_newtype_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.de(|state| state.deserialize_newtype_struct(name, visitor))
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.de(|state| state.deserialize_seq(visitor))
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.de(|state| state.deserialize_tuple(len, visitor))
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.de(|state| state.deserialize_tuple_struct(name, len, visitor))
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.de(|state| state.deserialize_map(visitor))
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.de(|state| state.deserialize_struct(name, fields, visitor))
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.de(|state| state.deserialize_enum(name, variants, visitor))
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.de(|state| state.deserialize_identifier(visitor))
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.de(|state| state.deserialize_ignored_any(visitor))
    }
}

pub(crate) enum Event {
    Alias(usize),
    Scalar(Scalar),
    SequenceStart,
    SequenceEnd,
    MappingStart,
    MappingEnd,
}

struct DeserializerFromEvents<'a> {
    document: &'a Document,
    pos: &'a mut usize,
    path: Path<'a>,
    remaining_depth: u8,
}

impl<'a> DeserializerFromEvents<'a> {
    fn peek_event(&self) -> Result<&'a Event> {
        self.peek_event_mark().map(|(event, _mark)| event)
    }

    fn peek_event_mark(&self) -> Result<(&'a Event, Mark)> {
        match self.document.events.get(*self.pos) {
            Some((event, mark)) => Ok((event, *mark)),
            None => Err(match &self.document.error {
                Some(parse_error) => error::shared(Arc::clone(parse_error)),
                None => error::end_of_stream(),
            }),
        }
    }

    fn next_event(&mut self) -> Result<&'a Event> {
        self.next_event_mark().map(|(event, _mark)| event)
    }

    fn next_event_mark(&mut self) -> Result<(&'a Event, Mark)> {
        self.peek_event_mark().map(|(event, mark)| {
            *self.pos += 1;
            (event, mark)
        })
    }

    fn jump<'b>(&'b self, pos: &'b mut usize) -> Result<DeserializerFromEvents<'b>> {
        match self.document.aliases.get(pos) {
            Some(found) => {
                *pos = *found;
                Ok(DeserializerFromEvents {
                    document: self.document,
                    pos,
                    path: Path::Alias { parent: &self.path },
                    remaining_depth: self.remaining_depth,
                })
            }
            None => panic!("unresolved alias: {}", *pos),
        }
    }

    fn ignore_any(&mut self) -> Result<()> {
        enum Nest {
            Sequence,
            Mapping,
        }

        let mut stack = Vec::new();

        loop {
            match self.next_event()? {
                Event::Alias(_) | Event::Scalar(_) => {}
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
    }

    fn visit_sequence<'de, V>(&mut self, visitor: V, mark: Mark) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let (value, len) = self.recursion_check(mark, |de| {
            let mut seq = SeqAccess { de, len: 0 };
            let value = visitor.visit_seq(&mut seq)?;
            Ok((value, seq.len))
        })?;
        self.end_sequence(len)?;
        Ok(value)
    }

    fn visit_mapping<'de, V>(&mut self, visitor: V, mark: Mark) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let (value, len) = self.recursion_check(mark, |de| {
            let mut map = MapAccess {
                de,
                len: 0,
                key: None,
            };
            let value = visitor.visit_map(&mut map)?;
            Ok((value, map.len))
        })?;
        self.end_mapping(len)?;
        Ok(value)
    }

    fn end_sequence(&mut self, len: usize) -> Result<()> {
        let total = {
            let mut seq = SeqAccess { de: self, len };
            while de::SeqAccess::next_element::<Ignore>(&mut seq)?.is_some() {}
            seq.len
        };
        match self.next_event()? {
            Event::SequenceEnd => {}
            _ => panic!("expected a SequenceEnd event"),
        }
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
                len,
                key: None,
            };
            while de::MapAccess::next_entry::<Ignore, Ignore>(&mut map)?.is_some() {}
            map.len
        };
        match self.next_event()? {
            Event::MappingEnd => {}
            _ => panic!("expected a MappingEnd event"),
        }
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

    fn recursion_check<F: FnOnce(&mut Self) -> Result<T>, T>(
        &mut self,
        mark: Mark,
        f: F,
    ) -> Result<T> {
        let previous_depth = self.remaining_depth;
        self.remaining_depth = match previous_depth.checked_sub(1) {
            Some(depth) => depth,
            None => return Err(error::recursion_limit_exceeded(mark)),
        };
        let result = f(self);
        self.remaining_depth = previous_depth;
        result
    }
}

struct SeqAccess<'a: 'r, 'r> {
    de: &'r mut DeserializerFromEvents<'a>,
    len: usize,
}

impl<'de, 'a, 'r> de::SeqAccess<'de> for SeqAccess<'a, 'r> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        match self.de.peek_event()? {
            Event::SequenceEnd => Ok(None),
            _ => {
                let mut element_de = DeserializerFromEvents {
                    document: self.de.document,
                    pos: self.de.pos,
                    path: Path::Seq {
                        parent: &self.de.path,
                        index: self.len,
                    },
                    remaining_depth: self.de.remaining_depth,
                };
                self.len += 1;
                seed.deserialize(&mut element_de).map(Some)
            }
        }
    }
}

struct MapAccess<'a: 'r, 'r> {
    de: &'r mut DeserializerFromEvents<'a>,
    len: usize,
    key: Option<&'a [u8]>,
}

impl<'de, 'a, 'r> de::MapAccess<'de> for MapAccess<'a, 'r> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        match self.de.peek_event()? {
            Event::MappingEnd => Ok(None),
            Event::Scalar(scalar) => {
                self.len += 1;
                self.key = Some(&scalar.value);
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
        let mut value_de = DeserializerFromEvents {
            document: self.de.document,
            pos: self.de.pos,
            path: if let Some(key) = self.key.and_then(|key| str::from_utf8(key).ok()) {
                Path::Map {
                    parent: &self.de.path,
                    key,
                }
            } else {
                Path::Unknown {
                    parent: &self.de.path,
                }
            },
            remaining_depth: self.de.remaining_depth,
        };
        seed.deserialize(&mut value_de)
    }
}

struct EnumAccess<'a: 'r, 'r> {
    de: &'r mut DeserializerFromEvents<'a>,
    name: &'static str,
    tag: Option<&'static str>,
}

impl<'de, 'a, 'r> de::EnumAccess<'de> for EnumAccess<'a, 'r> {
    type Error = Error;
    type Variant = DeserializerFromEvents<'r>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        #[derive(Debug)]
        enum Nope {}

        struct BadKey {
            name: &'static str,
        }

        impl<'de> Visitor<'de> for BadKey {
            type Value = Nope;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "variant of enum `{}`", self.name)
            }
        }

        let variant = if let Some(tag) = self.tag {
            tag
        } else {
            match match self.de.next_event()? {
                Event::Scalar(scalar) => str::from_utf8(&scalar.value).ok(),
                _ => None,
            } {
                Some(variant) => variant,
                None => {
                    *self.de.pos -= 1;
                    let bad = BadKey { name: self.name };
                    return Err(de::Deserializer::deserialize_any(&mut *self.de, bad).unwrap_err());
                }
            }
        };

        let str_de = IntoDeserializer::<Error>::into_deserializer(variant);
        let ret = seed.deserialize(str_de)?;
        let variant_visitor = DeserializerFromEvents {
            document: self.de.document,
            pos: self.de.pos,
            path: Path::Map {
                parent: &self.de.path,
                key: variant,
            },
            remaining_depth: self.de.remaining_depth,
        };
        Ok((ret, variant_visitor))
    }
}

impl<'de, 'a> de::VariantAccess<'de> for DeserializerFromEvents<'a> {
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
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(&mut self, visitor)
    }

    fn struct_variant<V>(mut self, fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_struct(&mut self, "", fields, visitor)
    }
}

struct UnitVariantAccess<'a: 'r, 'r> {
    de: &'r mut DeserializerFromEvents<'a>,
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
        V: Visitor<'de>,
    {
        Err(de::Error::invalid_type(
            Unexpected::UnitVariant,
            &"tuple variant",
        ))
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(de::Error::invalid_type(
            Unexpected::UnitVariant,
            &"struct variant",
        ))
    }
}

fn visit_scalar<'de, V>(visitor: V, scalar: &Scalar) -> Result<V::Value>
where
    V: Visitor<'de>,
{
    let v = match str::from_utf8(&scalar.value) {
        Ok(v) => v,
        Err(_) => {
            return Err(de::Error::invalid_type(
                Unexpected::Bytes(&scalar.value),
                &visitor,
            ))
        }
    };
    if let Some(tag) = &scalar.tag {
        if tag == Tag::BOOL {
            match v.parse::<bool>() {
                Ok(v) => visitor.visit_bool(v),
                Err(_) => Err(de::Error::invalid_value(Unexpected::Str(v), &"a boolean")),
            }
        } else if tag == Tag::INT {
            match v.parse::<i64>() {
                Ok(v) => visitor.visit_i64(v),
                Err(_) => Err(de::Error::invalid_value(Unexpected::Str(v), &"an integer")),
            }
        } else if tag == Tag::FLOAT {
            match v.parse::<f64>() {
                Ok(v) => visitor.visit_f64(v),
                Err(_) => Err(de::Error::invalid_value(Unexpected::Str(v), &"a float")),
            }
        } else if tag == Tag::NULL {
            match parse_null(v.as_bytes()) {
                Some(()) => visitor.visit_unit(),
                None => Err(de::Error::invalid_value(Unexpected::Str(v), &"null")),
            }
        } else {
            visitor.visit_str(v)
        }
    } else if scalar.style == ScalarStyle::Plain {
        visit_untagged_str(visitor, v)
    } else {
        visitor.visit_str(v)
    }
}

fn parse_null(scalar: &[u8]) -> Option<()> {
    if scalar == b"~" || scalar == b"null" {
        Some(())
    } else {
        None
    }
}

fn parse_bool(scalar: &str) -> Option<bool> {
    if scalar == "true" {
        Some(true)
    } else if scalar == "false" {
        Some(false)
    } else {
        None
    }
}

fn parse_unsigned_int<T>(
    scalar: &str,
    from_str_radix: fn(&str, radix: u32) -> Result<T, ParseIntError>,
) -> Option<T> {
    let unpositive = scalar.strip_prefix('+').unwrap_or(scalar);
    if let Some(rest) = unpositive.strip_prefix("0x") {
        if let Ok(int) = from_str_radix(rest, 16) {
            return Some(int);
        }
    }
    if let Some(rest) = unpositive.strip_prefix("0o") {
        if let Ok(int) = from_str_radix(rest, 8) {
            return Some(int);
        }
    }
    if let Some(rest) = unpositive.strip_prefix("0b") {
        if let Ok(int) = from_str_radix(rest, 2) {
            return Some(int);
        }
    }
    if digits_but_not_number(scalar) {
        return None;
    }
    from_str_radix(unpositive, 10).ok()
}

fn parse_negative_int<T>(
    scalar: &str,
    from_str_radix: fn(&str, radix: u32) -> Result<T, ParseIntError>,
) -> Option<T> {
    if let Some(rest) = scalar.strip_prefix("-0x") {
        let negative = format!("-{}", rest);
        if let Ok(int) = from_str_radix(&negative, 16) {
            return Some(int);
        }
    }
    if let Some(rest) = scalar.strip_prefix("-0o") {
        let negative = format!("-{}", rest);
        if let Ok(int) = from_str_radix(&negative, 8) {
            return Some(int);
        }
    }
    if let Some(rest) = scalar.strip_prefix("-0b") {
        let negative = format!("-{}", rest);
        if let Ok(int) = from_str_radix(&negative, 2) {
            return Some(int);
        }
    }
    if digits_but_not_number(scalar) {
        return None;
    }
    from_str_radix(scalar, 10).ok()
}

fn digits_but_not_number(scalar: &str) -> bool {
    // Leading zero(s) followed by numeric characters is a string according to
    // the YAML 1.2 spec. https://yaml.org/spec/1.2/spec.html#id2761292
    let scalar = scalar.strip_prefix(['-', '+']).unwrap_or(scalar);
    scalar.len() > 1 && scalar.starts_with('0') && scalar[1..].bytes().all(|b| b.is_ascii_digit())
}

fn visit_untagged_str<'de, V>(visitor: V, v: &str) -> Result<V::Value>
where
    V: Visitor<'de>,
{
    if v.is_empty() || parse_null(v.as_bytes()) == Some(()) {
        return visitor.visit_unit();
    }
    if let Some(boolean) = parse_bool(v) {
        return visitor.visit_bool(boolean);
    }
    if let Some(int) = parse_unsigned_int(v, u64::from_str_radix) {
        return visitor.visit_u64(int);
    }
    if let Some(int) = parse_negative_int(v, i64::from_str_radix) {
        return visitor.visit_i64(int);
    }
    if let Some(int) = parse_unsigned_int(v, u128::from_str_radix) {
        return visitor.visit_u128(int);
    }
    if let Some(int) = parse_negative_int(v, i128::from_str_radix) {
        return visitor.visit_i128(int);
    }
    if digits_but_not_number(v) {
        return visitor.visit_str(v);
    }
    match v.strip_prefix('+').unwrap_or(v) {
        ".inf" | ".Inf" | ".INF" => return visitor.visit_f64(f64::INFINITY),
        _ => {}
    }
    if v == "-.inf" || v == "-.Inf" || v == "-.INF" {
        return visitor.visit_f64(f64::NEG_INFINITY);
    }
    if v == ".nan" || v == ".NaN" || v == ".NAN" {
        return visitor.visit_f64(f64::NAN);
    }
    if let Ok(n) = v.parse::<f64>() {
        if n.is_finite() {
            return visitor.visit_f64(n);
        }
    }
    visitor.visit_str(v)
}

fn invalid_type(event: &Event, exp: &dyn Expected) -> Error {
    enum Void {}

    struct InvalidType<'a> {
        exp: &'a dyn Expected,
    }

    impl<'de, 'a> Visitor<'de> for InvalidType<'a> {
        type Value = Void;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            self.exp.fmt(formatter)
        }
    }

    match event {
        Event::Alias(_) => unreachable!(),
        Event::Scalar(scalar) => {
            let get_type = InvalidType { exp };
            match visit_scalar(get_type, scalar) {
                Ok(void) => match void {},
                Err(invalid_type) => invalid_type,
            }
        }
        Event::SequenceStart => de::Error::invalid_type(Unexpected::Seq, exp),
        Event::MappingStart => de::Error::invalid_type(Unexpected::Map, exp),
        Event::SequenceEnd => panic!("unexpected end of sequence"),
        Event::MappingEnd => panic!("unexpected end of mapping"),
    }
}

impl<'a> DeserializerFromEvents<'a> {
    fn deserialize_scalar<'de, V>(&mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let (next, mark) = self.next_event_mark()?;
        match next {
            Event::Alias(mut pos) => self.jump(&mut pos)?.deserialize_scalar(visitor),
            Event::Scalar(scalar) => visit_scalar(visitor, scalar),
            other => Err(invalid_type(other, &visitor)),
        }
        .map_err(|err| error::fix_mark(err, mark, self.path))
    }
}

impl<'de, 'a, 'r> de::Deserializer<'de> for &'r mut DeserializerFromEvents<'a> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let (next, mark) = self.next_event_mark()?;
        match next {
            Event::Alias(mut pos) => self.jump(&mut pos)?.deserialize_any(visitor),
            Event::Scalar(scalar) => visit_scalar(visitor, scalar),
            Event::SequenceStart => self.visit_sequence(visitor, mark),
            Event::MappingStart => self.visit_mapping(visitor, mark),
            Event::SequenceEnd => panic!("unexpected end of sequence"),
            Event::MappingEnd => panic!("unexpected end of mapping"),
        }
        // The de::Error impl creates errors with unknown line and column. Fill
        // in the position here by looking at the current index in the input.
        .map_err(|err| error::fix_mark(err, mark, self.path))
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let (next, mark) = self.next_event_mark()?;
        loop {
            match next {
                Event::Alias(mut pos) => break self.jump(&mut pos)?.deserialize_bool(visitor),
                Event::Scalar(scalar) if scalar.style == ScalarStyle::Plain => {
                    if let Ok(value) = str::from_utf8(&scalar.value) {
                        if let Some(boolean) = parse_bool(value) {
                            break visitor.visit_bool(boolean);
                        }
                    }
                }
                _ => {}
            }
            break Err(invalid_type(next, &visitor));
        }
        .map_err(|err| error::fix_mark(err, mark, self.path))
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_scalar(visitor)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_scalar(visitor)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_scalar(visitor)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_scalar(visitor)
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_scalar(visitor)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_scalar(visitor)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_scalar(visitor)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_scalar(visitor)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_scalar(visitor)
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_scalar(visitor)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_scalar(visitor)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_scalar(visitor)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let (next, mark) = self.next_event_mark()?;
        match next {
            Event::Scalar(scalar) => {
                if let Ok(v) = str::from_utf8(&scalar.value) {
                    visitor.visit_str(v)
                } else {
                    Err(invalid_type(next, &visitor))
                }
            }
            Event::Alias(mut pos) => self.jump(&mut pos)?.deserialize_str(visitor),
            other => Err(invalid_type(other, &visitor)),
        }
        .map_err(|err: Error| error::fix_mark(err, mark, self.path))
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    /// Parses `null` as None and any other values as `Some(...)`.
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let is_some = match self.peek_event()? {
            Event::Alias(mut pos) => {
                *self.pos += 1;
                return self.jump(&mut pos)?.deserialize_option(visitor);
            }
            Event::Scalar(scalar) => {
                if scalar.style != ScalarStyle::Plain {
                    true
                } else if let Some(tag) = &scalar.tag {
                    if tag == Tag::NULL {
                        if let Some(()) = parse_null(&scalar.value) {
                            false
                        } else if let Ok(v) = str::from_utf8(&scalar.value) {
                            return Err(de::Error::invalid_value(Unexpected::Str(v), &"null"));
                        } else {
                            return Err(de::Error::invalid_value(
                                Unexpected::Bytes(&scalar.value),
                                &"null",
                            ));
                        }
                    } else {
                        true
                    }
                } else {
                    !scalar.value.is_empty() && parse_null(&scalar.value).is_none()
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

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_scalar(visitor)
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    /// Parses a newtype struct as the underlying value.
    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let (next, mark) = self.next_event_mark()?;
        match next {
            Event::Alias(mut pos) => self.jump(&mut pos)?.deserialize_seq(visitor),
            Event::SequenceStart => self.visit_sequence(visitor, mark),
            other => Err(invalid_type(other, &visitor)),
        }
        .map_err(|err| error::fix_mark(err, mark, self.path))
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let (next, mark) = self.next_event_mark()?;
        match next {
            Event::Alias(mut pos) => self.jump(&mut pos)?.deserialize_map(visitor),
            Event::MappingStart => self.visit_mapping(visitor, mark),
            other => Err(invalid_type(other, &visitor)),
        }
        .map_err(|err| error::fix_mark(err, mark, self.path))
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let (next, mark) = self.next_event_mark()?;
        match next {
            Event::Alias(mut pos) => self
                .jump(&mut pos)?
                .deserialize_struct(name, fields, visitor),
            Event::SequenceStart => self.visit_sequence(visitor, mark),
            Event::MappingStart => self.visit_mapping(visitor, mark),
            other => Err(invalid_type(other, &visitor)),
        }
        .map_err(|err| error::fix_mark(err, mark, self.path))
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
        V: Visitor<'de>,
    {
        let (next, mark) = self.peek_event_mark()?;
        match next {
            Event::Alias(mut pos) => {
                *self.pos += 1;
                self.jump(&mut pos)?
                    .deserialize_enum(name, variants, visitor)
            }
            Event::Scalar(scalar) => {
                if let Some((b'!', tag)) = scalar.tag.as_ref().and_then(|tag| tag.split_first()) {
                    if let Some(tag) = variants.iter().find(|v| v.as_bytes() == tag) {
                        return visitor.visit_enum(EnumAccess {
                            de: self,
                            name,
                            tag: Some(tag),
                        });
                    }
                }
                visitor.visit_enum(UnitVariantAccess { de: self })
            }
            Event::MappingStart => {
                *self.pos += 1;
                let value = visitor.visit_enum(EnumAccess {
                    de: self,
                    name,
                    tag: None,
                })?;
                self.end_mapping(1)?;
                Ok(value)
            }
            Event::SequenceStart => {
                let err = de::Error::invalid_type(Unexpected::Seq, &"string or singleton map");
                Err(error::fix_mark(err, mark, self.path))
            }
            Event::SequenceEnd => panic!("unexpected end of sequence"),
            Event::MappingEnd => panic!("unexpected end of mapping"),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.ignore_any()?;
        visitor.visit_unit()
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
    from_str_seed(s, PhantomData)
}

/// Deserialize an instance of type `T` from a string of YAML text with a seed.
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
pub fn from_str_seed<T, S>(s: &str, seed: S) -> Result<T>
where
    S: for<'de> DeserializeSeed<'de, Value = T>,
{
    seed.deserialize(Deserializer::from_str(s))
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
pub fn from_reader<R, T>(rdr: R) -> Result<T>
where
    R: io::Read,
    T: DeserializeOwned,
{
    from_reader_seed(rdr, PhantomData)
}

/// Deserialize an instance of type `T` from an IO stream of YAML with a seed.
///
/// This conversion can fail if the structure of the Value does not match the
/// structure expected by `T`, for example if `T` is a struct type but the Value
/// contains something other than a YAML map. It can also fail if the structure
/// is correct but `T`'s implementation of `Deserialize` decides that something
/// is wrong with the data, for example required struct fields are missing from
/// the YAML map or some number is too big to fit in the expected primitive
/// type.
pub fn from_reader_seed<R, T, S>(rdr: R, seed: S) -> Result<T>
where
    R: io::Read,
    S: for<'de> DeserializeSeed<'de, Value = T>,
{
    seed.deserialize(Deserializer::from_reader(rdr))
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
    from_slice_seed(v, PhantomData)
}

/// Deserialize an instance of type `T` from bytes of YAML text with a seed.
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
pub fn from_slice_seed<T, S>(v: &[u8], seed: S) -> Result<T>
where
    S: for<'de> DeserializeSeed<'de, Value = T>,
{
    seed.deserialize(Deserializer::from_slice(v))
}
