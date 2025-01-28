# nom-parse-macros

[![CI](https://github.com/marcdejonge/nom-parse-macros/actions/workflows/ci.yml/badge.svg)](https://github.com/marcdejonge/nom-parse-macros/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/nom-parse-macros.svg)](https://crates.io/crates/nom-parse-macros)
[![Documentation](https://docs.rs/nom-parse-macros/badge.svg)](https://docs.rs/nom-parse-macros)

This crate provides 2 macros to generate a `ParseFrom` implementation for a struct
or enum using the provided nom expression(s). The expression given should return a
tuple for the parsed fields.

There are 2 separate macros available, `parse_from` and `parse_match`. The first
one is really generic and can be useful in many cases, since you have the full
flexibility of nom functions and combinators. The second one is a very simple 
one that matches a string verbatim. This is useful when you have a very simple
format that you want to parse.

As a quick example, consider the following struct:

```rust
use nom_parse_macros::parse_from;

#[parse_from((be_32, be_u16))]
struct MyStruct {
    a: u32,
    b: u16,
}
```

After this, you can easily call `MyStruct::parse` with a byte array and get a
parsed instance back. You have many of the nom functions available to you, so
you can parse more complex structures as well. Look at the documentation for
more examples and more explanation on how to use the macros.

## License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.