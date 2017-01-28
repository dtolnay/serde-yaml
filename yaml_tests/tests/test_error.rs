// Copyright 2016 Serde YAML Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use serde;
use serde_yaml;

use std::fmt::Debug;

fn test_error<T>(yaml: &str, expected: &str)
    where T: serde::Deserialize + Debug,
{
    let result = serde_yaml::from_str::<T>(yaml);
    assert_eq!(expected, format!("{}", result.unwrap_err()));
}

#[test]
fn test_incorrect_type() {
    let yaml = indoc!("
        ---
        str");
    let expected = "invalid type: string \"str\", expected i16 at line 2 column 1";
    test_error::<i16>(yaml, expected);
}

#[test]
fn test_empty() {
    let yaml = "";
    let expected = "EOF while parsing a value";
    test_error::<String>(yaml, expected);
}

#[test]
fn test_missing_field() {
    #[derive(Deserialize, Debug)]
    struct Basic {
        v: bool,
        w: bool,
    }
    let yaml = indoc!("
        ---
        v: true");
    let expected = "missing field `w` at line 2 column 2";
    test_error::<Basic>(yaml, expected);
}

#[test]
fn test_unknown_anchor() {
    let yaml = indoc!("
        ---
        *some");
    let expected = "while parsing node, found unknown anchor at line 2 column 1";
    test_error::<String>(yaml, expected);
}

#[test]
fn test_two_documents() {
    let yaml = indoc!("
        ---
        0
        ---
        1");
    let expected = "deserializing from YAML containing more than one document is not supported";
    test_error::<usize>(yaml, expected);
}

#[test]
fn test_variant_map_wrong_size() {
    #[derive(Deserialize, Debug)]
    enum Variant {
        V(usize),
    }
    let yaml = indoc!(r#"
        ---
        "V": 16
        "other": 32"#);
    let expected = "invalid length 2, expected map containing 1 entry";
    test_error::<Variant>(yaml, expected);
}

#[test]
fn test_variant_not_a_map() {
    #[derive(Deserialize, Debug)]
    enum Variant {
        V(usize),
    }
    let yaml = indoc!(r#"
        ---
        - "V""#);
    let expected = "invalid type: sequence, expected string or singleton map at line 2 column 1";
    test_error::<Variant>(yaml, expected);
}

#[test]
fn test_bad_bool() {
    let yaml = indoc!("
        ---
        !!bool str");
    let expected = "invalid value: string \"str\", expected a boolean at line 2 column 8";
    test_error::<bool>(yaml, expected);
}

#[test]
fn test_bad_int() {
    let yaml = indoc!("
        ---
        !!int str");
    let expected = "invalid value: string \"str\", expected an integer at line 2 column 7";
    test_error::<i64>(yaml, expected);
}

#[test]
fn test_bad_float() {
    let yaml = indoc!("
        ---
        !!float str");
    let expected = "invalid value: string \"str\", expected a float at line 2 column 9";
    test_error::<f64>(yaml, expected);
}

#[test]
fn test_bad_null() {
    let yaml = indoc!("
        ---
        !!null str");
    let expected = "invalid value: string \"str\", expected null at line 2 column 8";
    test_error::<()>(yaml, expected);
}

#[test]
fn test_short_tuple() {
    let yaml = indoc!("
        ---
        [0, 0]");
    let expected = "invalid length 2, expected a tuple of size 3 at line 2 column 1";
    test_error::<(u8, u8, u8)>(yaml, expected);
}

#[test]
fn test_long_tuple() {
    let yaml = indoc!("
        ---
        [0, 0, 0]");
    let expected = "invalid length 3, expected sequence of 2 elements at line 2 column 1";
    test_error::<(u8, u8)>(yaml, expected);
}
