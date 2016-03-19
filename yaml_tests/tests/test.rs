// Copyright 2016 Serde YAML Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![cfg_attr(not(feature = "with-syntex"), feature(custom_derive, plugin))]
#![cfg_attr(not(feature = "with-syntex"), plugin(serde_macros, indoc))]

extern crate serde;
extern crate serde_yaml;

#[cfg(feature = "with-syntex")]
include!(concat!(env!("OUT_DIR"), "/test.rs"));

#[cfg(not(feature = "with-syntex"))]
include!("test.rs.in");
