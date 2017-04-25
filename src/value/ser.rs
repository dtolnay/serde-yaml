use serde::{self, Serialize};

use super::Value;

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer
    {
        match *self {
            Value::Null => serializer.serialize_unit(),
            Value::Bool(b) => serializer.serialize_bool(b),
            Value::Number(ref n) => {
                if let Some(u) = n.as_u64() {
                    serializer.serialize_u64(u)
                } else if let Some(i) = n.as_i64() {
                    serializer.serialize_i64(i)
                } else if let Some(f) = n.as_f64() {
                    serializer.serialize_f64(f)
                } else {
                    unreachable!("unexpected number")
                }
            }
            Value::String(ref s) => serializer.serialize_str(s),
            Value::Sequence(ref seq) => seq.serialize(serializer),
            Value::Mapping(ref hash) => {
                use serde::ser::SerializeMap;
                let mut map = serializer.serialize_map(Some(hash.len()))?;
                for (k, v) in hash {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
        }
    }
}
