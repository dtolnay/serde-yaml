//! YAML Serialization
//!
//! This module provides YAML serialization with the type `Serializer`.

use std::{fmt, io, num, str};

use yaml_rust::{yaml, Yaml, YamlEmitter};

use serde::ser;

use super::error::{Error, Result};
use private;

/// A structure for serializing Rust values into YAML.
pub struct Serializer<W> {
    writer: W,
}

impl<W> Serializer<W>
where
    W: io::Write,
{
    /// Creates a new YAML serializer.
    #[inline]
    pub fn new(writer: W) -> Serializer<W> {
        Serializer { writer }
    }

    /// Unwrap the `Writer` from the `Serializer`.
    pub fn into_inner(self) -> W {
        self.writer
    }
}

impl<'a, W> ser::Serializer for &'a mut Serializer<W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = &'a mut Serializer<W>;
    type SerializeTuple = &'a mut Serializer<W>;
    type SerializeTupleStruct = &'a mut Serializer<W>;
    type SerializeTupleVariant = &'a mut Serializer<W>;
    type SerializeMap = &'a mut Serializer<W>;
    type SerializeStruct = &'a mut Serializer<W>;
    type SerializeStructVariant = &'a mut Serializer<W>;
}

impl<'a, W> ser::SerializeSeq for &'a mut Serializer<W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        value.serialize(&mut self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeTuple for &'a mut Serializer<W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        value.serialize(&mut self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeTupleStruct for &'a mut Serializer<W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        value.serialize(&mut self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeTupleVariant for &'a mut Serializer<W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        value.serialize(&mut self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeMap for &'a mut Serializer<W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        key.serialize(&mut self)
    }

    #[inline]
    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        value.serialize(&mut self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}
