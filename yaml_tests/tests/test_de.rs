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
use std::collections::BTreeMap;

fn test_de<T>(yaml: &str, expected: T)
    where T: serde::Deserialize + PartialEq + Debug,
{
    let deserialized: T = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(expected, deserialized);
}

#[test]
fn test_alias() {
    let yaml = indoc!("
        ---
        first:
          &alias
          1
        second:
          *alias
        third: 3");
    let mut expected = BTreeMap::new();
    {
        expected.insert(String::from("first"), 1);
        expected.insert(String::from("second"), 1);
        expected.insert(String::from("third"), 3);
    }
    test_de(yaml, expected);
}

#[test]
fn test_option() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Data {
        a: Option<f64>,
        b: Option<String>,
        c: Option<bool>,
    }
    let yaml = indoc!("
        ---
        b:
        c: true");
    let expected = Data {
        a: None,
        b: None,
        c: Some(true),
    };
    test_de(yaml, expected);
}

#[test]
fn test_option_alias() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Data {
        a: Option<f64>,
        b: Option<String>,
        c: Option<bool>,
        d: Option<f64>,
        e: Option<String>,
        f: Option<bool>,
    }
    let yaml = indoc!("
        ---
        none_f:
            &none_f
            ~
        none_s:
            &none_s
            ~
        none_b:
            &none_b
            ~

        some_f:
            &some_f
            1.0
        some_s:
            &some_s
            x
        some_b:
            &some_b
            true

        a: *none_f
        b: *none_s
        c: *none_b
        d: *some_f
        e: *some_s
        f: *some_b");
    let expected = Data {
        a: None,
        b: None,
        c: None,
        d: Some(1.0),
        e: Some("x".to_owned()),
        f: Some(true),
    };
    test_de(yaml, expected);
}
