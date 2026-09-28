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
#![cfg_attr(feature = "nightly", feature(ascii_char))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "std")]
extern crate core;

macro_rules! borrow_from_as_ref {
    ($from:ty, $to:ty) => {
        borrow_from_as_ref!(const $from, $to);
        borrow_from_as_ref!(mut $from, $to);
    };
    (const $from:ty, $to:ty) => {
        impl Borrow<$to> for $from {
            fn borrow(&self) -> &$to {
                self.as_ref()
            }
        }
    };
    (mut $from:ty, $to:ty) => {
        impl BorrowMut<$to> for $from {
            fn borrow_mut(&mut self) -> &mut $to {
                self.as_mut()
            }
        }
    };
}

macro_rules! from_ref_from_as_ref {
    ($from:ty, $to:ty) => {
        from_ref_from_as_ref!(const $from, $to);
        from_ref_from_as_ref!(mut $from, $to);
    };
    (const $from:ty, $to:ty) => {
        impl<'a> From<&'a $from> for &'a $to {
            fn from(value: &$from) -> &$to {
                value.as_ref()
            }
        }
    };
    (mut $from:ty, $to:ty) => {
        impl<'a> From<&'a mut $from> for &'a mut $to {
            fn from(value: &mut $from) -> &mut $to {
                value.as_mut()
            }
        }
    };
}

mod ain_char;
mod ain_str;
#[cfg(feature = "alloc")]
mod ain_string;
#[cfg(feature = "alloc")]
mod alloc_conversions;
#[cfg(feature = "ascii")]
mod ascii_conversions;
#[cfg(feature = "nightly")]
mod nightly_conversions;
mod validation;

pub use ain_char::{AinChar, TryIntoAinCharError};
pub use ain_str::{AinSliceIndexOutputMap, AinStr, Lines, Split, SplitMut};
#[cfg(feature = "alloc")]
pub use ain_string::{AinString, IntoAinStringError};
pub use validation::AinValidationError;
