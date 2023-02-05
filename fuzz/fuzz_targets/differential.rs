#![no_main]
use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, PartialEq, Debug, Arbitrary)]
enum Enum {
    Unit,
    Newtype(usize),
    Tuple(usize, usize),
    Struct { value: usize },
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Arbitrary)]
struct SingletonMapStruct {
    #[serde(with = "serde_yaml::with::singleton_map")]
    w: Enum,
    #[serde(with = "serde_yaml::with::singleton_map")]
    x: Enum,
    #[serde(with = "serde_yaml::with::singleton_map")]
    y: Enum,
    #[serde(with = "serde_yaml::with::singleton_map")]
    z: Enum,
}

#[derive(Debug, PartialEq, Arbitrary, Serialize, Deserialize)]
enum All {
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    I128(i128),
    U128(u128),
    USize(usize),
    Char(char),
    String(String),
    Bool(bool),
    Tuple(usize, usize, usize),
    EnumStruct { x: String, y: i64, z: Vec<All> },
    Struct(SingletonMapStruct),
    BTree(BTreeMap<String, All>),
}

fuzz_target!(|map: All| {
    let yaml = serde_yaml::to_string(&map).unwrap();
    // This should never fail so we unwrap.
    let deserialized_map: All = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(map, deserialized_map);
});
