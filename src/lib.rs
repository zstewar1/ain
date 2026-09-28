// Copyright 2026 Zachary Stewart. See the COPYRIGHT file at the top-level directory of this
// distribution and at http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be copied, modified, or
// distributed except according to those terms.
//
// Parts of this code are copied from the `ascii` crate and the Rust standard library under the same
// licenses.

//! # ASCII Insensitive
//!
//! This crate provides ASCII chars and string, similar to the
//! [`ascii`](https://crates.io/crates/ascii) crate, but with case-insensitive comparisons by
//! default.
//!
//! This allows you to store strings in containers like maps while allowing case-insensitive
//! lookups.
//!
//! It provides the usual set of things you would expect from a string-like library in Rust:
//!
//! *   [`AinString`]: an owning container like [`String`]
//! *   [`AinStr`]: a borrowed slice like [`str`]
//! *   [`AinChar`]: an individual element like [`char`]
//!
//! [`AinStr`] and [`AinChar`] constants can be created from literals using the [`ain!`] macro:
//!
//! ```
//! # use ain::{ain, AinStr, AinChar};
//! const STR: &'static AinStr = ain!("Hello World");
//! // assert!(STR.contains("hello"));
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "nightly", feature(ascii_char))]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "alloc")]
extern crate alloc;

#[macro_use]
mod macros;

#[doc(hidden)]
pub use macros::Literal;

mod ain_char;
mod ain_str;
#[cfg(feature = "alloc")]
mod ain_string;
#[cfg(feature = "alloc")]
mod alloc_conversions;
#[cfg(feature = "ascii")]
mod ascii_conversions;
pub mod iter;
#[cfg(feature = "nightly")]
mod nightly_conversions;
pub mod pattern;
mod repr;
mod validation;

pub use ain_char::{AinChar, TryIntoAinCharError};
pub use ain_str::{AinSliceIndexOutputMap, AinStr};
#[cfg(feature = "alloc")]
pub use ain_string::{AinString, AinStringFromError};
pub use validation::AinValidationError;
