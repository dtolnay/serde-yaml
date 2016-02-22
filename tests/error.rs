#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate serde;
extern crate serde_yaml;

use std::fmt::Debug;

fn test_error<T>(yaml: &str, expected: &str)
    where T: serde::Deserialize + Debug
{
    let result = serde_yaml::from_str::<T>(yaml);
    assert_eq!(expected, format!("{:?}", result.unwrap_err()));
}

#[test]
fn test_incorrect_type() {
    let yaml = "---\n\
                str";
    let expected = "syntax error: incorrect type";
    test_error::<i16>(yaml, expected);
}

#[test]
fn test_empty() {
    let yaml = "";
    let expected = "EOF while parsing a value";
    test_error::<String>(yaml, expected);
}

#[test]
fn test_unknown_field() {
    #[derive(Deserialize, Debug)]
    struct Basic {
        v: bool,
    }
    let yaml = "---\n\
                x: true";
    let expected = "unknown field \"x\"";
    test_error::<Basic>(yaml, expected);
}

#[test]
fn test_missing_field() {
    #[derive(Deserialize, Debug)]
    struct Basic {
        v: bool,
        w: bool,
    }
    let yaml = "---\n\
                v: true";
    let expected = "missing field \"w\"";
    test_error::<Basic>(yaml, expected);
}

#[test]
fn test_unknown_anchor() {
    let yaml = "---\n\
                *some";
    let expected =
        "ScanError { \
          mark: Marker { \
            index: 4, \
            line: 2, \
            col: 0 \
          }, \
          info: \"while parsing node, found unknown anchor\" \
        }";
    test_error::<String>(yaml, expected);
}

#[test]
fn test_two_documents() {
    let yaml = "---\n\
                0\n\
                ---\n\
                1";
    let expected = "expected a single YAML document but found 2";
    test_error::<usize>(yaml, expected);
}

#[test]
fn test_variant_map_wrong_size() {
    #[derive(Deserialize, Debug)]
    enum Variant {
        V(usize),
    }
    let yaml = "---\n\
                \"V\": 16\n\
                \"other\": 32";
    let expected = "expected a YAML map of size 1 while parsing variant Variant but was size 2";
    test_error::<Variant>(yaml, expected);
}

#[test]
fn test_variant_not_a_map() {
    #[derive(Deserialize, Debug)]
    enum Variant {
        V(usize),
    }
    let yaml = "---\n\
                - \"V\"";
    let expected = "expected a YAML map while parsing variant Variant";
    test_error::<Variant>(yaml, expected);
}
