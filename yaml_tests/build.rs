// Copyright 2016 Serde YAML Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[cfg(feature = "with-syntex")]
mod with_syntex {
    extern crate syntex;
    extern crate serde_codegen;
    extern crate indoc;

    use std::env;
    use std::path::Path;

    pub fn main() {
        let out_dir = env::var_os("OUT_DIR").unwrap();

        let src = Path::new("tests/test.rs.in");
        let dst = Path::new(&out_dir).join("test.rs");

        let mut registry = syntex::Registry::new();

        serde_codegen::register(&mut registry);
        indoc::register(&mut registry);
        registry.expand("", &src, &dst).unwrap();
    }
}

#[cfg(not(feature = "with-syntex"))]
mod with_syntex {
    pub fn main() {}
}

pub fn main() {
    with_syntex::main();
}
