// Copyright 2016 Serde YAML Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This crate is a Rust library for using the [Serde] serialization framework
//! with data in [YAML] file format.
//!
//! This library does not reimplement a YAML parser; it uses [yaml-rust] which
//! is a pure Rust YAML 1.2 implementation.
//!
//! [Serde]: https://github.com/serde-rs/serde
//! [YAML]: http://yaml.org
//! [yaml-rust]: https://github.com/chyh1990/yaml-rust
//!
//! # Examples
//!
//! ```rust
//! extern crate serde_yaml;
//!
//! use std::collections::BTreeMap;
//!
//! // You have some type.
//! let mut map = BTreeMap::new();
//! map.insert("x".to_string(), 1.0);
//! map.insert("y".to_string(), 2.0);
//!
//! // Serialize it to a YAML string.
//! let s = serde_yaml::to_string(&map).unwrap();
//! assert_eq!(s, "---\nx: 1\ny: 2");
//!
//! // Deserialize it back to a Rust type.
//! let deserialized_map: BTreeMap<String, f64> = serde_yaml::from_str(&s).unwrap();
//! assert_eq!(map, deserialized_map);
//! ```
//!
//! ## Using serde derive
//!
//! It can also be used with Serde's serialization code generator `serde_derive` to
//! handle structs and enums defined in your own program.
//!
//! ```rust
//! #[macro_use] extern crate serde_derive;
//! extern crate serde_yaml;
//!
//! #[derive(Debug, PartialEq, Serialize, Deserialize)]
//! struct Point { x: f64, y: f64 }
//!
//! # fn main() {
//! let point = Point { x: 1.0, y: 2.0 };
//!
//! let s = serde_yaml::to_string(&point).unwrap();
//! assert_eq!(s, "---\nx: 1\ny: 2");
//!
//! let deserialized_point: Point = serde_yaml::from_str(&s).unwrap();
//! assert_eq!(point, deserialized_point);
//! # }
//! ```

#![cfg_attr(feature = "cargo-clippy", deny(clippy, clippy_pedantic))]
// Whitelisted clippy_pedantic lints
#![cfg_attr(feature = "cargo-clippy", allow(
// private Deserializer::next
    should_implement_trait,
// things are often more readable this way
    single_match_else,
    stutter,
// not practical
    missing_docs_in_private_items,
// not stable
    empty_enum,
))]

extern crate linked_hash_map;
extern crate num_traits;
#[macro_use]
extern crate serde;
extern crate yaml_rust;

pub use self::de::{from_reader, from_slice, from_str};
pub use self::ser::{to_string, to_vec, to_writer};
pub use self::value::{Sequence, Value, from_value, to_value, Number};
pub use self::error::{Error, Result};
pub use self::mapping::Mapping;

mod de;
mod ser;
mod value;
mod error;
mod path;
mod mapping;
