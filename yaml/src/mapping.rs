// Copyright 2016 Serde YAML Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt;
use std::iter::FromIterator;
use std::ops::{Index, IndexMut};

use linked_hash_map::{self, LinkedHashMap};
use serde::{self, Serialize, Deserialize, Deserializer};

use value::Value;

/// A YAML mapping in which the keys and values are both `serde_yaml::Value`.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, PartialOrd)]
pub struct Mapping {
    map: LinkedHashMap<Value, Value>,
}

impl Mapping {
    #[inline]
    pub fn new() -> Self { Self::default() }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Mapping {
            map: LinkedHashMap::with_capacity(capacity),
        }
    }

    #[inline]
    pub fn reserve(&mut self, additional: usize) { self.map.reserve(additional) }

    #[inline]
    pub fn shrink_to_fit(&mut self) { self.map.shrink_to_fit() }

    #[inline]
    pub fn insert(&mut self, k: Value, v: Value) -> Option<Value> { self.map.insert(k, v) }

    #[inline]
    pub fn contains_key(&self, k: &Value) -> bool { self.map.contains_key(k) }

    #[inline]
    pub fn get(&self, k: &Value) -> Option<&Value> { self.map.get(k) }

    #[inline]
    pub fn get_mut(&mut self, k: &Value) -> Option<&mut Value> { self.map.get_mut(k) }

    #[inline]
    pub fn remove(&mut self, k: &Value) -> Option<Value> { self.map.remove(k) }

    #[inline]
    pub fn capacity(&self) -> usize { self.map.capacity() }

    #[inline]
    pub fn len(&self) -> usize { self.map.len() }

    #[inline]
    pub fn is_empty(&self) -> bool { self.map.is_empty() }

    #[inline]
    pub fn clear(&mut self) { self.map.clear() }

    #[inline]
    pub fn iter(&self) -> linked_hash_map::Iter<Value, Value> { self.map.iter() }

    #[inline]
    pub fn iter_mut(&mut self) -> linked_hash_map::IterMut<Value, Value> { self.map.iter_mut() }
}

impl<'a> Index<&'a Value> for Mapping {
    type Output = Value;
    #[inline]
    fn index(&self, index: &'a Value) -> &Value {
        self.map.index(index)
    }
}

impl<'a> IndexMut<&'a Value> for Mapping {
    #[inline]
    fn index_mut(&mut self, index: &'a Value) -> &mut Value {
        self.map.index_mut(index)
    }
}

impl Extend<(Value, Value)> for Mapping {
    #[inline]
    fn extend<I: IntoIterator<Item=(Value, Value)>>(&mut self, iter: I) {
        self.map.extend(iter);
    }
}

impl FromIterator<(Value, Value)> for Mapping {
    #[inline]
    fn from_iter<I: IntoIterator<Item=(Value, Value)>>(iter: I) -> Self {
        Mapping {
            map: LinkedHashMap::from_iter(iter)
        }
    }
}

impl<'a> IntoIterator for &'a Mapping {
    type Item = (&'a Value, &'a Value);
    type IntoIter = linked_hash_map::Iter<'a, Value, Value>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.map.iter() }
}

impl<'a> IntoIterator for &'a mut Mapping {
    type Item = (&'a Value, &'a mut Value);
    type IntoIter = linked_hash_map::IterMut<'a, Value, Value>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.map.iter_mut() }
}

impl IntoIterator for Mapping {
    type Item = (Value, Value);
    type IntoIter = linked_hash_map::IntoIter<Value, Value>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.map.into_iter() }
}

impl Serialize for Mapping {
    #[inline]
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map_serializer = try!(serializer.serialize_map(Some(self.len())));
        for (k, v) in self {
            try!(map_serializer.serialize_key(k));
            try!(map_serializer.serialize_value(v));
        }
        map_serializer.end()
    }
}

impl Deserialize for Mapping {
    fn deserialize<D: Deserializer>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;

        impl serde::de::Visitor for Visitor {
            type Value = Mapping;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a YAML mapping")
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Self::Value, E>
                where E: serde::de::Error
            {
                Ok(Mapping::new())
            }

            #[inline]
            fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
                where V: serde::de::MapVisitor
            {
                let mut values = Mapping::with_capacity(visitor.size_hint().0);
                while let Some((k, v)) = try!(visitor.visit()) {
                    values.insert(k, v);
                }
                Ok(values)
            }
        }

        deserializer.deserialize_map(Visitor)
    }
}
