use core::borrow::{Borrow, BorrowMut};
use core::cmp::Ordering;
use core::fmt;
use core::hash::Hash;
use core::ops::{Index, IndexMut};
use core::slice::SliceIndex;

use crate::iter::{
    Chars, CharsMut, Lines, LinesMut, RSplit, RSplitMut, RSplitN, RSplitNMut, Split, SplitMut,
    SplitN, SplitNMut,
};
use crate::pattern::{DoubleEndedPattern, Pattern, SearchHolder};
use crate::validation::run_ain_validation;
use crate::{AinChar, AinValidationError};

/// [`AinStr`] represents a string slice that only contains ASCII characters, and has
/// case-insensitive comparisons.
///
/// It wraps a slice of [`AinChar`] and implements many of `str`s methods and traits.
///
/// It can be created from a literal with the [`ain!`] macro, or by a checked conversion from a
/// `str` or `[u8]`, or borrowed from an `AinString`.
///
/// For Ord, the 'case insensitive' order is to treat all characters as uppercase. This affects the
/// sort order of letters relative to the following symbols, which lie between uppercase and
/// lowercase ascii: `` [\]^_` ``
// We could derive PartialEq, Ord, and PartialOrd,  but we can better ensure autovectorization
// by writing them ourselves.
#[derive(Eq)]
#[repr(transparent)]
pub struct AinStr {
    pub(crate) slice: [AinChar],
}

// This is mostly copied from ascii::AsciiStr.
impl AinStr {
    /// Gets an empty `&'static AinStr` constant.
    #[must_use]
    #[inline]
    pub const fn empty() -> &'static AinStr {
        AinStr::from_slice(&[])
    }

    /// Gets an empty `&'static mut AinStr` constant.
    #[must_use]
    #[inline]
    pub const fn empty_mut() -> &'static mut AinStr {
        AinStr::from_slice_mut(&mut [])
    }

    /// Converts a slice of [AinChar] to an `AinStr`.
    ///
    /// # Examples
    /// ```
    /// # use ain::{AinStr, ain};
    /// let foo = AinStr::from_slice(&[ain!('f'), ain!('o'), ain!('o')]);
    /// assert_eq!(foo.as_str(), "foo");
    /// ```
    #[must_use]
    #[inline]
    pub const fn from_slice(slice: &[AinChar]) -> &Self {
        // SAFETY: We're going from one slice to another slice with identical repr and lifetime.
        unsafe { &*(slice as *const [AinChar] as *const AinStr) }
    }

    /// Converts a mutable slice of [AinChar] to an `AinStr`.
    ///
    /// # Examples
    /// ```
    /// # use ain::{ain, AinStr};
    /// let mut chars = [ain!('f'), ain!('o'), ain!('o')];
    /// let foo = AinStr::from_slice_mut(&mut chars);
    /// assert_eq!(foo.as_str(), "foo");
    /// ```
    #[must_use]
    #[inline]
    pub const fn from_slice_mut(slice: &mut [AinChar]) -> &mut Self {
        // SAFETY: We're going from one slice to another slice with identical repr and lifetime.
        unsafe { &mut *(slice as *mut [AinChar] as *mut AinStr) }
    }

    /// Convert a string slice to an AinStr. Errors if the str is not ASCII.
    #[inline]
    pub const fn from_str(s: &str) -> Result<&AinStr, AinValidationError> {
        Self::from_ascii(s.as_bytes())
    }

    /// Convert a mutable string slice to an AinStr. Errors if the str is not ASCII.
    ///
    /// Because swapping ascii characters cannot invalidate utf-8, it's completely safe to mutate
    /// the resulting AinStr without breaking the original str.
    #[inline]
    pub const fn from_str_mut(s: &mut str) -> Result<&mut AinStr, AinValidationError> {
        // SAFETY: because from_ascii_mut validates that the input is all ASCII, and an &mut AinStr
        // cannot change any character to be non-ascii, this guarantees that the original str will
        // all be valid utf-8 when the borrow ends.
        Self::from_ascii_mut(unsafe { s.as_bytes_mut() })
    }

    /// Convert a byte slice containing ASCII data to an AinStr. Errors if the bytes are not ASCII.
    #[inline]
    pub const fn from_ascii(bytes: &[u8]) -> Result<&AinStr, AinValidationError> {
        match run_ain_validation(bytes) {
            // SAFETY: we just validated that the bytes are valid.
            Ok(()) => Ok(unsafe { Self::from_ascii_unchecked(bytes) }),
            Err(e) => Err(e),
        }
    }

    /// Convert a mutable byte slice containing ASCII data to an AinStr. Errors if the bytes are not
    /// ASCII.
    #[inline]
    pub const fn from_ascii_mut(bytes: &mut [u8]) -> Result<&mut AinStr, AinValidationError> {
        match run_ain_validation(bytes) {
            // SAFETY: we just validated that the bytes are valid.
            Ok(()) => Ok(unsafe { Self::from_ascii_unchecked_mut(bytes) }),
            Err(e) => Err(e),
        }
    }

    /// Converts a slice of ascii bytes to an `AinStr` without checking for non-ASCII characters.
    ///
    /// # Safety
    ///
    /// If any of the bytes in `bytes` do not represent valid ascii characters, calling
    /// this function is undefined behavior.
    ///
    /// # Examples
    /// ```
    /// # use ain::AinStr;
    /// let foo = unsafe { AinStr::from_ascii_unchecked(b"foo") };
    /// assert_eq!(foo.as_str(), "foo");
    /// ```
    #[must_use]
    #[inline]
    pub const unsafe fn from_ascii_unchecked(bytes: &[u8]) -> &Self {
        // SAFETY: Caller guarantees all bytes in `bytes` are valid ascii characters.
        let ascii = unsafe { &*(bytes as *const [u8] as *const [AinChar]) };
        Self::from_slice(ascii)
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
    /// let mut bytes = *b"foo";
    /// let foo = unsafe { AinStr::from_ascii_unchecked_mut(&mut bytes[..]) };
    /// assert_eq!(foo.as_str(), "foo");
    /// ```
    #[must_use]
    #[inline]
    pub const unsafe fn from_ascii_unchecked_mut(bytes: &mut [u8]) -> &mut Self {
        // SAFETY: Caller guarantees all bytes in `bytes` are valid ascii characters.
        let ascii = unsafe { &mut *(bytes as *mut [u8] as *mut [AinChar]) };
        Self::from_slice_mut(ascii)
    }

    /// Converts `&self` to a `&str` slice.
    #[must_use]
    #[inline]
    pub const fn as_str(&self) -> &str {
        // SAFETY: All valid values of `AinChar` are valid ASCII, and therefore utf-8.
        unsafe { &*(self as *const AinStr as *const str) }
    }

    /// Converts `&self` into a byte slice.
    #[must_use]
    #[inline]
    pub const fn as_bytes(&self) -> &[u8] {
        // SAFETY: AinChar is a transparent wrapper around repr(u8), therefore it is valid as bytes.
        unsafe { &*(self as *const AinStr as *const [u8]) }
    }

    /// Returns the entire string as slice of `AinChar`s.
    #[must_use]
    #[inline]
    pub const fn as_slice(&self) -> &[AinChar] {
        &self.slice
    }

    /// Returns the entire string as mutable slice of `AinChar`s.
    ///
    /// Since all characters are a single byte, all safe mutations of the slice are safe for AinStr.
    #[must_use]
    #[inline]
    pub const fn as_mut_slice(&mut self) -> &mut [AinChar] {
        &mut self.slice
    }

    /// Gets a pointer to the contents of the `AinStr`.
    #[must_use]
    #[inline]
    pub const fn as_ptr(&self) -> *const AinChar {
        self.as_slice().as_ptr()
    }

    /// Returns a mutable pointer to the `AinStr`'s contents.
    #[must_use]
    #[inline]
    pub const fn as_mut_ptr(&mut self) -> *mut AinChar {
        self.as_mut_slice().as_mut_ptr()
    }

    /// Returns the number of characters / bytes in this ASCII sequence.
    ///
    /// # Examples
    /// ```
    /// # use ain::ain;
    /// let s = ain!("foo");
    /// assert_eq!(s.len(), 3);
    /// ```
    #[must_use]
    #[inline]
    pub const fn len(&self) -> usize {
        self.slice.len()
    }

    /// Returns true if the ASCII slice contains zero bytes.
    ///
    /// # Examples
    /// ```
    /// # use ain::ain;
    /// let empty = ain!("");
    /// let full = ain!("foo");
    /// assert!(empty.is_empty());
    /// assert!(!full.is_empty());
    /// ```
    #[must_use]
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.slice.is_empty()
    }

    /// Return a single index or subslice of self, without panicking if the index is out of bounds.
    #[inline]
    pub fn get<'s, I>(
        &'s self,
        idx: I,
    ) -> Option<&'s <<I as SliceIndex<[AinChar]>>::Output as AinSliceIndexOutputMap>::Output>
    where
        I: SliceIndex<[AinChar]>,
        I::Output: AinSliceIndexOutputMap + 's,
    {
        self.as_slice()
            .get(idx)
            .map(|res| res.convert_slice_index_output())
    }

    /// Return a single index or subslice of self, without panicking if the index is out of bounds.
    #[inline]
    pub fn get_mut<'s, I>(
        &'s mut self,
        idx: I,
    ) -> Option<&'s mut <<I as SliceIndex<[AinChar]>>::Output as AinSliceIndexOutputMap>::Output>
    where
        I: SliceIndex<[AinChar]>,
        I::Output: AinSliceIndexOutputMap + 's,
    {
        self.as_mut_slice()
            .get_mut(idx)
            .map(|res| res.convert_slice_index_output_mut())
    }

    /// Returns an iterator over the characters of the `AinStr`.
    #[inline]
    pub fn chars(&self) -> Chars<'_> {
        Chars {
            inner: self.slice.iter(),
        }
    }

    /// Returns an iterator over the characters of the `AinStr` which allows you to modify the
    /// value of each `AinChar`.
    #[inline]
    pub fn chars_mut(&mut self) -> CharsMut<'_> {
        CharsMut {
            inner: self.slice.iter_mut(),
        }
    }

    /// Find the index of the start of the first match of the given pattern in this `&AinStr`.
    ///
    /// [`AinChar`] and [`&AinStr`][AinStr] based patterns match case-insensitive.
    ///
    /// ```
    /// # use ain::ain;
    /// assert_eq!(ain!("Aa").find(ain!('a')), Some(0))
    /// ```
    #[must_use]
    #[inline]
    pub fn find<P: Pattern>(&self, pattern: P) -> Option<usize> {
        SearchHolder::new(pattern, self).find_impl()
    }

    /// Find the index of the start of the last match of the given pattern in this `&AinStr`.
    ///
    /// [`AinChar`] and [`&AinStr`][AinStr] based patterns match case-insensitive.
    ///
    /// ```
    /// # use ain::ain;
    /// assert_eq!(ain!("Aa").rfind(ain!('A')), Some(1))
    /// ```
    #[must_use]
    #[inline]
    pub fn rfind<P: Pattern>(&self, pattern: P) -> Option<usize> {
        SearchHolder::new(pattern, self).rfind_impl()
    }

    /// Returns true if this AinStr contains a match of the pattern.
    ///
    /// [`AinChar`] and [`&AinStr`][AinStr] based patterns match case-insensitive.
    ///
    /// ```
    /// # use ain::ain;
    /// assert_eq!(ain!("bAaB").contains(ain!('a')), true)
    /// ```
    #[must_use]
    #[inline]
    pub fn contains<P: Pattern>(&self, pattern: P) -> bool {
        self.find(pattern).is_some()
    }

    /// Split this string slice at the given index.
    #[must_use]
    #[inline]
    pub const fn split_at(&self, idx: usize) -> (&Self, &Self) {
        let (l, r) = self.slice.split_at(idx);
        (Self::from_slice(l), Self::from_slice(r))
    }

    /// Split this string slice at the given index.
    #[must_use]
    #[inline]
    pub const fn split_at_mut(&mut self, idx: usize) -> (&mut Self, &mut Self) {
        let (l, r) = self.slice.split_at_mut(idx);
        (Self::from_slice_mut(l), Self::from_slice_mut(r))
    }

    /// Returns an iterator over parts of the `AinStr` separated by a pattern.
    ///
    /// If the split pattern appears multiple times in a row, it will produce empty strings in
    /// between copies of the separator.
    ///
    /// # Examples
    /// ```
    /// # use ain::ain;
    /// let words = ain!("apple banana lemon")
    ///     .split(ain!(' '))
    ///     .map(|a| a.as_str())
    ///     .collect::<Vec<_>>();
    /// assert_eq!(words, ["apple", "banana", "lemon"]);
    ///
    /// let segments = ain!("aaaabaaaBaa")
    ///     .split(ain!('b')) // matches both `b` and `B`
    ///     .map(|s| s.as_str())
    ///     .collect::<Vec<_>>();
    /// assert_eq!(segments, ["aaaa", "aaa", "aa"]);
    ///
    /// let split_by_str = ain!("abcdabefAbghiaBjab")
    ///     .split(ain!("AB"))
    ///     .map(|s| s.as_str())
    ///     .collect::<Vec<_>>();
    /// assert_eq!(split_by_str, ["", "cd", "ef", "ghi", "j", ""]);
    ///
    /// let split_by_slice = ain!("Abc:def;ghi.jkl")
    ///     .split(&[ain!(':'), ain!(';'), ain!('.')])
    ///     .map(|s| s.as_str())
    ///     .collect::<Vec<_>>();
    /// assert_eq!(split_by_slice, ["Abc", "def", "ghi", "jkl"]);
    /// ```
    ///
    /// Splitting on the empty string produces matches before and after every character, including
    /// empty strings at the start and end:
    ///
    /// ```
    /// # use ain::ain;
    /// let split = ain!("ain")
    ///     .split(ain!(""))
    ///     .map(|c| c.as_str())
    ///     .collect::<Vec<_>>();
    /// assert_eq!(split, ["", "a", "i", "n", ""]);
    ///
    /// let split = ain!("")
    ///     .split(ain!(""))
    ///     .map(|c| c.as_str())
    ///     .collect::<Vec<_>>();
    /// assert_eq!(split, ["", ""]);
    /// ```
    #[inline]
    pub fn split<P: Pattern>(&self, on: P) -> Split<'_, P> {
        Split::new(self, on)
    }

    /// Returns an iterator over mutable parts of the `AinStr` separated by a pattern.
    ///
    /// If the split pattern appears multiple times in a row, it will produce empty strings in
    /// between copies of the separator.
    ///
    /// # Examples
    /// ```
    /// # use ain::{ain, ain_mut};
    /// ain_mut!(words, "apple banana lemon");
    /// let words = words.split_mut(ain!(' '))
    ///     .map(|word| {
    ///         word[0] = ain!('X');
    ///         word.as_str()
    ///     })
    ///     .collect::<Vec<_>>();
    /// assert_eq!(words, &["Xpple", "Xanana", "Xemon"]);
    ///
    /// ain_mut!(segments, "aaaabaaaBaa");
    /// let segments = segments.split_mut(ain!('b'))
    ///     .map(|word| {
    ///         word[0] = ain!('X');
    ///         word.as_str()
    ///     })
    ///     .collect::<Vec<_>>();
    /// assert_eq!(segments, &["Xaaa", "Xaa", "Xa"]);
    /// ```
    #[inline]
    pub fn split_mut<P: Pattern>(&mut self, on: P) -> SplitMut<'_, P> {
        SplitMut::new(self, on)
    }

    /// Returns an iterator over parts of the `AinStr` separated by a pattern, starting from the
    /// back.
    ///
    /// If the split pattern appears multiple times in a row, it will produce empty strings in
    /// between copies of the separator.
    ///
    /// # Examples
    /// ```
    /// # use ain::ain;
    ///
    /// let words = ain!("apple banana lemon")
    ///     .rsplit(ain!(' '))
    ///     .map(|a| a.as_str())
    ///     .collect::<Vec<_>>();
    /// assert_eq!(words, ["lemon", "banana", "apple"]);
    ///
    /// let segments = ain!("aaaabaaaBaa")
    ///     .rsplit(ain!('b')) // matches both `b` and `B`
    ///     .map(|s| s.as_str())
    ///     .collect::<Vec<_>>();
    /// assert_eq!(segments, ["aa", "aaa", "aaaa"]);
    ///
    /// let split_by_str = ain!("abcdabefAbghiaBjab")
    ///     .rsplit(ain!("AB"))
    ///     .map(|s| s.as_str())
    ///     .collect::<Vec<_>>();
    /// assert_eq!(split_by_str, ["", "j", "ghi", "ef", "cd", ""]);
    ///
    /// let split_by_slice = ain!("Abc:def;ghi.jkl")
    ///     .rsplit(&[ain!(':'), ain!(';'), ain!('.')])
    ///     .map(|s| s.as_str())
    ///     .collect::<Vec<_>>();
    /// assert_eq!(split_by_slice, ["jkl", "ghi", "def", "Abc"]);
    /// ```
    ///
    /// Splitting on the empty string produces matches before and after every character, including
    /// empty strings at the start and end:
    ///
    /// ```
    /// # use ain::ain;
    /// let split = ain!("ain")
    ///     .rsplit(ain!(""))
    ///     .map(|c| c.as_str())
    ///     .collect::<Vec<_>>();
    /// assert_eq!(split, ["", "n", "i", "a", ""]);
    ///
    /// let split = ain!("")
    ///     .rsplit(ain!(""))
    ///     .map(|c| c.as_str())
    ///     .collect::<Vec<_>>();
    /// assert_eq!(split, ["", ""]);
    /// ```
    #[inline]
    pub fn rsplit<P: Pattern>(&self, on: P) -> RSplit<'_, P> {
        RSplit::new(self, on)
    }

    /// Returns an iterator over mutable parts of the `AinStr` separated by a pattern.
    ///
    /// If the split pattern appears multiple times in a row, it will produce empty strings in
    /// between copies of the separator.
    ///
    /// # Examples
    /// ```
    /// # use ain::{ain, ain_mut};
    /// ain_mut!(words, "apple banana lemon");
    /// let words = words.rsplit_mut(ain!(' '))
    ///     .map(|word| {
    ///         word[0] = ain!('X');
    ///         word.as_str()
    ///     })
    ///     .collect::<Vec<_>>();
    /// assert_eq!(words, &["Xemon", "Xanana", "Xpple"]);
    ///
    /// ain_mut!(segments, "aaaabaaaBaa");
    /// let segments = segments.rsplit_mut(ain!('b'))
    ///     .map(|word| {
    ///         word[0] = ain!('X');
    ///         word.as_str()
    ///     })
    ///     .collect::<Vec<_>>();
    /// assert_eq!(segments, &["Xa", "Xaa", "Xaaa"]);
    /// ```
    #[inline]
    pub fn rsplit_mut<P: Pattern>(&mut self, on: P) -> RSplitMut<'_, P> {
        RSplitMut::new(self, on)
    }

    /// Returns an iterator over parts of the `AinStr` separated by a pattern, returning at most `n`
    /// items.
    ///
    /// If `n` substrings are returned, the last substring (the `n`th substring) will contain the
    /// remainder of the string.
    ///
    /// If the split pattern appears multiple times in a row, it will produce empty strings in
    /// between copies of the separator.
    ///
    /// # Examples
    /// ```
    /// # use ain::ain;
    /// let words = ain!("apple banana lemon")
    ///     .splitn(2, ain!(' '))
    ///     .map(|a| a.as_str())
    ///     .collect::<Vec<_>>();
    /// assert_eq!(words, ["apple", "banana lemon"]);
    ///
    /// let segments = ain!("aaaabaaaBaa")
    ///     .splitn(2, ain!('b')) // matches both `b` and `B`
    ///     .map(|s| s.as_str())
    ///     .collect::<Vec<_>>();
    /// assert_eq!(segments, ["aaaa", "aaaBaa"]);
    ///
    /// let split_by_str = ain!("abcdabefAbghiaBjab")
    ///     .splitn(4, ain!("AB"))
    ///     .map(|s| s.as_str())
    ///     .collect::<Vec<_>>();
    /// assert_eq!(split_by_str, ["", "cd", "ef", "ghiaBjab"]);
    /// ```
    #[inline]
    pub fn splitn<P: Pattern>(&self, n: usize, on: P) -> SplitN<'_, P> {
        SplitN::new(self, n, on)
    }

    /// Returns an iterator over mutable parts of the `AinStr` separated by a pattern, returning at
    /// most `n` items.
    ///
    /// If `n` substrings are returned, the last substring (the `n`th substring) will contain the
    /// remainder of the string.
    ///
    ///
    /// If the split pattern appears multiple times in a row, it will produce empty strings in
    /// between copies of the separator.
    ///
    /// # Examples
    /// ```
    /// # use ain::{ain, ain_mut};
    /// ain_mut!(words, "apple banana lemon");
    /// let words = words.splitn_mut(2, ain!(' '))
    ///     .map(|word| {
    ///         word[0] = ain!('X');
    ///         word.as_str()
    ///     })
    ///     .collect::<Vec<_>>();
    /// assert_eq!(words, &["Xpple", "Xanana lemon"]);
    ///
    /// ain_mut!(segments, "aaaabaaaBaa");
    /// let segments = segments.splitn_mut(2, ain!('b'))
    ///     .map(|word| {
    ///         word[0] = ain!('X');
    ///         word.as_str()
    ///     })
    ///     .collect::<Vec<_>>();
    /// assert_eq!(segments, &["Xaaa", "XaaBaa"]);
    /// ```
    #[inline]
    pub fn splitn_mut<P: Pattern>(&mut self, n: usize, on: P) -> SplitNMut<'_, P> {
        SplitNMut::new(self, n, on)
    }

    /// Returns an iterator over parts of the `AinStr` separated by a pattern, returning at most `n`
    /// items, starting from the end.
    ///
    /// If `n` substrings are returned, the last substring (the `n`th substring) will contain the
    /// remainder of the string.
    ///
    /// If the split pattern appears multiple times in a row, it will produce empty strings in
    /// between copies of the separator.
    ///
    /// # Examples
    /// ```
    /// # use ain::ain;
    ///
    /// let words = ain!("apple banana lemon")
    ///     .rsplitn(2, ain!(' '))
    ///     .map(|a| a.as_str())
    ///     .collect::<Vec<_>>();
    /// assert_eq!(words, ["lemon", "apple banana"]);
    ///
    /// let segments = ain!("aaaabaaaBaa")
    ///     .rsplitn(2, ain!('b')) // matches both `b` and `B`
    ///     .map(|s| s.as_str())
    ///     .collect::<Vec<_>>();
    /// assert_eq!(segments, ["aa", "aaaabaaa"]);
    /// ```
    #[inline]
    pub fn rsplitn<P: Pattern>(&self, n: usize, on: P) -> RSplitN<'_, P> {
        RSplitN::new(self, n, on)
    }

    /// Returns an iterator over mutable parts of the `AinStr` separated by a pattern, returning at
    /// most `n` items, starting from the end.
    ///
    /// If `n` substrings are returned, the last substring (the `n`th substring) will contain the
    /// remainder of the string.
    ///
    /// If the split pattern appears multiple times in a row, it will produce empty strings in
    /// between copies of the separator.
    ///
    /// # Examples
    /// ```
    /// # use ain::{ain, ain_mut};
    /// ain_mut!(words, "apple banana lemon");
    /// let words = words.rsplitn_mut(2, ain!(' '))
    ///     .map(|word| {
    ///         word[0] = ain!('X');
    ///         word.as_str()
    ///     })
    ///     .collect::<Vec<_>>();
    /// assert_eq!(words, &["Xemon", "Xpple banana"]);
    ///
    /// ain_mut!(segments, "aaaabaaaBaa");
    /// let segments = segments.rsplitn_mut(2, ain!('b'))
    ///     .map(|word| {
    ///         word[0] = ain!('X');
    ///         word.as_str()
    ///     })
    ///     .collect::<Vec<_>>();
    /// assert_eq!(segments, &["Xa", "Xaaabaaa"]);
    /// ```
    #[inline]
    pub fn rsplitn_mut<P: Pattern>(&mut self, n: usize, on: P) -> RSplitNMut<'_, P> {
        RSplitNMut::new(self, n, on)
    }

    /// Returns an iterator over the lines of the `AinStr`, which are themselves `AinStr`s.
    ///
    /// Lines are ended with either `LineFeed` (`\n`), or `CarriageReturn` then `LineFeed` (`\r\n`).
    ///
    /// The final line ending is optional. A string that ends with a final line ending will return
    /// the same lines as an otherwise identical string without a final line ending.
    ///
    /// ```
    /// # use ain::ain;
    /// let str = ain!("Some text with\nMixed Line Endings\r\nAnd Trailing Line Ending\n");
    /// let lines = str.lines().collect::<Vec<_>>();
    /// assert_eq!(
    ///     lines,
    ///     &[
    ///         ain!("Some text with"),
    ///         ain!("Mixed line endings"),
    ///         ain!("and trailing line ending"),
    ///     ],
    /// );
    ///
    /// let str = ain!("");
    /// let lines = str.lines().collect::<Vec<_>>();
    /// assert!(lines.is_empty());
    /// ```
    #[inline]
    pub fn lines(&self) -> Lines<'_> {
        Lines::new(self)
    }

    /// Returns an iterator over mutable lines of the `AinStr`, which are themselves `AinStr`s.
    ///
    /// Lines are ended with either `LineFeed` (`\n`), or `CarriageReturn` then `LineFeed` (`\r\n`).
    ///
    /// The final line ending is optional. A string that ends with a final line ending will return
    /// the same lines as an otherwise identical string without a final line ending.
    ///
    /// ```
    /// # use ain::{ain, ain_mut};
    /// ain_mut!(str, "Some text with\nMixed Line Endings\r\nAnd Trailing Line Ending\n");
    /// let lines = str.lines_mut()
    ///     .map(|line| {
    ///         line[0] = ain!('X');
    ///         &*line
    ///     })
    ///     .collect::<Vec<_>>();
    /// assert_eq!(
    ///     lines,
    ///     &[
    ///         ain!("Xome text with"),
    ///         ain!("xixed line endings"),
    ///         ain!("xnd trailing line ending"),
    ///     ],
    /// );
    ///
    /// ain_mut!(str, "");
    /// let lines = str.lines_mut().collect::<Vec<_>>();
    /// assert!(lines.is_empty());
    /// ```
    #[inline]
    pub fn lines_mut(&mut self) -> LinesMut<'_> {
        LinesMut::new(self)
    }

    /// Returns an ASCII string slice with leading and trailing whitespace removed.
    ///
    /// Note: This uses [AinChar::is_whitespace], which excludes Vertical Tab.
    ///
    /// # Examples
    /// ```
    /// # use ain::ain;
    /// let example = ain!("  \twhite \tspace  \t");
    /// assert_eq!(example.trim().as_str(), "white \tspace");
    /// ```
    #[must_use]
    #[inline]
    pub const fn trim(&self) -> &Self {
        self.trim_start().trim_end()
    }

    /// Returns an ASCII string slice with leading whitespace removed.
    ///
    /// Note: This uses [AinChar::is_whitespace], which excludes Vertical Tab.
    ///
    /// # Examples
    /// ```
    /// # use ain::ain;
    /// let example = ain!("  \twhite \tspace  \t");
    /// assert_eq!(example.trim_start().as_str(), "white \tspace  \t");
    /// ```
    #[must_use]
    #[inline]
    pub const fn trim_start(&self) -> &Self {
        let mut chars = self.as_slice();
        // Note: A pattern matching based approach (instead of indexing) allows making the function
        // const.
        while let [first, rest @ ..] = chars {
            if first.is_whitespace() {
                chars = rest;
            } else {
                break;
            }
        }
        Self::from_slice(chars)
    }

    /// Returns an ASCII string slice with trailing whitespace removed.
    ///
    /// Note: This uses [AinChar::is_whitespace], which excludes Vertical Tab.
    ///
    /// # Examples
    /// ```
    /// # use ain::ain;
    /// let example = ain!("  \twhite \tspace  \t");
    /// assert_eq!(example.trim_end().as_str(), "  \twhite \tspace");
    /// ```
    #[must_use]
    #[inline]
    pub const fn trim_end(&self) -> &Self {
        let mut chars = self.as_slice();
        // Note: A pattern matching based approach (instead of indexing) allows making the function
        // const.
        while let [rest @ .., last] = chars {
            if last.is_whitespace() {
                chars = rest;
            } else {
                break;
            }
        }
        Self::from_slice(chars)
    }

    /// If the given pattern exists at the start of self, returns self with one copy of the pattern
    /// removed, otherwise returns none.
    ///
    /// ```
    /// # use ain::ain;
    /// let s = ain!("aaabbb").strip_prefix(ain!('a'));
    /// assert_eq!(s, Some(ain!("aabbb")));
    ///
    /// let s = ain!("aaabbb").strip_prefix(ain!('c'));
    /// assert_eq!(s, None);
    /// ```
    #[must_use]
    #[inline]
    pub fn strip_prefix<P: Pattern>(&self, mut pattern: P) -> Option<&Self> {
        match pattern.start_match_split_point(self) {
            Some(prefix_end) => Some(&self[prefix_end..]),
            None => None,
        }
    }

    /// If the given pattern exists at the start of self, returns self with one copy of the pattern
    /// removed, otherwise returns none, using a mutable reference.
    ///
    /// ```
    /// # use ain::{ain, ain_mut};
    /// ain_mut!(s, "aaabbb");
    /// let s = s.strip_prefix_mut(ain!('a'));
    /// assert_eq!(s.as_deref(), Some(ain!("aabbb")));
    ///
    /// ain_mut!(s, "aaabbb");
    /// let s = s.strip_prefix_mut(ain!('c'));
    /// assert_eq!(s, None);
    /// ```
    #[must_use]
    #[inline]
    pub fn strip_prefix_mut<P: Pattern>(&mut self, mut pattern: P) -> Option<&mut Self> {
        match pattern.start_match_split_point(self) {
            Some(prefix_end) => Some(&mut self[prefix_end..]),
            None => None,
        }
    }

    /// Removes one copy of the given pattern from the start of self if it is present, otherwise
    /// returns self unchanged.
    ///
    /// ```
    /// # use ain::ain;
    /// let s = ain!("aaabbb").trim_prefix(ain!('a'));
    /// assert_eq!(s, ain!("aabbb"));
    ///
    /// let s = ain!("aaabbb").trim_prefix(ain!('c'));
    /// assert_eq!(s, ain!("aaabbb"));
    /// ```
    #[must_use]
    #[inline]
    pub fn trim_prefix<P: Pattern>(&self, pattern: P) -> &Self {
        self.strip_prefix(pattern).unwrap_or(self)
    }

    /// Removes one copy of the given pattern from the start of self if it is present, otherwise
    /// returns self unchanged, using a mutable reference.
    /// ```
    /// # use ain::{ain, ain_mut};
    /// ain_mut!(s, "aaabbb");
    /// let s = s.trim_prefix_mut(ain!('a'));
    /// assert_eq!(s, ain!("aabbb"));
    ///
    /// ain_mut!(s, "aaabbb");
    /// let s = s.trim_prefix_mut(ain!('c'));
    /// assert_eq!(s, ain!("aaabbb"));
    /// ```
    #[must_use]
    #[inline]
    pub fn trim_prefix_mut<P: Pattern>(&mut self, mut pattern: P) -> &mut Self {
        match pattern.start_match_split_point(self) {
            Some(prefix_end) => &mut self[prefix_end..],
            None => self,
        }
    }

    /// Removes all matches of the given pattern from the start of self.
    ///
    /// ```
    /// # use ain::ain;
    /// let s = ain!("aaabbb").trim_start_matches(ain!('a'));
    /// assert_eq!(s, ain!("bbb"));
    ///
    /// let s = ain!("aaabbb").trim_start_matches(ain!('b'));
    /// assert_eq!(s, ain!("aaabbb"));
    ///
    /// let s = ain!("aaabbb").trim_start_matches(ain!("aa"));
    /// assert_eq!(s, ain!("abbb"));
    ///
    /// let s = ain!("aaaaaa").trim_start_matches(ain!('a'));
    /// assert_eq!(s, ain!(""));
    ///
    /// let s = ain!("aaaaaa").trim_start_matches(ain!(""));
    /// assert_eq!(s, ain!("aaaaaa"));
    /// ```
    #[must_use]
    #[inline]
    pub fn trim_start_matches<P: Pattern>(&self, mut pattern: P) -> &Self {
        let start_matches_end = pattern.start_matches_split_point(self);
        &self[start_matches_end..]
    }

    /// Removes all matches of the given pattern from the start of self, keeping self mutable.
    /// ```
    /// # use ain::{ain, ain_mut};
    /// ain_mut!(s, "aaabbb");
    /// let s = s.trim_start_matches_mut(ain!('a'));
    /// assert_eq!(s, ain!("bbb"));
    ///
    /// ain_mut!(s, "aaabbb");
    /// let s = s.trim_start_matches_mut(ain!('b'));
    /// assert_eq!(s, ain!("aaabbb"));
    ///
    /// ain_mut!(s, "aaaaaa");
    /// let s = s.trim_start_matches_mut(ain!('a'));
    /// assert_eq!(s, ain!(""));
    /// ```
    #[must_use]
    #[inline]
    pub fn trim_start_matches_mut<P: Pattern>(&mut self, mut pattern: P) -> &mut Self {
        let start_matches_end = pattern.start_matches_split_point(self);
        &mut self[start_matches_end..]
    }

    /// If the given pattern exists on the end of self, returns self with one copy of the pattern
    /// removed, otherwise returns none.
    ///
    /// ```
    /// # use ain::ain;
    /// let s = ain!("aaabbb").strip_suffix(ain!('b'));
    /// assert_eq!(s, Some(ain!("aaabb")));
    ///
    /// let s = ain!("aaabbb").strip_suffix(ain!('c'));
    /// assert_eq!(s, None);
    /// ```
    #[must_use]
    #[inline]
    pub fn strip_suffix<P: Pattern>(&self, mut pattern: P) -> Option<&Self> {
        match pattern.end_match_split_point(self) {
            Some(suffix_start) => Some(&self[..suffix_start]),
            None => None,
        }
    }

    /// If the given pattern exists on the end of self, returns self with one copy of the pattern
    /// removed, otherwise returns none, using a mutable reference.
    ///
    /// ```
    /// # use ain::{ain, ain_mut};
    /// ain_mut!(s, "aaabbb");
    /// let s = s.strip_suffix_mut(ain!('b'));
    /// assert_eq!(s.as_deref(), Some(ain!("aaabb")));
    ///
    /// ain_mut!(s, "aaabbb");
    /// let s = s.strip_suffix_mut(ain!('c'));
    /// assert_eq!(s, None);
    /// ```
    #[must_use]
    #[inline]
    pub fn strip_suffix_mut<P: Pattern>(&mut self, mut pattern: P) -> Option<&mut Self> {
        match pattern.end_match_split_point(self) {
            Some(suffix_start) => Some(&mut self[..suffix_start]),
            None => None,
        }
    }

    /// Removes one copy of the given pattern from the end of self if it is present, otherwise
    /// returns self unchanged.
    ///
    /// ```
    /// # use ain::ain;
    /// let s = ain!("aaabbb").trim_suffix(ain!('b'));
    /// assert_eq!(s, ain!("aaabb"));
    ///
    /// let s = ain!("aaabbb").trim_suffix(ain!('c'));
    /// assert_eq!(s, ain!("aaabbb"));
    /// ```
    #[must_use]
    #[inline]
    pub fn trim_suffix<P: Pattern>(&self, pattern: P) -> &Self {
        self.strip_suffix(pattern).unwrap_or(self)
    }

    /// Removes one copy of the given pattern from the end of self if it is present, otherwise
    /// returns self unchanged, using a mutable reference.
    ///
    /// ```
    /// # use ain::{ain, ain_mut};
    /// ain_mut!(s, "aaabbb");
    /// let s = s.trim_suffix_mut(ain!('b'));
    /// assert_eq!(s, ain!("aaabb"));
    ///
    /// ain_mut!(s, "aaabbb");
    /// let s = s.trim_suffix_mut(ain!('c'));
    /// assert_eq!(s, ain!("aaabbb"));
    /// ```
    #[must_use]
    #[inline]
    pub fn trim_suffix_mut<P: Pattern>(&mut self, mut pattern: P) -> &mut Self {
        match pattern.end_match_split_point(self) {
            Some(suffix_start) => &mut self[..suffix_start],
            None => self,
        }
    }

    /// Removes all matches of the given pattern from the end of self.
    ///
    /// ```
    /// # use ain::ain;
    /// let s = ain!("aaabbb").trim_end_matches(ain!('b'));
    /// assert_eq!(s, ain!("aaa"));
    ///
    /// let s = ain!("aaabbb").trim_end_matches(ain!('a'));
    /// assert_eq!(s, ain!("aaabbb"));
    ///
    /// let s = ain!("aaabbb").trim_end_matches(ain!("bb"));
    /// assert_eq!(s, ain!("aaab"));
    ///
    /// let s = ain!("aaaaaa").trim_end_matches(ain!('A'));
    /// assert_eq!(s, ain!(""));
    ///
    /// let s = ain!("aaaaaa").trim_end_matches(ain!(""));
    /// assert_eq!(s, ain!("aaaaaa"));
    /// ```
    #[must_use]
    #[inline]
    pub fn trim_end_matches<P: Pattern>(&self, mut pattern: P) -> &Self {
        let end_matches_start = pattern.end_matches_split_point(self);
        &self[..end_matches_start]
    }

    /// Removes all matches of the given pattern from the start of self, keeping self mutable.
    ///
    /// ```
    /// # use ain::{ain, ain_mut};
    /// ain_mut!(s, "aaabbb");
    /// let s = s.trim_end_matches_mut(ain!('b'));
    /// assert_eq!(s, ain!("aaa"));
    ///
    /// ain_mut!(s, "aaabbb");
    /// let s = s.trim_end_matches_mut(ain!('a'));
    /// assert_eq!(s, ain!("aaabbb"));
    ///
    /// ain_mut!(s, "aaaAaA");
    /// let s = s.trim_end_matches_mut(ain!('A'));
    /// assert_eq!(s, ain!(""));
    /// ```
    #[must_use]
    #[inline]
    pub fn trim_end_matches_mut<P: Pattern>(&mut self, mut pattern: P) -> &mut Self {
        let end_matches_start = pattern.end_matches_split_point(self);
        &mut self[..end_matches_start]
    }

    /// Removes all matches of the given pattern from both ends of self.
    ///
    /// ```
    /// # use ain::ain;
    /// let s = ain!("aaabbbaaa").trim_matches(ain!('a'));
    /// assert_eq!(s, ain!("bbb"));
    ///
    /// let s = ain!("aaabbbaaa").trim_matches(ain!('b'));
    /// assert_eq!(s, ain!("aaabbbaaa"));
    ///
    /// let s = ain!("aaaaaa").trim_matches(ain!('A'));
    /// assert_eq!(s, ain!(""));
    /// ```
    #[must_use]
    #[inline]
    pub fn trim_matches<P: DoubleEndedPattern>(&self, mut pattern: P) -> &Self {
        let start_split = pattern.start_matches_split_point(self);
        let start_trimmed = &self[start_split..];
        let end_split = pattern.end_matches_split_point(start_trimmed);
        &start_trimmed[..end_split]
    }

    /// Removes all matches of the given pattern from both ends of self.
    ///
    /// ```
    /// # use ain::{ain, ain_mut};
    /// ain_mut!(s, "aaabbbaaa");
    /// let s = s.trim_matches_mut(ain!('a'));
    /// assert_eq!(s, ain!("bbb"));
    ///
    /// ain_mut!(s, "aaabbbaaa");
    /// let s = s.trim_matches_mut(ain!('b'));
    /// assert_eq!(s, ain!("aaabbbaaa"));
    ///
    /// ain_mut!(s, "AaAAaA");
    /// let s = s.trim_matches_mut(ain!('a'));
    /// assert_eq!(s, ain!(""));
    /// ```
    #[must_use]
    #[inline]
    pub fn trim_matches_mut<P: DoubleEndedPattern>(&mut self, mut pattern: P) -> &mut Self {
        let start_split = pattern.start_matches_split_point(self);
        let start_trimmed = &mut self[start_split..];
        let end_split = pattern.end_matches_split_point(start_trimmed);
        &mut start_trimmed[..end_split]
    }

    /// Replaces lowercase letters with their uppercase equivalent.
    ///
    /// Since the string is case insensitive, this does not change the equality comparions or hash
    /// of the string.
    #[inline]
    pub const fn make_uppercase(&mut self) {
        let mut chars = self.as_mut_slice();
        // For loops are not const yet, so we either need to use an index loop or this
        // pattern-matching hack.
        while let [first, rest @ ..] = chars {
            first.make_uppercase();
            chars = rest;
        }
    }

    /// Replaces uppercase letters with their lowercase equivalent.
    ///
    /// Since the string is case insensitive, this does not change the equality comparions or hash
    /// of the string.
    #[inline]
    pub const fn make_lowercase(&mut self) {
        let mut chars = self.as_mut_slice();
        // For loops are not const yet, so we either need to use an index loop or this
        // pattern-matching hack.
        while let [first, rest @ ..] = chars {
            first.make_lowercase();
            chars = rest;
        }
    }

    /// Returns the first character if the string is not empty.
    #[must_use]
    #[inline]
    pub const fn first(&self) -> Option<AinChar> {
        self.slice.first().copied()
    }

    /// Returns the last character if the string is not empty.
    #[must_use]
    #[inline]
    pub const fn last(&self) -> Option<AinChar> {
        self.slice.last().copied()
    }
}

impl fmt::Debug for AinStr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl fmt::Display for AinStr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

impl Hash for AinStr {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        // Just delegate to the slice. This is identical to the derived hash impl, but makes clippy
        // happy because we override PartialEq.
        self.slice.hash(state)
    }
}

impl PartialEq for AinStr {
    #[inline]
    fn eq(&self, other: &AinStr) -> bool {
        self.as_bytes().eq_ignore_ascii_case(other.as_bytes())
    }
}

impl PartialEq<&mut AinStr> for AinStr {
    #[inline]
    fn eq(&self, other: &&mut AinStr) -> bool {
        PartialEq::eq(self, *other)
    }
}

impl PartialEq<&AinStr> for AinStr {
    #[inline]
    fn eq(&self, other: &&AinStr) -> bool {
        PartialEq::eq(self, *other)
    }
}

impl PartialEq<AinStr> for &AinStr {
    #[inline]
    fn eq(&self, other: &AinStr) -> bool {
        PartialEq::eq(*self, other)
    }
}

impl PartialEq<AinStr> for &mut AinStr {
    #[inline]
    fn eq(&self, other: &AinStr) -> bool {
        PartialEq::eq(*self, other)
    }
}

impl PartialEq<[AinChar]> for AinStr {
    #[inline]
    fn eq(&self, other: &[AinChar]) -> bool {
        PartialEq::eq(self, AinStr::from_slice(other))
    }
}

impl PartialEq<AinStr> for [AinChar] {
    #[inline]
    fn eq(&self, other: &AinStr) -> bool {
        PartialEq::eq(AinStr::from_slice(self), other)
    }
}

impl<const N: usize> PartialEq<[AinChar; N]> for AinStr {
    #[inline]
    fn eq(&self, other: &[AinChar; N]) -> bool {
        PartialEq::eq(self, AinStr::from_slice(other))
    }
}

impl<const N: usize> PartialEq<AinStr> for [AinChar; N] {
    #[inline]
    fn eq(&self, other: &AinStr) -> bool {
        PartialEq::eq(AinStr::from_slice(self), other)
    }
}

impl PartialOrd for AinStr {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(Ord::cmp(self, other))
    }
}

impl Ord for AinStr {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        // This implementation is based on the standard library handling of eq_ignore_ascii_case,
        // adapted for ord handling.
        #[cfg(any(
            all(target_arch = "x86_64", target_feature = "sse2"),
            all(target_arch = "aarch64", target_feature = "neon"),
        ))]
        {
            const CHUNK_SIZE: usize = 16;
            if self.len() >= CHUNK_SIZE && other.len() >= CHUNK_SIZE {
                return cmp_to_uppercase_chunks::<CHUNK_SIZE>(self.as_slice(), other.as_slice());
            }
        }

        // For the slow path, we can just compare slices, which uses the Ord defined on AinChar.
        <[AinChar]>::cmp(self.as_slice(), other.as_slice())
    }
}

/// Optimized version of Ord for AinStr to process chunks at a time.
///
/// Platforms that have SIMD instructions may benefit from this over a naive slice check.
///
/// # Invariants
///
/// The caller must guarantee that both slices are at least `CHUNK_SIZE` len.
#[inline]
fn cmp_to_uppercase_chunks<const CHUNK_SIZE: usize>(lhs: &[AinChar], rhs: &[AinChar]) -> Ordering {
    let overlap = lhs.len().min(rhs.len());
    let lhs_overlap = &lhs[..overlap];
    let rhs_overlap = &rhs[..overlap];
    // We only chunk up the overlapping portion of the two inputs. Rem is only used to find out the
    // len of the tail of the overlapping portion, so that we can step back and handle that as a
    // single chunk. We'll only worry about non-overlapping portions if the overlapping portions are
    // identical.
    let (lhs_chunks, lhs_rem) = lhs_overlap.as_chunks::<CHUNK_SIZE>();
    let (rhs_chunks, rhs_rem) = rhs_overlap.as_chunks::<CHUNK_SIZE>();

    // We copy this from the std lib eq_ignor_ascii_case_chunks to avoid going through the whole
    // eq_ignore_ascii_case chunking machinery to get to just this part.
    #[inline(always)]
    fn eq_ignore_ascii_case_inner<const L: usize>(lhs: &[AinChar; L], rhs: &[AinChar; L]) -> bool {
        // Branchless check to encourage auto-vectorization
        let mut equal_ascii = true;
        let mut j = 0;
        while j < L {
            equal_ascii &= lhs[j] == rhs[j];
            j += 1;
        }
        equal_ascii
    }

    for (lhs_chunk, rhs_chunk) in lhs_chunks.iter().zip(rhs_chunks) {
        if !eq_ignore_ascii_case_inner(lhs_chunk, rhs_chunk) {
            // If we've found a block with a difference, we do a branching linear scan to find out
            // where it is, by falling back to the implementation of cmp for the array type.
            return <[AinChar; CHUNK_SIZE]>::cmp(lhs_chunk, rhs_chunk);
        }
    }

    // If there are no differences so far and the overlapping portion has a non-chunk-sized tail,
    // get a chunk from tail of the overlapping portion and do a vectorized comparison on that.
    if !lhs_rem.is_empty() {
        // Since both lhs_overlap and rhs_overlap are the same len and both are longer than
        // CHUNK_SIZE, this if let will always match.
        if let (Some(lhs_tail), Some(rhs_tail)) = (
            lhs_overlap.last_chunk::<CHUNK_SIZE>(),
            rhs_overlap.last_chunk::<CHUNK_SIZE>(),
        ) && !eq_ignore_ascii_case_inner(lhs_tail, rhs_tail)
        {
            // There was a mismatch within the tail portion of the overlapping section of the
            // two slices, so check the tails to find the mismatch position.
            //
            // Note that we go back to using lhs_rem and rhs_rem here, because after the check
            // in the main chunks loop, we already know that the prefix of the tails is equal,
            // so we only need to check the actual rem portion. The _tail portions were just so
            // we could vectorize this last eq_ignor_ascii_case_inner check.
            return <[AinChar]>::cmp(lhs_rem, rhs_rem);
        }
    }

    // The overlapping portions of the two slices are identical, so now we just compare by len.
    lhs.len().cmp(&rhs.len())
}

impl PartialOrd<[AinChar]> for AinStr {
    #[inline]
    fn partial_cmp(&self, other: &[AinChar]) -> Option<Ordering> {
        PartialOrd::partial_cmp(self, AinStr::from_slice(other))
    }
}

impl PartialOrd<AinStr> for [AinChar] {
    #[inline]
    fn partial_cmp(&self, other: &AinStr) -> Option<Ordering> {
        PartialOrd::partial_cmp(AinStr::from_slice(self), other)
    }
}

impl<const N: usize> PartialOrd<[AinChar; N]> for AinStr {
    #[inline]
    fn partial_cmp(&self, other: &[AinChar; N]) -> Option<Ordering> {
        PartialOrd::partial_cmp(self, AinStr::from_slice(other))
    }
}

impl<const N: usize> PartialOrd<AinStr> for [AinChar; N] {
    #[inline]
    fn partial_cmp(&self, other: &AinStr) -> Option<Ordering> {
        PartialOrd::partial_cmp(AinStr::from_slice(self), other)
    }
}

trait Sealed {}

/// Trait to help map from slice-index types to the correct output types of [AinStr] indexing.
#[allow(private_bounds)]
pub trait AinSliceIndexOutputMap: Sealed {
    type Output: ?Sized + 'static;

    fn convert_slice_index_output(&self) -> &Self::Output;

    fn convert_slice_index_output_mut(&mut self) -> &mut Self::Output;
}

impl Sealed for [AinChar] {}
impl AinSliceIndexOutputMap for [AinChar] {
    type Output = AinStr;

    #[inline]
    fn convert_slice_index_output(&self) -> &Self::Output {
        AinStr::from_slice(self)
    }

    #[inline]
    fn convert_slice_index_output_mut(&mut self) -> &mut Self::Output {
        AinStr::from_slice_mut(self)
    }
}

impl Sealed for AinChar {}
impl AinSliceIndexOutputMap for AinChar {
    type Output = Self;

    #[inline]
    fn convert_slice_index_output(&self) -> &Self::Output {
        self
    }

    #[inline]
    fn convert_slice_index_output_mut(&mut self) -> &mut Self::Output {
        self
    }
}

impl<S> Index<S> for AinStr
where
    S: SliceIndex<[AinChar]>,
    S::Output: AinSliceIndexOutputMap + 'static,
{
    type Output = <S::Output as AinSliceIndexOutputMap>::Output;

    #[inline]
    fn index(&self, index: S) -> &Self::Output {
        self.slice[index].convert_slice_index_output()
    }
}

impl<S> IndexMut<S> for AinStr
where
    S: SliceIndex<[AinChar]>,
    S::Output: AinSliceIndexOutputMap + 'static,
{
    #[inline]
    fn index_mut(&mut self, index: S) -> &mut Self::Output {
        self.slice[index].convert_slice_index_output_mut()
    }
}

impl AsRef<AinStr> for AinStr {
    #[inline]
    fn as_ref(&self) -> &AinStr {
        self
    }
}

impl AsRef<[u8]> for AinStr {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}
from_ref_from_as_ref!(const AinStr, [u8]);

impl<'a> TryFrom<&'a [u8]> for &'a AinStr {
    type Error = AinValidationError;

    #[inline]
    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        AinStr::from_ascii(value)
    }
}

impl<'a> TryFrom<&'a mut [u8]> for &'a mut AinStr {
    type Error = AinValidationError;

    #[inline]
    fn try_from(value: &'a mut [u8]) -> Result<Self, Self::Error> {
        AinStr::from_ascii_mut(value)
    }
}

impl AsRef<str> for AinStr {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
from_ref_from_as_ref!(const AinStr, str);

impl<'a> TryFrom<&'a str> for &'a AinStr {
    type Error = AinValidationError;

    #[inline]
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        AinStr::from_str(value)
    }
}

impl<'a> TryFrom<&'a mut str> for &'a mut AinStr {
    type Error = AinValidationError;

    #[inline]
    fn try_from(value: &'a mut str) -> Result<Self, Self::Error> {
        AinStr::from_str_mut(value)
    }
}

impl AsRef<[AinChar]> for AinStr {
    #[inline]
    fn as_ref(&self) -> &[AinChar] {
        self.as_slice()
    }
}

impl AsMut<[AinChar]> for AinStr {
    #[inline]
    fn as_mut(&mut self) -> &mut [AinChar] {
        self.as_mut_slice()
    }
}

borrow_from_as_ref!(AinStr, [AinChar]);
from_ref_from_as_ref!(AinStr, [AinChar]);

impl Borrow<[AinChar]> for &AinStr {
    #[inline]
    fn borrow(&self) -> &[AinChar] {
        self.as_slice()
    }
}

impl Borrow<[AinChar]> for &mut AinStr {
    #[inline]
    fn borrow(&self) -> &[AinChar] {
        self.as_slice()
    }
}

impl BorrowMut<[AinChar]> for &mut AinStr {
    #[inline]
    fn borrow_mut(&mut self) -> &mut [AinChar] {
        self.as_mut_slice()
    }
}

impl AsRef<AinStr> for [AinChar] {
    #[inline]
    fn as_ref(&self) -> &AinStr {
        AinStr::from_slice(self)
    }
}

impl AsMut<AinStr> for [AinChar] {
    #[inline]
    fn as_mut(&mut self) -> &mut AinStr {
        AinStr::from_slice_mut(self)
    }
}

borrow_from_as_ref!([AinChar], AinStr);
from_ref_from_as_ref!([AinChar], AinStr);

impl Default for &AinStr {
    #[inline]
    fn default() -> &'static AinStr {
        AinStr::empty()
    }
}

impl Default for &mut AinStr {
    #[inline]
    fn default() -> &'static mut AinStr {
        AinStr::empty_mut()
    }
}

/// Unlike primitive slices, which return by reference because they don't know if the `T` type in
/// `[T]` is `Copy`, `AinStr` produces a copying iterator when iterated by reference.
impl<'a> IntoIterator for &'a AinStr {
    type Item = AinChar;
    type IntoIter = Chars<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.chars()
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

#[cfg(test)]
mod tests {
    use core::cmp::Ordering;
    use hashbrown::HashSet;

    #[test]
    fn case_insensitive_equality() {
        assert_eq!(ain!("abcd"), ain!("ABCD"));
        assert_eq!(ain!("abcd"), &[ain!('A'), ain!('b'), ain!('C'), ain!('d')]);
        assert_eq!(
            ain!("abcd"),
            &[ain!('A'), ain!('b'), ain!('C'), ain!('d')][..]
        );

        assert_eq!(ain!("abcd"), ain!("abcd"));
        assert_eq!(ain!("AbCd"), ain!("aBcD"));

        assert_ne!(ain!("AbCd"), ain!("aBc"));
        assert_ne!(ain!("AbCd"), ain!("aBcDe"));
        assert_ne!(ain!("AbCdf"), ain!("aBcDe"));
    }

    #[test]
    fn ord_upper_case() {
        assert_eq!(ain!("abcd").cmp(ain!("ABCD")), Ordering::Equal);
        assert_eq!(ain!("abcd").cmp(ain!("ABCe")), Ordering::Less);
        assert_eq!(ain!("abcF").cmp(ain!("ABCe")), Ordering::Greater);

        // Grave accent '`'` occurs between upper and lower case in ASCII.
        assert!(b'A' < b'`');
        assert!(b'`' < b'a');
        assert_eq!(ain!("A").cmp(ain!("`")), Ordering::Less);
        assert_eq!(ain!("a").cmp(ain!("`")), Ordering::Less);

        assert_eq!(
            ain!("abcd").partial_cmp(&[ain!('A'), ain!('b'), ain!('C'), ain!('E')]),
            Some(Ordering::Less)
        );
        assert_eq!(
            ain!("abce").partial_cmp(&[ain!('A'), ain!('b'), ain!('C'), ain!('d')][..]),
            Some(Ordering::Greater)
        );
    }

    #[test]
    fn ord_bigger_than_block_size() {
        assert_eq!(
            ain!("abcdefghijKlmnopQrstuvwxyz").cmp(ain!("AbcdefghijklMnOPQrstuvwxYZ")),
            Ordering::Equal
        );
        assert_eq!(
            ain!("abcdefghijKlmnopQrstuvwxyy").cmp(ain!("AbcdefghijklMnOPQrstuvwxYZ")),
            Ordering::Less
        );
        assert_eq!(
            ain!("abcdefghijKlmnopQrstuvwxyz").cmp(ain!("AbcdefghijklMnOPQrstuvwxYY")),
            Ordering::Greater
        );

        // Difference just before the block boundary.
        assert_eq!(
            ain!("abcdefghijKlmnoAQrstuvwxyz").cmp(ain!("AbcdefghijklMnOPQrstuvwxYy")),
            Ordering::Less
        );
        assert_eq!(
            ain!("abcdefghijKlmnopQrstuvwxYY").cmp(ain!("AbcdefghijklMnOAQrstuvwxYz")),
            Ordering::Greater
        );

        // Difference just after the block boundary.
        assert_eq!(
            ain!("abcdefghijKlmnopQrstuvwxyz").cmp(ain!("AbcdefghijklMnOPZrstuvwxYy")),
            Ordering::Less
        );
        assert_eq!(
            ain!("abcdefghijKlmnopZrstuvwxYY").cmp(ain!("AbcdefghijklMnOPQrstuvwxYz")),
            Ordering::Greater
        );
    }

    #[test]
    fn hash_matches_slice() {
        let mut set = HashSet::new();
        set.insert(ain!("ABCD"));
        assert!(set.contains(ain!("abcd")));
        assert!(set.contains(&[ain!('a'), ain!('B'), ain!('c'), ain!('D')][..]));
    }
}
