// Copyright 2016 Serde YAML Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![cfg_attr(feature="clippy", deny(clippy))] // turn warnings into errors

extern crate dtoa;
extern crate linked_hash_map;
#[macro_use]
extern crate serde;
extern crate yaml_rust;

pub use self::de::{from_iter, from_reader, from_slice, from_str};
pub use self::ser::{to_string, to_vec, to_writer};
pub use self::value::{Mapping, Sequence, Value, from_value, to_value};
pub use self::error::{Error, Result};

mod de;
mod ser;
mod value;
mod error;
