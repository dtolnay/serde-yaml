use super::Value;
use mapping::Mapping;
use value::number::Number;

// Implement a bunch of conversion to make it easier to create YAML values
// on the fly.

macro_rules! from_integer {
    ($($ty:ident)*) => {
        $(
            impl From<$ty> for Value {
                fn from(n: $ty) -> Self {
                    Value::Number(Number::from(n))
                }
            }
        )*
    };
}

from_integer! {
    i8 i16 i32 i64 isize
    u8 u16 u32 u64 usize
}

impl From<f32> for Value {
    /// Convert 32-bit floating point number to `Value`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate serde_yaml;
    /// #
    /// # fn main() {
    /// use serde_yaml::Value;
    ///
    /// let f: f32 = 13.37;
    /// let x: Value = f.into();
    /// # }
    /// ```
    fn from(f: f32) -> Self {
        From::from(f as f64)
    }
}

impl From<f64> for Value {
    /// Convert 64-bit floating point number to `Value`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate serde_yaml;
    /// #
    /// # fn main() {
    /// use serde_yaml::Value;
    ///
    /// let f: f64 = 13.37;
    /// let x: Value = f.into();
    /// # }
    /// ```
    fn from(f: f64) -> Self {
        Value::Number(Number::from_f64(f))
    }
}

impl From<bool> for Value {
    /// Convert boolean to `Value`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate serde_yaml;
    /// #
    /// # fn main() {
    /// use serde_yaml::Value;
    ///
    /// let b = false;
    /// let x: Value = b.into();
    /// # }
    /// ```
    fn from(f: bool) -> Self {
        Value::Bool(f)
    }
}

impl From<String> for Value {
    /// Convert `String` to `Value`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate serde_yaml;
    /// #
    /// # fn main() {
    /// use serde_yaml::Value;
    ///
    /// let s: String = "lorem".to_string();
    /// let x: Value = s.into();
    /// # }
    /// ```
    fn from(f: String) -> Self {
        Value::String(f)
    }
}

impl<'a> From<&'a str> for Value {
    /// Convert string slice to `Value`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate serde_yaml;
    /// #
    /// # fn main() {
    /// use serde_yaml::Value;
    ///
    /// let s: &str = "lorem";
    /// let x: Value = s.into();
    /// # }
    /// ```
    fn from(f: &str) -> Self {
        Value::String(f.to_string())
    }
}

use std::borrow::Cow;

impl<'a> From<Cow<'a, str>> for Value {
    /// Convert copy-on-write string to `Value`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate serde_yaml;
    /// #
    /// # fn main() {
    /// use serde_yaml::Value;
    /// use std::borrow::Cow;
    ///
    /// let s: Cow<str> = Cow::Borrowed("lorem");
    /// let x: Value = s.into();
    /// # }
    /// ```
    ///
    /// ```rust
    /// # extern crate serde_yaml;
    /// #
    /// # fn main() {
    /// use serde_yaml::Value;
    /// use std::borrow::Cow;
    ///
    /// let s: Cow<str> = Cow::Owned("lorem".to_string());
    /// let x: Value = s.into();
    /// # }
    /// ```
    fn from(f: Cow<'a, str>) -> Self {
        Value::String(f.to_string())
    }
}

impl From<Mapping> for Value {
    /// Convert map (with string keys) to `Value`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate serde_yaml;
    /// #
    /// # fn main() {
    /// use serde_yaml::{Mapping, Value};
    ///
    /// let mut m = Mapping::new();
    /// m.insert("Lorem".into(), "ipsum".into());
    /// let x: Value = m.into();
    /// # }
    /// ```
    fn from(f: Mapping) -> Self {
        Value::Mapping(f)
    }
}

impl<T: Into<Value>> From<Vec<T>> for Value {
    /// Convert a `Vec` to `Value`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate serde_yaml;
    /// #
    /// # fn main() {
    /// use serde_yaml::Value;
    ///
    /// let v = vec!["lorem", "ipsum", "dolor"];
    /// let x: Value = v.into();
    /// # }
    /// ```
    fn from(f: Vec<T>) -> Self {
        Value::Sequence(f.into_iter().map(Into::into).collect())
    }
}

impl<'a, T: Clone + Into<Value>> From<&'a [T]> for Value {
    /// Convert a slice to `Value`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate serde_yaml;
    /// #
    /// # fn main() {
    /// use serde_yaml::Value;
    ///
    /// let v: &[&str] = &["lorem", "ipsum", "dolor"];
    /// let x: Value = v.into();
    /// # }
    /// ```
    fn from(f: &'a [T]) -> Self {
        Value::Sequence(f.into_iter().cloned().map(Into::into).collect())
    }
}

use std::iter::FromIterator;

impl<T: Into<Value>> FromIterator<T> for Value {
    /// Convert an iteratable type to a YAML sequence
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate serde_yaml;
    /// #
    /// # fn main() {
    /// use serde_yaml::Value;
    ///
    /// let v = std::iter::repeat(42).take(5);
    /// let x: Value = v.collect();
    /// # }
    /// ```
    ///
    /// ```rust
    /// # extern crate serde_yaml;
    /// #
    /// # fn main() {
    /// use serde_yaml::Value;
    ///
    /// let v: Vec<_> = vec!["lorem", "ipsum", "dolor"];
    /// let x: Value = v.into_iter().collect();
    /// # }
    /// ```
    ///
    /// ```rust
    /// # extern crate serde_yaml;
    /// #
    /// # fn main() {
    /// use std::iter::FromIterator;
    /// use serde_yaml::Value;
    ///
    /// let x: Value = Value::from_iter(vec!["lorem", "ipsum", "dolor"]);
    /// # }
    /// ```
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let vec: Vec<Value> = iter.into_iter().map(|x| x.into()).collect();

        Value::Sequence(vec)
    }
}
