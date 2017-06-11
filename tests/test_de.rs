// Copyright 2016 Serde YAML Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_yaml;

extern crate unindent;
use unindent::unindent;

use std::fmt::Debug;
use std::collections::BTreeMap;

fn test_de<T>(yaml: &str, expected: &T)
    where T: serde::de::DeserializeOwned + PartialEq + Debug
{
    let deserialized: T = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(*expected, deserialized);
}

#[test]
fn test_alias() {
    let yaml = unindent("
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
    test_de(&yaml, &expected);
}

#[test]
fn test_option() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Data {
        a: Option<f64>,
        b: Option<String>,
        c: Option<bool>,
    }
    let yaml = unindent("
        ---
        b:
        c: true");
    let expected = Data {
        a: None,
        b: None,
        c: Some(true),
    };
    test_de(&yaml, &expected);
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
    let yaml = unindent("
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
    test_de(&yaml, &expected);
}

#[test]
fn test_enum_alias() {
    #[derive(Deserialize, PartialEq, Debug)]
    enum E {
        A,
        B(u8, u8),
    }
    #[derive(Deserialize, PartialEq, Debug)]
    struct Data {
        a: E,
        b: E,
    }
    let yaml = unindent("
        ---
        aref:
          &aref
          A
        bref:
          &bref
          B:
            - 1
            - 2

        a: *aref
        b: *bref");
    let expected = Data {
        a: E::A,
        b: E::B(1, 2),
    };
    test_de(&yaml, &expected);
}

#[test]
fn test_number_as_string() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Num {
        value: String,
    }
    let yaml = unindent("
        ---
        value: 123456789012345678901234567890");
    let expected = Num { value: "123456789012345678901234567890".to_owned() };
    test_de(&yaml, &expected);
}

#[test]
fn test_number_alias_as_string() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Num {
        version: String,
        value: String,
    }
    let yaml = unindent("
        ---
        version: &a 1.10
        value: *a");
    let expected = Num { version: "1.10".to_owned(), value: "1.10".to_owned() };
    test_de(&yaml, &expected);
}

#[test]
fn test_de_mapping() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Data {
        pub substructure: serde_yaml::Mapping,
    }
    let yaml = unindent("
        ---
        substructure:
          a: 'foo'
          b: 'bar'");

    let mut expected = Data { substructure: serde_yaml::Mapping::new() };
    expected.substructure.insert(serde_yaml::Value::String("a".to_owned()),
                                 serde_yaml::Value::String("foo".to_owned()));
    expected.substructure.insert(serde_yaml::Value::String("b".to_owned()),
                                 serde_yaml::Value::String("bar".to_owned()));

    test_de(&yaml, &expected);
}
