// Copyright 2016 Serde YAML Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.extern crate serde;

#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate serde;
extern crate serde_yaml;

use std::fmt::Debug;
use std::collections::BTreeMap;

fn test_serde<T>(thing: T, yaml: &str)
    where T: serde::Serialize + serde::Deserialize + PartialEq + Debug
{
    let serialized = serde_yaml::to_string(&thing).unwrap();
    assert_eq!(yaml, serialized);

    let deserialized: T = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(thing, deserialized);
}

#[test]
fn test_int() {
    let thing = 256;
    let yaml = "---\n\
                256";
    test_serde(thing, yaml);
}

#[test]
fn test_float() {
    let thing = 25.6;
    let yaml = "---\n\
                25.6";
    test_serde(thing, yaml);
}

#[test]
fn test_vec() {
    let thing = vec![1, 2, 3];
    let yaml = "---\n\
                - 1\n\
                - 2\n\
                - 3";
    test_serde(thing, yaml);
}

#[test]
fn test_map() {
    let mut thing = BTreeMap::new();
    thing.insert(String::from("x"), 1);
    thing.insert(String::from("y"), 2);
    let yaml = "---\n\
                \"x\": 1\n\
                \"y\": 2";
    test_serde(thing, yaml);
}

#[test]
fn test_basic_struct() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Basic {
        x: isize,
        y: String,
        z: bool,
    }
    let thing = Basic{x: -4, y: String::from("hi"), z: true};
    let yaml = "---\n\
                \"x\": -4\n\
                \"y\": \"hi\"\n\
                \"z\": true";
    test_serde(thing, yaml);
}

#[test]
fn test_nested_struct() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Outer {
        inner: Inner,
    }
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Inner {
        v: u16,
    }
    let thing = Outer{inner: Inner{v: 512}};
    let yaml = "---\n\
                \"inner\": \n  \
                  \"v\": 512";
    test_serde(thing, yaml);
}

#[test]
fn test_option() {
    let thing = vec![Some(1), None, Some(3)];
    let yaml = "---\n\
                - 1\n\
                - ~\n\
                - 3";
    test_serde(thing, yaml);
}

#[test]
fn test_unit() {
    let thing = vec![(), ()];
    let yaml = "---\n\
                - ~\n\
                - ~";
    test_serde(thing, yaml);
}

#[test]
fn test_unit_variant() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    enum Variant {
        First,
        Second,
    }
    let thing = Variant::First;
    let yaml = "---\n\
                \"First\": []";
    test_serde(thing, yaml);
}

#[test]
fn test_newtype_struct() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct OriginalType {
        v: u16,
    }
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct NewType(OriginalType);
    let thing = NewType(OriginalType{v: 1});
    let yaml = "---\n\
                \"v\": 1";
    test_serde(thing, yaml);
}

#[test]
fn test_newtype_variant() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    enum Variant {
        Size(usize),
    }
    let thing = Variant::Size(127);
    let yaml = "---\n\
                \"Size\": 127";
    test_serde(thing, yaml);
}

#[test]
fn test_tuple_variant() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    enum Variant {
        Rgb(u8, u8, u8),
    }
    let thing = Variant::Rgb(32, 64, 96);
    let yaml = "---\n\
                \"Rgb\": \n  \
                  - 32\n  \
                  - 64\n  \
                  - 96";
    test_serde(thing, yaml);
}

#[test]
fn test_struct_variant() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    enum Variant {
        Color{r: u8, g: u8, b: u8},
    }
    let thing = Variant::Color{r: 32, g: 64, b: 96};
    let yaml = "---\n\
                \"Color\": \n  \
                  \"b\": 96\n  \
                  \"g\": 64\n  \
                  \"r\": 32";
    test_serde(thing, yaml);
}
