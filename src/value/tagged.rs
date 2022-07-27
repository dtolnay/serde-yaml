use crate::value::de::{MapDeserializer, SeqDeserializer};
use crate::value::Value;
use crate::Error;
use serde::de::value::StrDeserializer;
use serde::de::{
    Deserialize, DeserializeSeed, Deserializer, EnumAccess, Error as _, VariantAccess, Visitor,
};
use serde::ser::{Serialize, SerializeMap, Serializer};
use std::cmp::Ordering;
use std::fmt::{self, Debug};
use std::hash::{Hash, Hasher};

/// FIXME
#[derive(Clone)]
pub struct Tag {
    pub(crate) string: String,
}

/// FIXME
#[derive(Clone, PartialEq, PartialOrd, Hash, Debug)]
pub struct TaggedValue {
    /// FIXME
    pub tag: Tag,
    /// FIXME
    pub value: Value,
}

impl Tag {
    /// FIXME
    pub fn new(string: impl Into<String>) -> Self {
        Tag {
            string: string.into(),
        }
    }
}

pub(crate) fn nobang(maybe_banged: &str) -> &str {
    maybe_banged.strip_prefix('!').unwrap_or(maybe_banged)
}

impl Eq for Tag {}

impl PartialEq for Tag {
    fn eq(&self, other: &Tag) -> bool {
        PartialEq::eq(nobang(&self.string), nobang(&other.string))
    }
}

impl<T> PartialEq<T> for Tag
where
    T: ?Sized + AsRef<str>,
{
    fn eq(&self, other: &T) -> bool {
        PartialEq::eq(nobang(&self.string), nobang(other.as_ref()))
    }
}

impl Ord for Tag {
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(nobang(&self.string), nobang(&other.string))
    }
}

impl PartialOrd for Tag {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        PartialOrd::partial_cmp(nobang(&self.string), nobang(&other.string))
    }
}

impl Hash for Tag {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        nobang(&self.string).hash(hasher);
    }
}

impl Debug for Tag {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "!{}", nobang(&self.string))
    }
}

impl Serialize for TaggedValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry(&format_args!("!{}", nobang(&self.tag.string)), &self.value)?;
        map.end()
    }
}

impl<'de> EnumAccess<'de> for TaggedValue {
    type Error = Error;
    type Variant = Value;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Error>
    where
        V: DeserializeSeed<'de>,
    {
        let tag = StrDeserializer::<Error>::new(nobang(&self.tag.string));
        let value = seed.deserialize(tag)?;
        Ok((value, self.value))
    }
}

impl<'de> VariantAccess<'de> for Value {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Error> {
        Deserialize::deserialize(self)
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Error>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        if let Value::Sequence(v) = self {
            Deserializer::deserialize_any(SeqDeserializer::new(v), visitor)
        } else {
            Err(Error::invalid_type(self.unexpected(), &"tuple variant"))
        }
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        if let Value::Mapping(v) = self {
            Deserializer::deserialize_any(MapDeserializer::new(v), visitor)
        } else {
            Err(Error::invalid_type(self.unexpected(), &"struct variant"))
        }
    }
}
