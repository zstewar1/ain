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
//! This crate provides a wrapper for ASCII strings, based on the [ascii] crate, but with
//! case-insensitive comparisons by default.
//!
//! This allows you to store strings in containers like maps while allowing case-insensitive
//! lookups.
//!
//! It provides the usual set of things you would expect from a string-like library in Rust:
//!
//! *   [`AinString`]: an owning container like [`String`]
//! *   [`AinStr`]: a borrowed slice like [`str`]
//! *   [`AinChar`]: an individual element like [`char`]

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
#[macro_use]
extern crate alloc;
#[cfg(feature = "std")]
extern crate core;

mod ain_char;
mod ain_str;
#[cfg(feature = "alloc")]
mod ain_string;

pub use ain_char::{AinChar, ToAinChar, ToAinCharError};
pub use ain_str::{AinStr, AsAinStrError, AsAinStr, AsMutAinStr};
#[cfg(feature = "alloc")]
pub use ain_string::{AinString, IntoAinStringError};
