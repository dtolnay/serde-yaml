Serde YAML Serialization Library
================================

[![Build Status](https://api.travis-ci.org/dtolnay/serde-yaml.svg?branch=master)](https://travis-ci.org/dtolnay/serde-yaml)
[![Latest Version](https://img.shields.io/crates/v/serde_yaml.svg)](https://crates.io/crates/serde_yaml)
[![Rust Documentation](https://img.shields.io/crates/v/serde_yaml.svg?label=rustdoc)](https://dtolnay.github.io/serde-yaml/serde_yaml/)

This crate is a Rust library for using the [Serde](https://github.com/serde-rs/serde)
serialization framework with data in [YAML](http://yaml.org) file format. This
library does not reimplement a YAML parser; it uses [yaml-rust](https://github.com/chyh1990/yaml-rust)
which is a pure Rust YAML 1.2 implementation.

Installation
============

Version 0.2.x of this crate works with 0.7.x of Serde. Both can be found on
[crates.io](https://crates.io/crates/serde_yaml) with a `Cargo.toml` like:

```toml
[dependencies]
serde = "^0.7"
serde_yaml = "^0.2"
```

Release notes are available under [GitHub releases](https://github.com/dtolnay/serde-yaml/releases).

Using Serde YAML
================

[API documentation is available in rustdoc form](https://dtolnay.github.io/serde-yaml/)
but the general idea is:

```rust
extern crate serde;
extern crate serde_yaml;

use std::collections::BTreeMap;

fn main() {
    let mut map = BTreeMap::new();
    map.insert("x".to_string(), 1.0);
    map.insert("y".to_string(), 2.0);

    let s = serde_yaml::to_string(&map).unwrap();
    assert_eq!(s, "---\n\"x\": 1\n\"y\": 2");

    let deserialized_map: BTreeMap<String, f64> = serde_yaml::from_str(&s).unwrap();
    assert_eq!(map, deserialized_map);
}
```

It can also be used with Serde's automatic serialization library,
`serde_macros`. First add this to `Cargo.toml`:

```toml
[dependencies]
serde = "^0.7"
serde_macros = "^0.7"
serde_yaml = "^0.2"
```

Then use:

```rust
#![feature(plugin)]
#![plugin(serde_macros)]

extern crate serde;
extern crate serde_yaml;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Point {
    x: f64,
    y: f64,
}

fn main() {
    let point = Point { x: 1.0, y: 2.0 };

    let s = serde_yaml::to_string(&point).unwrap();
    assert_eq!(s, "---\n\"x\": 1\n\"y\": 2");

    let deserialized_point: Point = serde_yaml::from_str(&s).unwrap();
    assert_eq!(point, deserialized_point);
}
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

Build scripts are licensed under Creative Commons CC0 1.0 Universal
([LICENSE-CC](LICENSE-CC) or https://creativecommons.org/publicdomain/zero/1.0/legalcode).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Serde YAML by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
