# ASCII Insenstive (ain)

This crate provides a wrapper for ASCII strings, based on the
[ascii](https://crates.io/crates/ascii) crate, but with case-insensitive comparisons by default.

This allows you to store strings in containers like maps while allowing case-insensitive lookups.

It provides the usual set of things you would expect from a string-like library in Rust:

*   `AinString`: an owning container like [`String`](https://doc.rust-lang.org/std/string/struct.String.html)
*   `AinStr`: a borrowed slice like [`str`](https://doc.rust-lang.org/std/primitive.str.html)
*   `AinChar`: an individual element like [`char`](https://doc.rust-lang.org/std/primitive.char.html)

## Licensing

Licensed under either of

*   Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
*   MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

at your option.

Portions of this library are copied verbatim or with modification from
[ascii](https://crates.io/crates/ascii), and the Rust standard library under the same license terms
(MIT or Apache-2.0).
