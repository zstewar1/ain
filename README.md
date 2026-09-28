# ASCII Insenstive (ain)

This crate provides a types for case-insensitive ASCII strings.

This allows you to store ASCII strings in containers like maps while allowing case-insensitive
lookups.

It provides the usual set of things you would expect from a string-like library in Rust:

*   `AinString`: an owning container like [`String`](https://doc.rust-lang.org/std/string/struct.String.html)
*   `AinStr`: a borrowed slice like [`str`](https://doc.rust-lang.org/std/primitive.str.html)
*   `AinChar`: an individual element like [`char`](https://doc.rust-lang.org/std/primitive.char.html)

The types optionally offer zero-cost, infallible, bidirectional conversion with the types from the
[`ascii`](https://crates.io/crates/ascii) crate, which can be enabled with the `ascii` feature. Like
the `ascii` crate, we offer `no_std` support.

## Similar Crates

### [`uncased`](https://crates.io/crates/uncased):

`uncased` also offers types with built in comparsisons and hashing that are not case sensitive to
ASCII case, though it has some significant differences to this library:

*   **Not limited to ASCII**

    *   Types in `uncased` can hold arbitrary unicode data. This allows infallible conversion from
        `&str`, but comparsions are still only case-insensitive in the ASCII range.

    *   `ain`'s types are all restricted to *only* hold 7-bit `ascii` data.

*   **Limited API**

    *   `uncased` provides only two types: a slice-like borrowed string (`UncasedStr`) and a `Cow`
        wrapper string (`Uncased`). There's no option for a purely owned string other than
        `Uncased<'static>`.

    *   `ain` provides a full set of types including analogues for `char`, `&str`, and `String`.
        Copy-on-write works with the normal built-in `Cow` type.

## License & Attribution

Licensed under either of

*   Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
*   MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

at your option.

This library includes heavily modified code snippets, algorithms, and logic derived from the Rust
Standard Library and the [`ascii`](https://crates.io/crates/ascii) crate.

Original Copyright:

Copyright (c) 2010-2014 The Rust Project Developers Licensed under the Apache License, Version 2.0
and the MIT License.
