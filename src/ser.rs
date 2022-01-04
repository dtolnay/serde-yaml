//! YAML Serialization
//!
//! This module provides YAML serialization with the type `Serializer`.

use crate::yamlformat::{Format, MemberId, YamlFormat, YamlFormatType};
use crate::{error, Error, Result};
use serde::ser;
use std::{fmt, io, num, str};
use yaml_rust::{yaml, Yaml, YamlEmitter};

/// A structure for serializing Rust values into YAML.
///
/// # Example
///
/// ```
/// use anyhow::Result;
/// use serde::Serialize;
/// use std::collections::BTreeMap;
///
/// fn main() -> Result<()> {
///     let mut buffer = Vec::new();
///     let mut ser = serde_yaml::Serializer::new(&mut buffer);
///
///     let mut object = BTreeMap::new();
///     object.insert("k", 107);
///     object.serialize(&mut ser)?;
///
///     object.insert("J", 74);
///     object.serialize(&mut ser)?;
///
///     assert_eq!(buffer, b"---\nk: 107\n...\n---\nJ: 74\nk: 107\n");
///     Ok(())
/// }
/// ```
pub struct Serializer<'f, W> {
    yamlformat: Option<&'f dyn YamlFormat>,
    documents: usize,
    writer: W,
}

impl<'f, W> Serializer<'f, W>
where
    W: io::Write,
{
    /// Creates a new YAML serializer.
    pub fn new(writer: W) -> Self {
        Serializer {
            yamlformat: None,
            documents: 0,
            writer,
        }
    }

    /// Creates a new YAML serializer with formatting info.
    pub fn new_with_format(writer: W, yamlformat: Option<&'f dyn YamlFormat>) -> Self {
        Serializer {
            yamlformat: yamlformat,
            documents: 0,
            writer,
        }
    }

    /// Unwrap the underlying `io::Write` object from the `Serializer`.
    pub fn into_inner(self) -> W {
        self.writer
    }

    fn write(&mut self, doc: Yaml) -> Result<()> {
        if self.documents > 0 {
            self.writer.write_all(b"...\n").map_err(error::io)?;
        }
        self.documents += 1;
        let mut writer_adapter = FmtToIoWriter {
            writer: &mut self.writer,
        };
        YamlEmitter::new(&mut writer_adapter)
            .dump(&doc)
            .map_err(error::emitter)?;
        writer_adapter.writer.write_all(b"\n").map_err(error::io)?;
        Ok(())
    }
}

impl<'a, 'f, W> ser::Serializer for &'a mut Serializer<'f, W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = ThenWrite<'a, 'f, W, SerializeArray<'f>>;
    type SerializeTuple = ThenWrite<'a, 'f, W, SerializeArray<'f>>;
    type SerializeTupleStruct = ThenWrite<'a, 'f, W, SerializeArray<'f>>;
    type SerializeTupleVariant = ThenWrite<'a, 'f, W, SerializeTupleVariant<'f>>;
    type SerializeMap = ThenWrite<'a, 'f, W, SerializeMap>;
    type SerializeStruct = ThenWrite<'a, 'f, W, SerializeStruct<'f>>;
    type SerializeStructVariant = ThenWrite<'a, 'f, W, SerializeStructVariant<'f>>;

    fn serialize_bool(self, v: bool) -> Result<()> {
        let doc = SerializerToYaml::default().serialize_bool(v)?;
        self.write(doc)
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        let doc = SerializerToYaml::default().serialize_i8(v)?;
        self.write(doc)
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        let doc = SerializerToYaml::default().serialize_i16(v)?;
        self.write(doc)
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        let doc = SerializerToYaml::default().serialize_i32(v)?;
        self.write(doc)
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        let doc = SerializerToYaml::default().serialize_i64(v)?;
        self.write(doc)
    }

    fn serialize_i128(self, v: i128) -> Result<()> {
        let doc = SerializerToYaml::default().serialize_i128(v)?;
        self.write(doc)
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        let doc = SerializerToYaml::default().serialize_u8(v)?;
        self.write(doc)
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        let doc = SerializerToYaml::default().serialize_u16(v)?;
        self.write(doc)
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        let doc = SerializerToYaml::default().serialize_u32(v)?;
        self.write(doc)
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        let doc = SerializerToYaml::default().serialize_u64(v)?;
        self.write(doc)
    }

    fn serialize_u128(self, v: u128) -> Result<()> {
        let doc = SerializerToYaml::default().serialize_u128(v)?;
        self.write(doc)
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        let doc = SerializerToYaml::default().serialize_f32(v)?;
        self.write(doc)
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        let doc = SerializerToYaml::default().serialize_f64(v)?;
        self.write(doc)
    }

    fn serialize_char(self, value: char) -> Result<()> {
        let doc = SerializerToYaml::default().serialize_char(value)?;
        self.write(doc)
    }

    fn serialize_str(self, value: &str) -> Result<()> {
        let doc = SerializerToYaml::default().serialize_str(value)?;
        self.write(doc)
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<()> {
        let doc = SerializerToYaml::default().serialize_bytes(value)?;
        self.write(doc)
    }

    fn serialize_unit(self) -> Result<()> {
        let doc = SerializerToYaml::default().serialize_unit()?;
        self.write(doc)
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<()> {
        let doc = SerializerToYaml::default().serialize_unit_struct(name)?;
        self.write(doc)
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        let doc =
            SerializerToYaml::default().serialize_unit_variant(name, variant_index, variant)?;
        self.write(doc)
    }

    fn serialize_newtype_struct<T: ?Sized>(self, name: &'static str, value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        let doc = SerializerToYaml::default().serialize_newtype_struct(name, value)?;
        self.write(doc)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ser::Serialize,
    {
        let doc = SerializerToYaml::new(self.yamlformat, None).serialize_newtype_variant(
            name,
            variant_index,
            variant,
            value,
        )?;
        self.write(doc)
    }

    fn serialize_none(self) -> Result<()> {
        let doc = SerializerToYaml::default().serialize_none()?;
        self.write(doc)
    }

    fn serialize_some<V: ?Sized>(self, value: &V) -> Result<()>
    where
        V: ser::Serialize,
    {
        let doc = SerializerToYaml::default().serialize_some(value)?;
        self.write(doc)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        let delegate = SerializerToYaml::default().serialize_seq(len)?;
        Ok(ThenWrite::new(self, delegate))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        let delegate = SerializerToYaml::default().serialize_tuple(len)?;
        Ok(ThenWrite::new(self, delegate))
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        let delegate = SerializerToYaml::default().serialize_tuple_struct(name, len)?;
        Ok(ThenWrite::new(self, delegate))
    }

    fn serialize_tuple_variant(
        self,
        enm: &'static str,
        idx: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        let delegate =
            SerializerToYaml::default().serialize_tuple_variant(enm, idx, variant, len)?;
        Ok(ThenWrite::new(self, delegate))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        let delegate = SerializerToYaml::default().serialize_map(len)?;
        Ok(ThenWrite::new(self, delegate))
    }

    fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        let delegate = SerializerToYaml::default().serialize_struct(name, len)?;
        Ok(ThenWrite::new(self, delegate))
    }

    fn serialize_struct_variant(
        self,
        enm: &'static str,
        idx: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        let delegate =
            SerializerToYaml::default().serialize_struct_variant(enm, idx, variant, len)?;
        Ok(ThenWrite::new(self, delegate))
    }
}

pub struct ThenWrite<'a, 'f, W, D> {
    serializer: &'a mut Serializer<'f, W>,
    delegate: D,
}

impl<'a, 'f, W, D> ThenWrite<'a, 'f, W, D> {
    fn new(serializer: &'a mut Serializer<'f, W>, delegate: D) -> Self {
        ThenWrite {
            serializer,
            delegate,
        }
    }
}

impl<'a, 'f, W> ser::SerializeSeq for ThenWrite<'a, 'f, W, SerializeArray<'f>>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, elem: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        self.delegate.serialize_element(elem)
    }

    fn end(self) -> Result<()> {
        let doc = self.delegate.end()?;
        self.serializer.write(doc)
    }
}

impl<'a, 'f, W> ser::SerializeTuple for ThenWrite<'a, 'f, W, SerializeArray<'f>>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, elem: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        self.delegate.serialize_element(elem)
    }

    fn end(self) -> Result<()> {
        let doc = self.delegate.end()?;
        self.serializer.write(doc)
    }
}

impl<'a, 'f, W> ser::SerializeTupleStruct for ThenWrite<'a, 'f, W, SerializeArray<'f>>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<V: ?Sized>(&mut self, value: &V) -> Result<()>
    where
        V: ser::Serialize,
    {
        let field = MemberId::Index(self.delegate.array.len() as u32);
        self.delegate.serializer.format = self
            .serializer
            .yamlformat
            .map(|y| y.format(None, &field))
            .flatten();
        self.delegate.serializer.comment = self
            .serializer
            .yamlformat
            .map(|y| y.comment(None, &field))
            .flatten();
        self.delegate.serialize_field(value)
    }

    fn end(self) -> Result<()> {
        let doc = self.delegate.end()?;
        self.serializer.write(doc)
    }
}

impl<'a, 'f, W> ser::SerializeTupleVariant for ThenWrite<'a, 'f, W, SerializeTupleVariant<'f>>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<V: ?Sized>(&mut self, v: &V) -> Result<()>
    where
        V: ser::Serialize,
    {
        let field = MemberId::Index(self.delegate.array.len() as u32);
        self.delegate.serializer.format = self
            .serializer
            .yamlformat
            .map(|y| y.format(Some(self.delegate.name), &field))
            .flatten();
        self.delegate.serializer.comment = self
            .serializer
            .yamlformat
            .map(|y| y.comment(Some(self.delegate.name), &field))
            .flatten();
        self.delegate.serialize_field(v)
    }

    fn end(mut self) -> Result<()> {
        self.delegate.serializer.format = self
            .serializer
            .yamlformat
            .map(|y| y.format(Some(self.delegate.name), &MemberId::Variant))
            .flatten();
        self.delegate.serializer.comment = self
            .serializer
            .yamlformat
            .map(|y| y.comment(Some(self.delegate.name), &MemberId::Variant))
            .flatten();
        let doc = self.delegate.end()?;
        self.serializer.write(doc)
    }
}

impl<'a, 'f, W> ser::SerializeMap for ThenWrite<'a, 'f, W, SerializeMap>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        self.delegate.serialize_key(key)
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        self.delegate.serialize_value(value)
    }

    fn serialize_entry<K: ?Sized, V: ?Sized>(&mut self, key: &K, value: &V) -> Result<()>
    where
        K: ser::Serialize,
        V: ser::Serialize,
    {
        self.delegate.serialize_entry(key, value)
    }

    fn end(self) -> Result<()> {
        let doc = self.delegate.end()?;
        self.serializer.write(doc)
    }
}

impl<'a, 'f, W> ser::SerializeStruct for ThenWrite<'a, 'f, W, SerializeStruct<'f>>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<V: ?Sized>(&mut self, key: &'static str, value: &V) -> Result<()>
    where
        V: ser::Serialize,
    {
        let field = MemberId::Name(key);
        self.delegate.serializer.format = self
            .serializer
            .yamlformat
            .map(|y| y.format(None, &field))
            .flatten();
        self.delegate.serializer.comment = self
            .serializer
            .yamlformat
            .map(|y| y.comment(None, &field))
            .flatten();

        self.delegate.serialize_field(key, value)
    }

    fn end(self) -> Result<()> {
        let doc = self.delegate.end()?;
        self.serializer.write(doc)
    }
}

impl<'a, 'f, W> ser::SerializeStructVariant for ThenWrite<'a, 'f, W, SerializeStructVariant<'f>>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<V: ?Sized>(&mut self, field: &'static str, v: &V) -> Result<()>
    where
        V: ser::Serialize,
    {
        let fld = MemberId::Name(field);
        self.delegate.serializer.format = self
            .serializer
            .yamlformat
            .map(|y| y.format(Some(self.delegate.name), &fld))
            .flatten();
        self.delegate.serializer.comment = self
            .serializer
            .yamlformat
            .map(|y| y.comment(Some(self.delegate.name), &fld))
            .flatten();
        self.delegate.serialize_field(self.delegate.name, v)
    }

    fn end(mut self) -> Result<()> {
        self.delegate.serializer.format = self
            .serializer
            .yamlformat
            .map(|y| y.format(Some(self.delegate.name), &MemberId::Variant))
            .flatten();
        self.delegate.serializer.comment = self
            .serializer
            .yamlformat
            .map(|y| y.comment(Some(self.delegate.name), &MemberId::Variant))
            .flatten();
        let doc = self.delegate.end()?;
        self.serializer.write(doc)
    }
}

pub struct SerializerToYaml<'f> {
    yamlformat: Option<&'f dyn YamlFormat>,
    format: Option<Format>,
    comment: Option<String>,
}

impl<'f> SerializerToYaml<'f> {
    pub fn new(yf: Option<&'f dyn YamlFormat>, format: Option<Format>) -> Self {
        SerializerToYaml {
            yamlformat: yf,
            format: format,
            comment: None,
        }
    }
}

impl<'f> Default for SerializerToYaml<'f> {
    fn default() -> Self {
        SerializerToYaml {
            yamlformat: None,
            format: None,
            comment: None,
        }
    }
}

impl<'f> ser::Serializer for SerializerToYaml<'f> {
    type Ok = Yaml;
    type Error = Error;

    type SerializeSeq = SerializeArray<'f>;
    type SerializeTuple = SerializeArray<'f>;
    type SerializeTupleStruct = SerializeArray<'f>;
    type SerializeTupleVariant = SerializeTupleVariant<'f>;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeStruct<'f>;
    type SerializeStructVariant = SerializeStructVariant<'f>;

    fn serialize_bool(self, v: bool) -> Result<Yaml> {
        Ok(Yaml::Boolean(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Yaml> {
        match self.format {
            None => Ok(Yaml::Integer(v as i64)),
            Some(Format::Binary) => Ok(Yaml::Real(format!("{:#010b}", v))),
            Some(Format::Decimal) => Ok(Yaml::Integer(v as i64)),
            Some(Format::Hex) => Ok(Yaml::Real(format!("{:#04x}", v))),
            Some(Format::Octal) => Ok(Yaml::Real(format!("{:#05o}", v))),
            _ => Err(error::format(format!(
                "Format {:?} not supported for type i8",
                self.format
            ))),
        }
    }

    fn serialize_i16(self, v: i16) -> Result<Yaml> {
        match self.format {
            None => Ok(Yaml::Integer(v as i64)),
            Some(Format::Binary) => Ok(Yaml::Real(format!("{:#018b}", v))),
            Some(Format::Decimal) => Ok(Yaml::Integer(v as i64)),
            Some(Format::Hex) => Ok(Yaml::Real(format!("{:#06x}", v))),
            Some(Format::Octal) => Ok(Yaml::Real(format!("{:#08o}", v))),
            _ => Err(error::format(format!(
                "Format {:?} not supported for type i16",
                self.format
            ))),
        }
    }

    fn serialize_i32(self, v: i32) -> Result<Yaml> {
        match self.format {
            None => Ok(Yaml::Integer(v as i64)),
            Some(Format::Binary) => Ok(Yaml::Real(format!("{:#034b}", v))),
            Some(Format::Decimal) => Ok(Yaml::Integer(v as i64)),
            Some(Format::Hex) => Ok(Yaml::Real(format!("{:#010x}", v))),
            Some(Format::Octal) => Ok(Yaml::Real(format!("{:#013o}", v))),
            _ => Err(error::format(format!(
                "Format {:?} not supported for type i32",
                self.format
            ))),
        }
    }

    fn serialize_i64(self, v: i64) -> Result<Yaml> {
        match self.format {
            None => Ok(Yaml::Integer(v)),
            Some(Format::Binary) => Ok(Yaml::Real(format!("{:#066b}", v))),
            Some(Format::Decimal) => Ok(Yaml::Integer(v)),
            Some(Format::Hex) => Ok(Yaml::Real(format!("{:#018x}", v))),
            Some(Format::Octal) => Ok(Yaml::Real(format!("{:#024o}", v))),
            _ => Err(error::format(format!(
                "Format {:?} not supported for type i64",
                self.format
            ))),
        }
    }

    fn serialize_i128(self, v: i128) -> Result<Yaml> {
        match self.format {
            None => Ok(Yaml::Real(v.to_string())),
            Some(Format::Binary) => Ok(Yaml::Real(format!("{:#0130}", v))),
            Some(Format::Decimal) => Ok(Yaml::Real(v.to_string())),
            Some(Format::Hex) => Ok(Yaml::Real(format!("{:#034x}", v))),
            Some(Format::Octal) => Ok(Yaml::Real(format!("{:#045o}", v))),
            _ => Err(error::format(format!(
                "Format {:?} not supported for type i128",
                self.format
            ))),
        }
    }

    fn serialize_u8(self, v: u8) -> Result<Yaml> {
        match self.format {
            None => Ok(Yaml::Integer(v as i64)),
            Some(Format::Binary) => Ok(Yaml::Real(format!("{:#010b}", v))),
            Some(Format::Decimal) => Ok(Yaml::Integer(v as i64)),
            Some(Format::Hex) => Ok(Yaml::Real(format!("{:#04x}", v))),
            Some(Format::Octal) => Ok(Yaml::Real(format!("{:#05o}", v))),
            _ => Err(error::format(format!(
                "Format {:?} not supported for type u8",
                self.format
            ))),
        }
    }

    fn serialize_u16(self, v: u16) -> Result<Yaml> {
        match self.format {
            None => Ok(Yaml::Integer(v as i64)),
            Some(Format::Binary) => Ok(Yaml::Real(format!("{:#018b}", v))),
            Some(Format::Decimal) => Ok(Yaml::Integer(v as i64)),
            Some(Format::Hex) => Ok(Yaml::Real(format!("{:#06x}", v))),
            Some(Format::Octal) => Ok(Yaml::Real(format!("{:#08o}", v))),
            _ => Err(error::format(format!(
                "Format {:?} not supported for type u16",
                self.format
            ))),
        }
    }

    fn serialize_u32(self, v: u32) -> Result<Yaml> {
        match self.format {
            None => Ok(Yaml::Integer(v as i64)),
            Some(Format::Binary) => Ok(Yaml::Real(format!("{:#034b}", v))),
            Some(Format::Decimal) => Ok(Yaml::Integer(v as i64)),
            Some(Format::Hex) => Ok(Yaml::Real(format!("{:#010x}", v))),
            Some(Format::Octal) => Ok(Yaml::Real(format!("{:#013o}", v))),
            _ => Err(error::format(format!(
                "Format {:?} not supported for type u32",
                self.format
            ))),
        }
    }

    fn serialize_u64(self, v: u64) -> Result<Yaml> {
        match self.format {
            None => Ok(Yaml::Real(v.to_string())),
            Some(Format::Binary) => Ok(Yaml::Real(format!("{:#066b}", v))),
            Some(Format::Decimal) => Ok(Yaml::Real(v.to_string())),
            Some(Format::Hex) => Ok(Yaml::Real(format!("{:#018x}", v))),
            Some(Format::Octal) => Ok(Yaml::Real(format!("{:#024o}", v))),
            _ => Err(error::format(format!(
                "Format {:?} not supported for type u64",
                self.format
            ))),
        }
    }

    fn serialize_u128(self, v: u128) -> Result<Yaml> {
        match self.format {
            None => Ok(Yaml::Real(v.to_string())),
            Some(Format::Binary) => Ok(Yaml::Real(format!("{:#0130}", v))),
            Some(Format::Decimal) => Ok(Yaml::Real(v.to_string())),
            Some(Format::Hex) => Ok(Yaml::Real(format!("{:#034x}", v))),
            Some(Format::Octal) => Ok(Yaml::Real(format!("{:#045o}", v))),
            _ => Err(error::format(format!(
                "Format {:?} not supported for type u128",
                self.format
            ))),
        }
    }

    fn serialize_f32(self, v: f32) -> Result<Yaml> {
        Ok(Yaml::Real(match v.classify() {
            num::FpCategory::Infinite if v.is_sign_positive() => ".inf".into(),
            num::FpCategory::Infinite => "-.inf".into(),
            num::FpCategory::Nan => ".nan".into(),
            _ => ryu::Buffer::new().format_finite(v).into(),
        }))
    }

    fn serialize_f64(self, v: f64) -> Result<Yaml> {
        Ok(Yaml::Real(match v.classify() {
            num::FpCategory::Infinite if v.is_sign_positive() => ".inf".into(),
            num::FpCategory::Infinite => "-.inf".into(),
            num::FpCategory::Nan => ".nan".into(),
            _ => ryu::Buffer::new().format_finite(v).into(),
        }))
    }

    fn serialize_char(self, value: char) -> Result<Yaml> {
        Ok(Yaml::String(value.to_string()))
    }

    fn serialize_str(self, value: &str) -> Result<Yaml> {
        match self.format {
            None => Ok(Yaml::String(value.to_owned())),
            Some(Format::Block) => Ok(Yaml::BlockScalar(value.to_owned())),
            _ => Err(error::format(format!(
                "Format {:?} not supported for type str",
                self.format
            ))),
        }
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Yaml> {
        let vec = value.iter().map(|&b| Yaml::Integer(b as i64)).collect();
        Ok(Yaml::Array(vec))
    }

    fn serialize_unit(self) -> Result<Yaml> {
        Ok(Yaml::Null)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Yaml> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &str,
        _variant_index: u32,
        variant: &str,
    ) -> Result<Yaml> {
        Ok(Yaml::String(variant.to_owned()))
    }

    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, value: &T) -> Result<Yaml>
    where
        T: ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &str,
        _variant_index: u32,
        variant: &str,
        value: &T,
    ) -> Result<Yaml>
    where
        T: ser::Serialize,
    {
        let format = self
            .yamlformat
            .map(|y| y.format(Some(variant), &MemberId::Variant))
            .flatten();
        let comment = self
            .yamlformat
            .map(|y| y.comment(Some(variant), &MemberId::Variant))
            .flatten();
        Ok(singleton_hash(
            to_yaml(variant, None, None)?,
            to_yaml(value, None, None)?,
            format,
            comment,
        ))
    }

    fn serialize_none(self) -> Result<Yaml> {
        self.serialize_unit()
    }

    fn serialize_some<V: ?Sized>(self, value: &V) -> Result<Yaml>
    where
        V: ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<SerializeArray<'f>> {
        let array = match len {
            None => yaml::Array::new(),
            Some(len) => yaml::Array::with_capacity(len),
        };
        Ok(SerializeArray {
            serializer: self,
            array,
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<SerializeArray<'f>> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(self, _name: &'static str, len: usize) -> Result<SerializeArray<'f>> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _enum: &'static str,
        _idx: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<SerializeTupleVariant<'f>> {
        Ok(SerializeTupleVariant {
            serializer: self,
            name: variant,
            array: yaml::Array::with_capacity(len),
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<SerializeMap> {
        Ok(SerializeMap {
            hash: yaml::Hash::new(),
            next_key: None,
        })
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<SerializeStruct<'f>> {
        Ok(SerializeStruct {
            serializer: self,
            hash: yaml::Hash::new(),
        })
    }

    fn serialize_struct_variant(
        self,
        _enum: &'static str,
        _idx: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<SerializeStructVariant<'f>> {
        Ok(SerializeStructVariant {
            serializer: self,
            name: variant,
            hash: yaml::Hash::new(),
        })
    }
}

#[doc(hidden)]
pub struct SerializeArray<'f> {
    serializer: SerializerToYaml<'f>,
    array: yaml::Array,
}

#[doc(hidden)]
pub struct SerializeTupleVariant<'f> {
    serializer: SerializerToYaml<'f>,
    name: &'static str,
    array: yaml::Array,
}

#[doc(hidden)]
pub struct SerializeMap {
    hash: yaml::Hash,
    next_key: Option<yaml::Yaml>,
}

#[doc(hidden)]
pub struct SerializeStruct<'f> {
    serializer: SerializerToYaml<'f>,
    hash: yaml::Hash,
}

#[doc(hidden)]
pub struct SerializeStructVariant<'f> {
    serializer: SerializerToYaml<'f>,
    name: &'static str,
    hash: yaml::Hash,
}

impl<'f> ser::SerializeSeq for SerializeArray<'f> {
    type Ok = yaml::Yaml;
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, elem: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        self.array.push(to_yaml(elem, None, None)?);
        Ok(())
    }

    fn end(self) -> Result<Yaml> {
        Ok(Yaml::Array(self.array))
    }
}

impl<'f> ser::SerializeTuple for SerializeArray<'f> {
    type Ok = yaml::Yaml;
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, elem: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        ser::SerializeSeq::serialize_element(self, elem)
    }

    fn end(self) -> Result<Yaml> {
        ser::SerializeSeq::end(self)
    }
}

impl<'f> ser::SerializeTupleStruct for SerializeArray<'f> {
    type Ok = yaml::Yaml;
    type Error = Error;

    fn serialize_field<V: ?Sized>(&mut self, value: &V) -> Result<()>
    where
        V: ser::Serialize,
    {
        let field = MemberId::Index(self.array.len() as u32);
        let format = self.serializer.format.take().or_else(|| {
            self.serializer
                .yamlformat
                .map(|y| y.format(None, &field))
                .flatten()
        });
        let comment = self.serializer.comment.take().or_else(|| {
            self.serializer
                .yamlformat
                .map(|y| y.comment(None, &field))
                .flatten()
        });
        self.array.push(to_yaml(value, format, comment)?);
        Ok(())
    }

    fn end(self) -> Result<Yaml> {
        ser::SerializeSeq::end(self)
    }
}

impl<'f> ser::SerializeTupleVariant for SerializeTupleVariant<'f> {
    type Ok = yaml::Yaml;
    type Error = Error;

    fn serialize_field<V: ?Sized>(&mut self, v: &V) -> Result<()>
    where
        V: ser::Serialize,
    {
        let field = MemberId::Index(self.array.len() as u32);
        let format = self.serializer.format.take().or_else(|| {
            self.serializer
                .yamlformat
                .map(|y| y.format(Some(self.name), &field))
                .flatten()
        });
        let comment = self.serializer.comment.take().or_else(|| {
            self.serializer
                .yamlformat
                .map(|y| y.comment(Some(self.name), &field))
                .flatten()
        });
        self.array.push(to_yaml(v, format, comment)?);
        Ok(())
    }

    fn end(mut self) -> Result<Yaml> {
        let format = self.serializer.format.take().or_else(|| {
            self.serializer
                .yamlformat
                .map(|y| y.format(Some(self.name), &MemberId::Variant))
                .flatten()
        });
        let comment = self.serializer.comment.take().or_else(|| {
            self.serializer
                .yamlformat
                .map(|y| y.comment(Some(self.name), &MemberId::Variant))
                .flatten()
        });
        Ok(singleton_hash(
            to_yaml(self.name, None, None)?,
            Yaml::Array(self.array),
            format,
            comment,
        ))
    }
}

impl ser::SerializeMap for SerializeMap {
    type Ok = yaml::Yaml;
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        self.next_key = Some(to_yaml(key, None, None)?);
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        match self.next_key.take() {
            Some(key) => self.hash.insert(key, to_yaml(value, None, None)?),
            None => panic!("serialize_value called before serialize_key"),
        };
        Ok(())
    }

    fn serialize_entry<K: ?Sized, V: ?Sized>(&mut self, key: &K, value: &V) -> Result<()>
    where
        K: ser::Serialize,
        V: ser::Serialize,
    {
        self.hash
            .insert(to_yaml(key, None, None)?, to_yaml(value, None, None)?);
        Ok(())
    }

    fn end(self) -> Result<Yaml> {
        Ok(Yaml::Hash(self.hash))
    }
}

impl<'f> ser::SerializeStruct for SerializeStruct<'f> {
    type Ok = yaml::Yaml;
    type Error = Error;

    fn serialize_field<V: ?Sized>(&mut self, key: &'static str, value: &V) -> Result<()>
    where
        V: ser::Serialize,
    {
        let field = MemberId::Name(key);
        let format = self.serializer.format.take().or_else(|| {
            self.serializer
                .yamlformat
                .map(|y| y.format(None, &field))
                .flatten()
        });
        let comment = self.serializer.comment.take().or_else(|| {
            self.serializer
                .yamlformat
                .map(|y| y.comment(None, &field))
                .flatten()
        });

        self.hash
            .insert(to_yaml(key, None, comment)?, to_yaml(value, format, None)?);
        Ok(())
    }

    fn end(self) -> Result<Yaml> {
        Ok(Yaml::Hash(self.hash))
    }
}

impl<'f> ser::SerializeStructVariant for SerializeStructVariant<'f> {
    type Ok = yaml::Yaml;
    type Error = Error;

    fn serialize_field<V: ?Sized>(&mut self, field: &'static str, v: &V) -> Result<()>
    where
        V: ser::Serialize,
    {
        let fld = MemberId::Name(field);
        let format = self.serializer.format.take().or_else(|| {
            self.serializer
                .yamlformat
                .map(|y| y.format(Some(self.name), &fld))
                .flatten()
        });
        let comment = self.serializer.comment.take().or_else(|| {
            self.serializer
                .yamlformat
                .map(|y| y.comment(Some(self.name), &fld))
                .flatten()
        });
        self.hash
            .insert(to_yaml(field, None, comment)?, to_yaml(v, format, None)?);
        Ok(())
    }

    fn end(mut self) -> Result<Yaml> {
        let format = self.serializer.format.take().or_else(|| {
            self.serializer
                .yamlformat
                .map(|y| y.format(Some(self.name), &MemberId::Variant))
                .flatten()
        });
        let comment = self.serializer.comment.take().or_else(|| {
            self.serializer
                .yamlformat
                .map(|y| y.comment(Some(self.name), &MemberId::Variant))
                .flatten()
        });
        Ok(singleton_hash(
            to_yaml(self.name, None, None)?,
            Yaml::Hash(self.hash),
            format,
            comment,
        ))
    }
}

/// Serialize the given data structure as YAML into the IO stream.
///
/// Serialization can fail if `T`'s implementation of `Serialize` decides to
/// return an error.
pub fn to_writer<W, T: ?Sized>(writer: W, value: &T) -> Result<()>
where
    W: io::Write,
    T: ser::Serialize,
{
    let yf = YamlFormatType::get(value);
    value.serialize(&mut Serializer::new_with_format(writer, yf))
}

/// Serialize the given data structure as a YAML byte vector.
///
/// Serialization can fail if `T`'s implementation of `Serialize` decides to
/// return an error.
pub fn to_vec<T: ?Sized>(value: &T) -> Result<Vec<u8>>
where
    T: ser::Serialize,
{
    let mut vec = Vec::with_capacity(128);
    to_writer(&mut vec, value)?;
    Ok(vec)
}

/// Serialize the given data structure as a String of YAML.
///
/// Serialization can fail if `T`'s implementation of `Serialize` decides to
/// return an error.
pub fn to_string<T: ?Sized>(value: &T) -> Result<String>
where
    T: ser::Serialize,
{
    String::from_utf8(to_vec(value)?).map_err(error::string_utf8)
}

/// The yaml-rust library uses `fmt::Write` intead of `io::Write` so this is a
/// simple adapter.
struct FmtToIoWriter<W> {
    writer: W,
}

impl<W> fmt::Write for FmtToIoWriter<W>
where
    W: io::Write,
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if self.writer.write_all(s.as_bytes()).is_err() {
            return Err(fmt::Error);
        }
        Ok(())
    }
}

fn to_yaml<T>(elem: &T, format: Option<Format>, comment: Option<String>) -> Result<Yaml>
where
    T: ser::Serialize + ?Sized,
{
    let yf = YamlFormatType::get(elem);
    let yaml = elem.serialize(SerializerToYaml::new(yf, format))?;
    if let Some(c) = comment {
        Ok(Yaml::DocFragment(
            vec![Yaml::Comment(c), yaml],
            yaml::FragStyle::None,
        ))
    } else {
        Ok(yaml)
    }
}

fn singleton_hash(k: Yaml, v: Yaml, format: Option<Format>, comment: Option<String>) -> Yaml {
    let mut hash = yaml::Hash::new();
    hash.insert(k, v);
    let yaml = Yaml::Hash(hash);

    match (format, comment) {
        (None, Some(c)) => Yaml::DocFragment(vec![Yaml::Comment(c), yaml], yaml::FragStyle::None),
        (Some(f), None) if f == Format::Oneline => {
            Yaml::DocFragment(vec![yaml], yaml::FragStyle::Oneline)
        }
        (Some(f), Some(c)) if f == Format::Oneline => Yaml::DocFragment(
            vec![
                Yaml::Comment(c),
                Yaml::DocFragment(vec![yaml], yaml::FragStyle::Oneline),
            ],
            yaml::FragStyle::Indented,
        ),
        (_, _) => yaml,
    }
}
