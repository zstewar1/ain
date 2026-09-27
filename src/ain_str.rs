use core::ops::{Index, IndexMut};
use core::slice::{Iter, IterMut, SliceIndex};
use core::{fmt, slice};
#[cfg(feature = "alloc")]
use alloc::{
    boxed::Box,
    borrow::ToOwned,
};

pub use ascii::AsAsciiStrError as AsAinStrError;
use ascii::{AsciiChar, AsAsciiStr, AsMutAsciiStr, AsciiStr};

use crate::AinChar;
#[cfg(feature = "alloc")]
use crate::AinString;

/// [`AinStr`] represents a byte or string slice that only contains ASCII characters.
///
/// It wraps an [`AinChar`] and implements many of `str`s methods and traits.
///
/// It can be created by a checked conversion from a `str` or `[u8]`, or borrowed from an
/// `AinString`.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct AinStr {
    slice: [AinChar],
}

// This is mostly copied from ascii::AsciiStr.
impl AinStr {
    /// Converts `&self` to a `&str` slice.
    #[inline]
    #[must_use]
    pub const fn as_str(&self) -> &str {
        // SAFETY: All variants of `AinChar` are valid bytes for a `str`.
        unsafe { &*(self as *const AinStr as *const str) }
    }

    /// Converts `&self` into a byte slice.
    #[inline]
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8] {
        // SAFETY: All variants of `AinChar` are valid `u8`, given they're `repr(u8)` via AsciiChar.
        unsafe { &*(self as *const AinStr as *const [u8]) }
    }

    /// Returns the entire string as slice of `AinChar`s.
    #[inline]
    #[must_use]
    pub const fn as_slice(&self) -> &[AinChar] {
        &self.slice
    }

    /// Returns the entire string as mutable slice of `AinChar`s.
    #[inline]
    #[must_use]
    pub const fn as_mut_slice(&mut self) -> &mut [AinChar] {
        &mut self.slice
    }

    /// Returns a raw pointer to the `AinStr`'s buffer.
    ///
    /// The caller must ensure that the slice outlives the pointer this function returns, or else it
    /// will end up pointing to garbage. Modifying the `AinStr` may cause it's buffer to be
    /// reallocated, which would also make any pointers to it invalid.
    #[inline]
    #[must_use]
    pub const fn as_ptr(&self) -> *const AinChar {
        self.as_slice().as_ptr()
    }

    /// Returns an unsafe mutable pointer to the `AinStr`'s buffer.
    ///
    /// The caller must ensure that the slice outlives the pointer this function returns, or else it
    /// will end up pointing to garbage. Modifying the `AinStr` may cause it's buffer to be
    /// reallocated, which would also make any pointers to it invalid.
    #[inline]
    #[must_use]
    pub const fn as_mut_ptr(&mut self) -> *mut AinChar {
        self.as_mut_slice().as_mut_ptr()
    }

    /// Copies the content of this `AinStr` into an owned `AinString`.
    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn to_ain_string(&self) -> AinString {
        use crate::AinString;

        AinString::from(self.slice.to_vec())
    }

    /// Converts anything that can represent a byte slice into an `AinStr`.
    ///
    /// # Errors
    /// If `bytes` contains a non-ascii byte, `Err` will be returned
    ///
    /// # Examples
    /// ```
    /// # use ain::AinStr;
    /// let foo = AinStr::from_ascii(b"foo");
    /// let err = AinStr::from_ascii("Ŋ");
    /// assert_eq!(foo.unwrap().as_str(), "foo");
    /// assert_eq!(err.unwrap_err().valid_up_to(), 0);
    /// ```
    #[inline]
    pub fn from_ascii<B>(bytes: &B) -> Result<&AinStr, AsAinStrError>
    where
        B: AsAinStr + ?Sized,
    {
        bytes.as_ain_str()
    }

    /// Converts anything that can be represented as a byte slice to an `AinStr` without checking
    /// for non-ASCII characters.
    ///
    /// # Safety
    /// If any of the bytes in `bytes` do not represent valid ascii characters, calling
    /// this function is undefined behavior.
    ///
    /// # Examples
    /// ```
    /// # use ain::AinStr;
    /// let foo = unsafe { AinStr::from_ascii_unchecked(&b"foo"[..]) };
    /// assert_eq!(foo.as_str(), "foo");
    /// ```
    #[inline]
    #[must_use]
    pub unsafe fn from_ascii_unchecked(bytes: &[u8]) -> &AinStr {
        // SAFETY: Caller guarantees all bytes in `bytes` are valid
        //         ascii characters.
        unsafe { bytes.as_ain_str_unchecked() }
    }

    /// Converts anything that can be represented as a mutable byte slice to an `AinStr` without
    /// checking for non-ASCII characters.
    ///
    /// # Safety
    /// If any of the bytes in `bytes` do not represent valid ascii characters, calling
    /// this function is undefined behavior.
    ///
    /// # Examples
    /// ```
    /// # use ain::AinStr;
    /// let foo = unsafe { AinStr::from_ascii_unchecked_mut(&mut b"foo"[..]) };
    /// assert_eq!(foo.as_str(), "foo");
    /// ```
    #[inline]
    #[must_use]
    pub unsafe fn from_ascii_unchecked_mut(bytes: &mut [u8]) -> &mut AinStr {
        // SAFETY: Caller guarantees all bytes in `bytes` are valid
        //         ascii characters.
        unsafe { bytes.as_mut_ain_str_unchecked() }
    }

    /// Returns the number of characters / bytes in this ASCII sequence.
    ///
    /// # Examples
    /// ```
    /// # use ain::AinStr;
    /// let s = AinStr::from_ascii("foo").unwrap();
    /// assert_eq!(s.len(), 3);
    /// ```
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.slice.len()
    }

    /// Returns true if the ASCII slice contains zero bytes.
    ///
    /// # Examples
    /// ```
    /// # use ain::AinStr;
    /// let mut empty = AinStr::from_ascii("").unwrap();
    /// let mut full = AinStr::from_ascii("foo").unwrap();
    /// assert!(empty.is_empty());
    /// assert!(!full.is_empty());
    /// ```
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an iterator over the characters of the `AinStr`.
    #[inline]
    #[must_use]
    pub fn chars(&self) -> Chars<'_> {
        Chars(self.slice.iter())
    }

    /// Returns an iterator over the characters of the `AinStr` which allows you to modify the
    /// value of each `AinChar`.
    #[inline]
    #[must_use]
    pub fn chars_mut(&mut self) -> CharsMut<'_> {
        CharsMut(self.slice.iter_mut())
    }

    /// Returns an iterator over parts of the `AinStr` separated by a character.
    ///
    /// # Examples
    /// ```
    /// # use ain::{AinStr, AinChar};
    /// let words = AinStr::from_ascii("apple banana lemon").unwrap()
    ///     .split(AinChar::Space)
    ///     .map(|a| a.as_str())
    ///     .collect::<Vec<_>>();
    /// assert_eq!(words, ["apple", "banana", "lemon"]);
    /// ```
    #[must_use]
    pub fn split(&self, on: AinChar) -> impl DoubleEndedIterator<Item = &AinStr> {
        Split {
            on,
            ended: false,
            chars: self.chars(),
        }
    }

    /// Returns an iterator over the lines of the `AinStr`, which are themselves `AinStr`s.
    ///
    /// Lines are ended with either `LineFeed` (`\n`), or `CarriageReturn` then `LineFeed` (`\r\n`).
    ///
    /// The final line ending is optional.
    #[inline]
    #[must_use]
    pub fn lines(&self) -> impl DoubleEndedIterator<Item = &AinStr> {
        Lines { string: self }
    }

    /// Returns an ASCII string slice with leading and trailing whitespace removed.
    ///
    /// # Examples
    /// ```
    /// # use ain::AinStr;
    /// let example = AinStr::from_ascii("  \twhite \tspace  \t").unwrap();
    /// assert_eq!("white \tspace", example.trim());
    /// ```
    #[must_use]
    pub fn trim(&self) -> &Self {
        self.trim_start().trim_end()
    }

    /// Returns an ASCII string slice with leading whitespace removed.
    ///
    /// # Examples
    /// ```
    /// # use ain::AinStr;
    /// let example = AinStr::from_ascii("  \twhite \tspace  \t").unwrap();
    /// assert_eq!("white \tspace  \t", example.trim_start());
    /// ```
    #[must_use]
    pub fn trim_start(&self) -> &Self {
        let whitespace_len = self
            .chars()
            .position(|ch| !ch.is_whitespace())
            .unwrap_or_else(|| self.len());

        // SAFETY: `whitespace_len` is `0..=len`, which is at most `len`, which is a valid empty slice.
        unsafe { self.as_slice().get_unchecked(whitespace_len..).into() }
    }

    /// Returns an ASCII string slice with trailing whitespace removed.
    ///
    /// # Examples
    /// ```
    /// # use ain::AinStr;
    /// let example = AinStr::from_ascii("  \twhite \tspace  \t").unwrap();
    /// assert_eq!("  \twhite \tspace", example.trim_end());
    /// ```
    #[must_use]
    pub fn trim_end(&self) -> &Self {
        // Number of whitespace characters counting from the end
        let whitespace_len = self
            .chars()
            .rev()
            .position(|ch| !ch.is_whitespace())
            .unwrap_or_else(|| self.len());

        // SAFETY: `whitespace_len` is `0..=len`, which is at most `len`, which is a valid empty slice, and at least `0`, which is the whole slice.
        unsafe {
            self.as_slice()
                .get_unchecked(..self.len() - whitespace_len)
                .into()
        }
    }

    /// Replaces lowercase letters with their uppercase equivalent.
    ///
    /// Since the string is case insensitive, this does not change the equality comparions or hash
    /// of the string.
    pub fn make_uppercase(&mut self) {
        for ch in self.chars_mut() {
            ch.make_uppercase()
        }
    }

    /// Replaces uppercase letters with their lowercase equivalent.
    ///
    /// Since the string is case insensitive, this does not change the equality comparions or hash
    /// of the string.
    pub fn make_lowercase(&mut self) {
        for ch in self.chars_mut() {
            ch.make_lowercase()
        }
    }

    /// Returns a copy of this string where letters 'a' to 'z' are mapped to 'A' to 'Z'.
    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn to_uppercase(&self) -> AinString {
        let mut ascii_string = self.to_ain_string();
        ascii_string.make_uppercase();
        ascii_string
    }

    /// Returns a copy of this string where letters 'A' to 'Z' are mapped to 'a' to 'z'.
    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn to_lowercase(&self) -> AinString {
        let mut ascii_string = self.to_ain_string();
        ascii_string.make_lowercase();
        ascii_string
    }

    /// Returns the first character if the string is not empty.
    #[inline]
    #[must_use]
    pub fn first(&self) -> Option<AinChar> {
        self.slice.first().copied()
    }

    /// Returns the last character if the string is not empty.
    #[inline]
    #[must_use]
    pub fn last(&self) -> Option<AinChar> {
        self.slice.last().copied()
    }

    /// Converts a [`Box<AinStr>`] into a [`AinString`] without copying or allocating.
    #[cfg(feature = "alloc")]
    #[inline]
    #[must_use]
    pub fn into_ain_string(self: Box<Self>) -> AinString {
        let slice = Box::<[AinChar]>::from(self);
        AinString::from(slice.into_vec())
    }
}

impl PartialEq<[AinChar]> for AinStr {
    #[inline]
    fn eq(&self, other: &[AinChar]) -> bool {
        <AinStr as AsRef<[AinChar]>>::as_ref(self) == other
    }
}
impl PartialEq<AinStr> for [AinChar] {
    #[inline]
    fn eq(&self, other: &AinStr) -> bool {
        self == <AinStr as AsRef<[AinChar]>>::as_ref(other)
    }
}

#[cfg(feature = "alloc")]
impl ToOwned for AinStr {
    type Owned = AinString;

    #[inline]
    fn to_owned(&self) -> AinString {
        self.to_ain_string()
    }
}

impl AsRef<[u8]> for AinStr {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}
impl AsRef<str> for AinStr {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl AsRef<[AinChar]> for AinStr {
    #[inline]
    fn as_ref(&self) -> &[AinChar] {
        &self.slice
    }
}
impl AsMut<[AinChar]> for AinStr {
    #[inline]
    fn as_mut(&mut self) -> &mut [AinChar] {
        &mut self.slice
    }
}

impl Default for &'static AinStr {
    #[inline]
    fn default() -> &'static AinStr {
        From::from(&[] as &[AinChar])
    }
}
impl<'a> From<&'a [AinChar]> for &'a AinStr {
    #[inline]
    fn from(slice: &[AinChar]) -> &AinStr {
        let ptr = slice as *const [AinChar] as *const AinStr;
        unsafe { &*ptr }
    }
}
impl<'a> From<&'a mut [AinChar]> for &'a mut AinStr {
    #[inline]
    fn from(slice: &mut [AinChar]) -> &mut AinStr {
        let ptr = slice as *mut [AinChar] as *mut AinStr;
        unsafe { &mut *ptr }
    }
}

#[cfg(feature = "alloc")]
impl From<Box<[AinChar]>> for Box<AinStr> {
    #[inline]
    fn from(owned: Box<[AinChar]>) -> Box<AinStr> {
        let ptr = Box::into_raw(owned) as *mut AinStr;
        unsafe { Box::from_raw(ptr) }
    }
}

#[cfg(feature = "alloc")]
impl From<Box<[AsciiChar]>> for Box<AinStr> {
    #[inline]
    fn from(owned: Box<[AsciiChar]>) -> Box<AinStr> {
        let ptr = Box::into_raw(owned) as *mut AinStr;
        unsafe { Box::from_raw(ptr) }
    }
}

impl AsRef<AinStr> for AinStr {
    #[inline]
    fn as_ref(&self) -> &AinStr {
        self
    }
}
impl AsMut<AinStr> for AinStr {
    #[inline]
    fn as_mut(&mut self) -> &mut AinStr {
        self
    }
}
impl AsRef<AinStr> for [AinChar] {
    #[inline]
    fn as_ref(&self) -> &AinStr {
        self.into()
    }
}
impl AsMut<AinStr> for [AinChar] {
    #[inline]
    fn as_mut(&mut self) -> &mut AinStr {
        self.into()
    }
}
impl AsRef<AsciiStr> for AinStr {
    #[inline]
    fn as_ref(&self) -> &AsciiStr {
        self.into()
    }
}
impl AsMut<AsciiStr> for AinStr {
    #[inline]
    fn as_mut(&mut self) -> &mut AsciiStr {
        self.into()
    }
}
impl AsRef<AinStr> for AsciiStr {
    #[inline]
    fn as_ref(&self) -> &AinStr {
        self.into()
    }
}
impl AsMut<AinStr> for AsciiStr {
    #[inline]
    fn as_mut(&mut self) -> &mut AinStr {
        self.into()
    }
}
impl AsRef<AinStr> for [AsciiChar] {
    #[inline]
    fn as_ref(&self) -> &AinStr {
        self.into()
    }
}
impl AsMut<AinStr> for [AsciiChar] {
    #[inline]
    fn as_mut(&mut self) -> &mut AinStr {
        self.into()
    }
}

impl<'a> From<&'a AinStr> for &'a [AinChar] {
    #[inline]
    fn from(astr: &AinStr) -> &[AinChar] {
        &astr.slice
    }
}
impl<'a> From<&'a mut AinStr> for &'a mut [AinChar] {
    #[inline]
    fn from(astr: &mut AinStr) -> &mut [AinChar] {
        &mut astr.slice
    }
}
impl<'a> From<&'a AinStr> for &'a [AsciiChar] {
    #[inline]
    fn from(astr: &AinStr) -> &[AsciiChar] {
        // SAFETY: Both AsciiChar and AinChar have the same repr.
        let ptr = astr.as_slice() as *const [AinChar] as *const [AsciiChar];
        // SAFETY: Ptr came from a ref with same mutability.
        unsafe { &*ptr }
    }
}
impl<'a> From<&'a mut AinStr> for &'a mut [AsciiChar] {
    #[inline]
    fn from(astr: &mut AinStr) -> &mut [AsciiChar] {
        // SAFETY: Both AsciiChar and AinChar have the same repr.
        let ptr = astr.as_mut_slice() as *mut [AinChar] as *mut [AsciiChar];
        // SAFETY: Ptr came from a ref with same mutability.
        unsafe { &mut *ptr }
    }
}
impl<'a> From<&'a [AsciiChar]> for &'a AinStr {
    #[inline]
    fn from(astr: &[AsciiChar]) -> &AinStr {
        // SAFETY: Both AsciiChar and AinChar have the same repr.
        let ptr = astr as *const [AsciiChar] as *const [AinChar];
        // SAFETY: Ptr came from a ref with same mutability.
        unsafe { &*ptr }.into()
    }
}
impl<'a> From<&'a mut [AsciiChar]> for &'a mut AinStr {
    #[inline]
    fn from(astr: &mut [AsciiChar]) -> &mut AinStr {
        // SAFETY: Both AsciiChar and AinChar have the same repr.
        let ptr = astr as *mut [AsciiChar] as *mut [AinChar];
        // SAFETY: Ptr came from a ref with same mutability.
        unsafe { &mut *ptr }.into()
    }
}
impl<'a> From<&'a AinStr> for &'a [u8] {
    #[inline]
    fn from(astr: &AinStr) -> &[u8] {
        astr.as_bytes()
    }
}
impl<'a> From<&'a AinStr> for &'a str {
    #[inline]
    fn from(astr: &AinStr) -> &str {
        astr.as_str()
    }
}
impl<'a> From<&'a AinStr> for &'a AsciiStr {
    #[inline]
    fn from(astr: &AinStr) -> &AsciiStr {
        <&AinStr as Into<&[AsciiChar]>>::into(astr).into()
    }
}
impl<'a> From<&'a AsciiStr> for &'a AinStr {
    #[inline]
    fn from(astr: &AsciiStr) -> &AinStr {
        <&AsciiStr as Into<&[AsciiChar]>>::into(astr).into()
    }
}
impl<'a> From<&'a mut AinStr> for &'a mut AsciiStr {
    #[inline]
    fn from(astr: &mut AinStr) -> &mut AsciiStr {
        <&mut AinStr as Into<&mut [AsciiChar]>>::into(astr).into()
    }
}
impl<'a> From<&'a mut AsciiStr> for &'a mut AinStr {
    #[inline]
    fn from(astr: &mut AsciiStr) -> &mut AinStr {
        <&mut AsciiStr as Into<&mut [AsciiChar]>>::into(astr).into()
    }
}

macro_rules! widen_box {
    ($wider: ty) => {
        #[cfg(feature = "alloc")]
        impl From<Box<AinStr>> for Box<$wider> {
            #[inline]
            fn from(owned: Box<AinStr>) -> Box<$wider> {
                let ptr = Box::into_raw(owned) as *mut $wider;
                unsafe { Box::from_raw(ptr) }
            }
        }
    };
}
widen_box! {[AinChar]}
widen_box! {[u8]}
widen_box! {str}

// allows &AinChar to be used by generic AinString Extend and FromIterator
impl AsRef<AinStr> for AinChar {
    fn as_ref(&self) -> &AinStr {
        slice::from_ref(self).into()
    }
}

impl fmt::Display for AinStr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

impl fmt::Debug for AinStr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

macro_rules! impl_index {
    ($idx:ty) => {
        #[allow(clippy::indexing_slicing)] // In `Index`, if it's out of bounds, panic is the default
        impl Index<$idx> for AinStr {
            type Output = AinStr;

            #[inline]
            fn index(&self, index: $idx) -> &AinStr {
                self.slice[index].as_ref()
            }
        }

        #[allow(clippy::indexing_slicing)] // In `IndexMut`, if it's out of bounds, panic is the default
        impl IndexMut<$idx> for AinStr {
            #[inline]
            fn index_mut(&mut self, index: $idx) -> &mut AinStr {
                self.slice[index].as_mut()
            }
        }
    };
}

impl_index! { core::ops::Range<usize> }
impl_index! { core::ops::RangeFrom<usize> }
impl_index! { core::ops::RangeInclusive<usize> }
impl_index! { core::ops::RangeToInclusive<usize> }
impl_index! { core::range::Range<usize> }
impl_index! { core::range::RangeTo<usize> }
impl_index! { core::range::RangeFrom<usize> }
impl_index! { core::range::RangeFull }
impl_index! { core::range::RangeInclusive<usize> }
impl_index! { core::range::RangeToInclusive<usize> }

impl Index<usize> for AinStr {
    type Output = AinChar;

    #[inline]
    fn index(&self, index: usize) -> &AinChar {
        &self.slice[index]
    }
}

impl IndexMut<usize> for AinStr {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut AinChar {
        &mut self.slice[index]
    }
}

/// Produces references for compatibility with `[u8]`.
///
/// (`str` doesn't implement `IntoIterator` for its references,
///  so there is no compatibility to lose.)
impl<'a> IntoIterator for &'a AinStr {
    type Item = &'a AinChar;
    type IntoIter = CharsRef<'a>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        CharsRef(self.as_slice().iter())
    }
}

impl<'a> IntoIterator for &'a mut AinStr {
    type Item = &'a mut AinChar;
    type IntoIter = CharsMut<'a>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.chars_mut()
    }
}

/// A copying iterator over the characters of an `AinStr`.
#[derive(Clone, Debug)]
pub struct Chars<'a>(Iter<'a, AinChar>);
impl<'a> Chars<'a> {
    /// Returns the ascii string slice with the remaining characters.
    #[must_use]
    pub fn as_str(&self) -> &'a AinStr {
        self.0.as_slice().into()
    }
}
impl<'a> Iterator for Chars<'a> {
    type Item = AinChar;
    #[inline]
    fn next(&mut self) -> Option<AinChar> {
        self.0.next().copied()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
impl<'a> DoubleEndedIterator for Chars<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<AinChar> {
        self.0.next_back().copied()
    }
}
impl<'a> ExactSizeIterator for Chars<'a> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

/// A mutable iterator over the characters of an `AinStr`.
#[derive(Debug)]
pub struct CharsMut<'a>(IterMut<'a, AinChar>);
impl<'a> CharsMut<'a> {
    /// Returns the ascii string slice with the remaining characters.
    #[must_use]
    pub fn into_str(self) -> &'a mut AinStr {
        self.0.into_slice().into()
    }
}
impl<'a> Iterator for CharsMut<'a> {
    type Item = &'a mut AinChar;
    #[inline]
    fn next(&mut self) -> Option<&'a mut AinChar> {
        self.0.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
impl<'a> DoubleEndedIterator for CharsMut<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a mut AinChar> {
        self.0.next_back()
    }
}
impl<'a> ExactSizeIterator for CharsMut<'a> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

/// An immutable iterator over the characters of an `AinStr`.
#[derive(Clone, Debug)]
pub struct CharsRef<'a>(Iter<'a, AinChar>);
impl<'a> CharsRef<'a> {
    /// Returns the ascii string slice with the remaining characters.
    #[must_use]
    pub fn as_str(&self) -> &'a AinStr {
        self.0.as_slice().into()
    }
}
impl<'a> Iterator for CharsRef<'a> {
    type Item = &'a AinChar;
    #[inline]
    fn next(&mut self) -> Option<&'a AinChar> {
        self.0.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
impl<'a> DoubleEndedIterator for CharsRef<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a AinChar> {
        self.0.next_back()
    }
}

/// An iterator over parts of an `AinStr` separated by an `AinChar`.
///
/// This type is created by [`AinChar::split()`](struct.AinChar.html#method.split).
#[derive(Clone, Debug)]
struct Split<'a> {
    on: AinChar,
    ended: bool,
    chars: Chars<'a>,
}
impl<'a> Iterator for Split<'a> {
    type Item = &'a AinStr;

    fn next(&mut self) -> Option<&'a AinStr> {
        if !self.ended {
            let start: &AinStr = self.chars.as_str();
            let split_on = self.on;

            if let Some(at) = self.chars.position(|ch| ch == split_on) {
                // SAFETY: `at` is guaranteed to be in bounds, as `position` returns `Ok(0..len)`.
                Some(unsafe { start.as_slice().get_unchecked(..at).into() })
            } else {
                self.ended = true;
                Some(start)
            }
        } else {
            None
        }
    }
}
impl<'a> DoubleEndedIterator for Split<'a> {
    fn next_back(&mut self) -> Option<&'a AinStr> {
        if !self.ended {
            let start: &AinStr = self.chars.as_str();
            let split_on = self.on;

            if let Some(at) = self.chars.rposition(|ch| ch == split_on) {
                // SAFETY: `at` is guaranteed to be in bounds, as `rposition` returns `Ok(0..len)`, and slices `1..`, `2..`, etc... until `len..` inclusive, are valid.
                Some(unsafe { start.as_slice().get_unchecked(at + 1..).into() })
            } else {
                self.ended = true;
                Some(start)
            }
        } else {
            None
        }
    }
}

/// An iterator over the lines of the internal character array.
#[derive(Clone, Debug)]
struct Lines<'a> {
    string: &'a AinStr,
}
impl<'a> Iterator for Lines<'a> {
    type Item = &'a AinStr;

    fn next(&mut self) -> Option<&'a AinStr> {
        if let Some(idx) = self.string.chars().position(|chr| chr == AinChar::LineFeed) {
            // SAFETY: `idx` is guaranteed to be `1..len`, as we get it from `position` as `0..len` and make sure it's not `0`.
            let line = if idx > 0
                && *unsafe { self.string.as_slice().get_unchecked(idx - 1) }
                    == AinChar::CarriageReturn
            {
                // SAFETY: As per above, `idx` is guaranteed to be `1..len`
                unsafe { self.string.as_slice().get_unchecked(..idx - 1).into() }
            } else {
                // SAFETY: As per above, `idx` is guaranteed to be `0..len`
                unsafe { self.string.as_slice().get_unchecked(..idx).into() }
            };
            // SAFETY: As per above, `idx` is guaranteed to be `0..len`, so at the extreme, slicing `len..` is a valid empty slice.
            self.string = unsafe { self.string.as_slice().get_unchecked(idx + 1..).into() };
            Some(line)
        } else if self.string.is_empty() {
            None
        } else {
            let line = self.string;
            // SAFETY: An empty string is a valid string.
            self.string = unsafe { AinStr::from_ascii_unchecked(b"") };
            Some(line)
        }
    }
}

impl<'a> DoubleEndedIterator for Lines<'a> {
    fn next_back(&mut self) -> Option<&'a AinStr> {
        if self.string.is_empty() {
            return None;
        }

        // If we end with `LF` / `CR/LF`, remove them
        if self.string.last() == Some(AinChar::LineFeed) {
            // SAFETY: `last()` returned `Some`, so our len is at least 1.
            self.string = unsafe {
                self.string
                    .as_slice()
                    .get_unchecked(..self.string.len() - 1)
                    .into()
            };

            if self.string.last() == Some(AinChar::CarriageReturn) {
                // SAFETY: `last()` returned `Some`, so our len is at least 1.
                self.string = unsafe {
                    self.string
                        .as_slice()
                        .get_unchecked(..self.string.len() - 1)
                        .into()
                };
            }
        }

        // Get the position of the first `LF` from the end.
        let lf_rev_pos = self
            .string
            .chars()
            .rev()
            .position(|ch| ch == AinChar::LineFeed)
            .unwrap_or_else(|| self.string.len());

        // SAFETY: `lf_rev_pos` will be in range `0..=len`, so `len - lf_rev_pos`
        //         will be within `0..=len`, making it correct as a start and end
        //         point for the strings.
        let line = unsafe {
            self.string
                .as_slice()
                .get_unchecked(self.string.len() - lf_rev_pos..)
                .into()
        };
        self.string = unsafe {
            self.string
                .as_slice()
                .get_unchecked(..self.string.len() - lf_rev_pos)
                .into()
        };
        Some(line)
    }
}

/// Convert slices of bytes or [`AinChar`] to [`AinStr`].
// Could nearly replace this trait with SliceIndex, but its methods isn't even
// on a path for stabilization.
pub trait AsAinStr {
    /// Used to constrain `SliceIndex`
    #[doc(hidden)]
    type Inner;
    /// Convert a subslice to an ASCII slice.
    ///
    /// # Errors
    /// Returns `Err` if the range is out of bounds or if not all bytes in the
    /// slice are ASCII. The value in the error will be the index of the first
    /// non-ASCII byte or the end of the slice.
    ///
    /// # Examples
    /// ```
    /// use ain::AsAinStr;
    /// assert!("'zoä'".slice_ascii(..3).is_ok());
    /// assert!("'zoä'".slice_ascii(0..4).is_err());
    /// assert!("'zoä'".slice_ascii(5..=5).is_ok());
    /// assert!("'zoä'".slice_ascii(4..).is_err());
    /// assert!(b"\r\n".slice_ascii(..).is_ok());
    /// ```
    fn slice_ain<R>(&self, range: R) -> Result<&AinStr, AsAinStrError>
    where
        R: SliceIndex<[Self::Inner], Output = [Self::Inner]>;
    /// Convert to an ASCII slice.
    ///
    /// # Errors
    /// Returns `Err` if not all bytes are valid ascii values.
    ///
    /// # Example
    /// ```
    /// use ain::{AsAinStr, AinChar};
    /// assert!("ASCII".as_ascii_str().is_ok());
    /// assert!(b"\r\n".as_ascii_str().is_ok());
    /// assert!("'zoä'".as_ascii_str().is_err());
    /// assert!(b"\xff".as_ascii_str().is_err());
    /// assert!([AinChar::C][..].as_ascii_str().is_ok()); // infallible
    /// ```
    fn as_ain_str(&self) -> Result<&AinStr, AsAinStrError> {
        self.slice_ain(..)
    }
    /// Get a single ASCII character from the slice.
    ///
    /// Returns `None` if the index is out of bounds or the byte is not ASCII.
    ///
    /// # Examples
    /// ```
    /// use ain::{AsAinStr, AinChar};
    /// assert_eq!("'zoä'".get_ascii(4), None);
    /// assert_eq!("'zoä'".get_ascii(5), Some(AinChar::Apostrophe));
    /// assert_eq!("'zoä'".get_ascii(6), None);
    /// ```
    fn get_ain(&self, index: usize) -> Option<AinChar> {
        self.slice_ain(index..=index).ok().and_then(AinStr::first)
    }
    /// Convert to an ASCII slice without checking for non-ASCII characters.
    ///
    /// # Safety
    /// Calling this function when `self` contains non-ascii characters is
    /// undefined behavior.
    ///
    /// # Examples
    ///
    unsafe fn as_ain_str_unchecked(&self) -> &AinStr;
}

/// Convert mutable slices of bytes or [`AinChar`] to [`AinStr`].
pub trait AsMutAinStr: AsAinStr {
    /// Convert a subslice to an ASCII slice.
    ///
    /// # Errors
    /// This function returns `Err` if range is out of bounds, or if
    /// `self` contains non-ascii values
    fn slice_ain_mut<R>(&mut self, range: R) -> Result<&mut AinStr, AsAinStrError>
    where
        R: SliceIndex<[Self::Inner], Output = [Self::Inner]>;

    /// Convert to a mutable ASCII slice.
    ///
    /// # Errors
    /// This function returns `Err` if `self` contains non-ascii values
    fn as_mut_ain_str(&mut self) -> Result<&mut AinStr, AsAinStrError> {
        self.slice_ain_mut(..)
    }

    /// Convert to a mutable ASCII slice without checking for non-ASCII characters.
    ///
    /// # Safety
    /// Calling this function when `self` contains non-ascii characters is
    /// undefined behavior.
    unsafe fn as_mut_ain_str_unchecked(&mut self) -> &mut AinStr;
}

impl AsAsciiStr for AinStr {
    type Inner = AsciiChar;

    fn slice_ascii<R>(&self, range: R) -> Result<&AsciiStr, AsAinStrError>
    where
        R: SliceIndex<[Self::Inner], Output = [Self::Inner]>,
    {
        let ascii: &AsciiStr = self.into();
        AsAsciiStr::slice_ascii(ascii, range)
    }

    unsafe fn as_ascii_str_unchecked(&self) -> &AsciiStr {
        self.into()
    }
}

impl AsMutAsciiStr for AinStr {
    fn slice_ascii_mut<R>(&mut self, range: R) -> Result<&mut AsciiStr, AsAinStrError>
    where
        R: SliceIndex<[Self::Inner], Output = [Self::Inner]>,
    {
        let ascii: &mut AsciiStr = self.into();
        AsMutAsciiStr::slice_ascii_mut(ascii, range)
    }

    unsafe fn as_mut_ascii_str_unchecked(&mut self) -> &mut AsciiStr {
        self.into()
    }
}

impl<T: AsAsciiStr + ?Sized> AsAinStr for T {
    type Inner = <T as AsAsciiStr>::Inner;

    #[inline]
    fn slice_ain<R>(&self, range: R) -> Result<&AinStr, AsAinStrError>
    where
        R: SliceIndex<[Self::Inner], Output = [Self::Inner]>,
    {
        self.slice_ascii(range).map(Into::into)
    }

    #[inline]
    unsafe fn as_ain_str_unchecked(&self) -> &AinStr {
        // SAFETY: invariants are identical to as_ain_str_unchecked.
        unsafe { self.as_ascii_str_unchecked() }.into()
    }

    #[inline]
    fn as_ain_str(&self) -> Result<&AinStr, AsAinStrError> {
        self.as_ascii_str().map(Into::into)
    }

    #[inline]
    fn get_ain(&self, index: usize) -> Option<AinChar> {
        self.get_ascii(index).map(Into::into)
    }
}

impl<T: AsMutAsciiStr + ?Sized> AsMutAinStr for T {
    #[inline]
    fn slice_ain_mut<R>(&mut self, range: R) -> Result<&mut AinStr, AsAinStrError>
    where
        R: SliceIndex<[Self::Inner], Output = [Self::Inner]>,
    {
        self.slice_ascii_mut(range).map(Into::into)
    }

    #[inline]
    unsafe fn as_mut_ain_str_unchecked(&mut self) -> &mut AinStr {
        // SAFETY: invariants are identical to as_mut_ain_str_unchecked.
        unsafe { self.as_mut_ascii_str_unchecked() }.into()
    }

    #[inline]
    fn as_mut_ain_str(&mut self) -> Result<&mut AinStr, AsAinStrError> {
        self.as_mut_ascii_str().map(Into::into)
    }
}
