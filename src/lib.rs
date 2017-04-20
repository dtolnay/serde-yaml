// Copyright 2016 Serde YAML Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

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
pub use self::value::{Sequence, Value, from_value, to_value};
pub use self::error::{Error, Result};
pub use self::mapping::Mapping;

mod de;
mod ser;
mod value;
mod error;
mod path;
mod mapping;
