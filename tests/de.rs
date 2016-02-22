extern crate serde;
extern crate serde_yaml;

use std::fmt::Debug;
use std::collections::BTreeMap;

fn test_de<T>(yaml: &str, expected: T)
    where T: serde::Deserialize + PartialEq + Debug
{
    let deserialized: T = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(expected, deserialized);
}

#[test]
fn test_alias() {
    let yaml = "---\n\
                first:\n  \
                  &alias\n  \
                  1\n\
                second:\n  \
                  *alias";
    let mut expected = BTreeMap::new();
    {
        expected.insert(String::from("first"), 1);
        expected.insert(String::from("second"), 1);
    }
    test_de(yaml, expected);
}
