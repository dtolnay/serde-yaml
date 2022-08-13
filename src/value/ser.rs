use crate::error::Error;
use crate::value::{to_value, Mapping, Number, Sequence, Tag, TaggedValue, Value};
use serde::ser::{self, Serialize};

type Result<T, E = Error> = std::result::Result<T, E>;

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Value::Null => serializer.serialize_unit(),
            Value::Bool(b) => serializer.serialize_bool(*b),
            Value::Number(n) => n.serialize(serializer),
            Value::String(s) => serializer.serialize_str(s),
            Value::Sequence(seq) => seq.serialize(serializer),
            Value::Mapping(mapping) => {
                use serde::ser::SerializeMap;
                let mut map = serializer.serialize_map(Some(mapping.len()))?;
                for (k, v) in mapping {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
            Value::Tagged(tagged) => tagged.serialize(serializer),
        }
    }
}

/// Serializer whose output is a `Value`.
///
/// This is the serializer that backs [`serde_yaml::to_value`][crate::to_value].
/// Unlike the main serde_yaml serializer which goes from some serializable
/// value of type `T` to YAML text, this one goes from `T` to
/// `serde_yaml::Value`.
///
/// The `to_value` function is implementable as:
///
/// ```
/// use serde::Serialize;
/// use serde_yaml::{Error, Value};
///
/// pub fn to_value<T>(input: T) -> Result<Value, Error>
/// where
///     T: Serialize,
/// {
///     input.serialize(serde_yaml::value::Serializer)
/// }
/// ```
pub struct Serializer;

impl ser::Serializer for Serializer {
    type Ok = Value;
    type Error = Error;

    type SerializeSeq = SerializeArray;
    type SerializeTuple = SerializeArray;
    type SerializeTupleStruct = SerializeArray;
    type SerializeTupleVariant = SerializeTupleVariant;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeStruct;
    type SerializeStructVariant = SerializeStructVariant;

    fn serialize_bool(self, v: bool) -> Result<Value> {
        Ok(Value::Bool(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Value> {
        Ok(Value::Number(Number::from(v)))
    }

    fn serialize_i16(self, v: i16) -> Result<Value> {
        Ok(Value::Number(Number::from(v)))
    }

    fn serialize_i32(self, v: i32) -> Result<Value> {
        Ok(Value::Number(Number::from(v)))
    }

    fn serialize_i64(self, v: i64) -> Result<Value> {
        Ok(Value::Number(Number::from(v)))
    }

    fn serialize_i128(self, v: i128) -> Result<Value> {
        if let Ok(v) = u64::try_from(v) {
            self.serialize_u64(v)
        } else if let Ok(v) = i64::try_from(v) {
            self.serialize_i64(v)
        } else {
            Ok(Value::String(v.to_string()))
        }
    }

    fn serialize_u8(self, v: u8) -> Result<Value> {
        Ok(Value::Number(Number::from(v)))
    }

    fn serialize_u16(self, v: u16) -> Result<Value> {
        Ok(Value::Number(Number::from(v)))
    }

    fn serialize_u32(self, v: u32) -> Result<Value> {
        Ok(Value::Number(Number::from(v)))
    }

    fn serialize_u64(self, v: u64) -> Result<Value> {
        Ok(Value::Number(Number::from(v)))
    }

    fn serialize_u128(self, v: u128) -> Result<Value> {
        if let Ok(v) = u64::try_from(v) {
            self.serialize_u64(v)
        } else {
            Ok(Value::String(v.to_string()))
        }
    }

    fn serialize_f32(self, v: f32) -> Result<Value> {
        Ok(Value::Number(Number::from(v)))
    }

    fn serialize_f64(self, v: f64) -> Result<Value> {
        Ok(Value::Number(Number::from(v)))
    }

    fn serialize_char(self, value: char) -> Result<Value> {
        Ok(Value::String(value.to_string()))
    }

    fn serialize_str(self, value: &str) -> Result<Value> {
        Ok(Value::String(value.to_owned()))
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Value> {
        let vec = value
            .iter()
            .map(|&b| Value::Number(Number::from(b)))
            .collect();
        Ok(Value::Sequence(vec))
    }

    fn serialize_unit(self) -> Result<Value> {
        Ok(Value::Null)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Value> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &str,
        _variant_index: u32,
        variant: &str,
    ) -> Result<Value> {
        Ok(Value::String(variant.to_owned()))
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Value>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &str,
        _variant_index: u32,
        variant: &str,
        value: &T,
    ) -> Result<Value>
    where
        T: ?Sized + ser::Serialize,
    {
        Ok(Value::Tagged(Box::new(TaggedValue {
            tag: Tag::new(variant),
            value: to_value(value)?,
        })))
    }

    fn serialize_none(self) -> Result<Value> {
        self.serialize_unit()
    }

    fn serialize_some<V>(self, value: &V) -> Result<Value>
    where
        V: ?Sized + ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<SerializeArray> {
        let sequence = match len {
            None => Sequence::new(),
            Some(len) => Sequence::with_capacity(len),
        };
        Ok(SerializeArray { sequence })
    }

    fn serialize_tuple(self, len: usize) -> Result<SerializeArray> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(self, _name: &'static str, len: usize) -> Result<SerializeArray> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _enum: &'static str,
        _idx: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<SerializeTupleVariant> {
        Ok(SerializeTupleVariant {
            tag: variant,
            sequence: Sequence::with_capacity(len),
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<SerializeMap> {
        Ok(SerializeMap {
            mapping: Mapping::new(),
            next_key: None,
        })
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<SerializeStruct> {
        Ok(SerializeStruct {
            mapping: Mapping::new(),
        })
    }

    fn serialize_struct_variant(
        self,
        _enum: &'static str,
        _idx: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<SerializeStructVariant> {
        Ok(SerializeStructVariant {
            tag: variant,
            mapping: Mapping::new(),
        })
    }
}

pub struct SerializeArray {
    sequence: Sequence,
}

impl ser::SerializeSeq for SerializeArray {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T>(&mut self, elem: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        self.sequence.push(to_value(elem)?);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(Value::Sequence(self.sequence))
    }
}

impl ser::SerializeTuple for SerializeArray {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T>(&mut self, elem: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        ser::SerializeSeq::serialize_element(self, elem)
    }

    fn end(self) -> Result<Value> {
        ser::SerializeSeq::end(self)
    }
}

impl ser::SerializeTupleStruct for SerializeArray {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<V>(&mut self, value: &V) -> Result<()>
    where
        V: ?Sized + ser::Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Value> {
        ser::SerializeSeq::end(self)
    }
}

pub struct SerializeTupleVariant {
    tag: &'static str,
    sequence: Sequence,
}

impl ser::SerializeTupleVariant for SerializeTupleVariant {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<V>(&mut self, v: &V) -> Result<()>
    where
        V: ?Sized + ser::Serialize,
    {
        self.sequence.push(to_value(v)?);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(Value::Tagged(Box::new(TaggedValue {
            tag: Tag::new(self.tag),
            value: Value::Sequence(self.sequence),
        })))
    }
}

pub struct SerializeMap {
    mapping: Mapping,
    next_key: Option<Value>,
}

impl ser::SerializeMap for SerializeMap {
    type Ok = Value;
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        self.next_key = Some(to_value(key)?);
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        match self.next_key.take() {
            Some(key) => self.mapping.insert(key, to_value(value)?),
            None => panic!("serialize_value called before serialize_key"),
        };
        Ok(())
    }

    fn serialize_entry<K, V>(&mut self, key: &K, value: &V) -> Result<()>
    where
        K: ?Sized + ser::Serialize,
        V: ?Sized + ser::Serialize,
    {
        self.mapping.insert(to_value(key)?, to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(Value::Mapping(self.mapping))
    }
}

pub struct SerializeStruct {
    mapping: Mapping,
}

impl ser::SerializeStruct for SerializeStruct {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<V>(&mut self, key: &'static str, value: &V) -> Result<()>
    where
        V: ?Sized + ser::Serialize,
    {
        self.mapping.insert(to_value(key)?, to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(Value::Mapping(self.mapping))
    }
}

pub struct SerializeStructVariant {
    tag: &'static str,
    mapping: Mapping,
}

impl ser::SerializeStructVariant for SerializeStructVariant {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<V>(&mut self, field: &'static str, v: &V) -> Result<()>
    where
        V: ?Sized + ser::Serialize,
    {
        self.mapping.insert(to_value(field)?, to_value(v)?);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(Value::Tagged(Box::new(TaggedValue {
            tag: Tag::new(self.tag),
            value: Value::Mapping(self.mapping),
        })))
    }
}
