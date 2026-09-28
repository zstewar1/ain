use core::borrow::{Borrow, BorrowMut};
use core::cmp::Ordering;
use core::iter::FusedIterator;
use core::ops::{Index, IndexMut};
use core::slice::{Iter, IterMut, SliceIndex};
use core::{fmt, mem};

use crate::validation::run_ain_validation;
use crate::{AinChar, AinValidationError};

/// [`AinStr`] represents a string slice that only contains ASCII characters, and has
/// case-insensitive comparisons.
///
/// It wraps a slice of [`AinChar`] and implements many of `str`s methods and traits.
///
/// It can be created by a checked conversion from a `str` or `[u8]`, or borrowed from an
/// `AinString`.
///
/// For Ord, the 'case insensitive' order is to treat all characters as uppercase. This affects the
/// sort order of letters relative to the following symbols, which lie between uppercase and
/// lowercase ascii: `` [\]^_` ``
// We could derive PartialEq, Ord, and PartialOrd,  but we can better ensure autovectorization
// by writing them ourselves. For Hash, we derive it relying on the AinChar implementation of
// hash_slice.
#[derive(Eq, Hash)]
#[repr(transparent)]
pub struct AinStr {
    pub(crate) slice: [AinChar],
}

// This is mostly copied from ascii::AsciiStr.
impl AinStr {
    /// An empty `AinStr`.
    // SAFETY: an empty slice is always valid as it cannot possibly contain any invalid bytes.
    pub const EMPTY: &'static AinStr = AinStr::from_slice(&[]);

    /// Converts a slice of [AinChar] to an `AinStr`.
    ///
    /// # Examples
    /// ```
    /// # use ain::{AinStr, AinChar},;
    /// let foo = AinStr::from_slice(&[AinChar::SmallF, AinChar::SmallO, AinChar::SmallO]);
    /// assert_eq!(foo.as_str(), "foo");
    /// ```
    #[inline]
    pub const fn from_slice(slice: &[AinChar]) -> &Self {
        // SAFETY: We're going from one slice to another slice with identical repr and lifetime.
        unsafe { &*(slice as *const [AinChar] as *const AinStr) }
    }

    /// Converts a mutable slice of [AinChar] to an `AinStr`.
    ///
    /// # Examples
    /// ```
    /// # use ain::{AinStr, AinChar};
    /// let foo = AinStr::from_mut_slice(&mut [AinChar::SmallF, AinChar::SmallO, AinChar::SmallO]);
    /// assert_eq!(foo.as_str(), "foo");
    /// ```
    #[inline]
    pub const fn from_mut_slice(slice: &mut [AinChar]) -> &mut Self {
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
    pub const fn from_mut_str(s: &mut str) -> Result<&mut AinStr, AinValidationError> {
        // SAFETY: because from_ascii_mut validates that the input is all ASCII, and an &mut AinStr
        // cannot change any character to be non-ascii, this guarantees that the original str will
        // all be valid utf-8 when the borrow ends.
        Self::from_mut_ascii(unsafe { s.as_bytes_mut() })
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
    pub const fn from_mut_ascii(bytes: &mut [u8]) -> Result<&mut AinStr, AinValidationError> {
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
    /// let foo = unsafe { AinStr::from_ascii_unchecked_mut(&mut b"foo"[..]) };
    /// assert_eq!(foo.as_str(), "foo");
    /// ```
    #[inline]
    pub const unsafe fn from_ascii_unchecked_mut(bytes: &mut [u8]) -> &mut Self {
        // SAFETY: Caller guarantees all bytes in `bytes` are valid ascii characters.
        let ascii = unsafe { &mut *(bytes as *mut [u8] as *mut [AinChar]) };
        Self::from_mut_slice(ascii)
    }

    /// Converts `&self` to a `&str` slice.
    #[inline]
    pub const fn as_str(&self) -> &str {
        // SAFETY: All variants of `AinChar` are valid bytes for a `str`.
        unsafe { &*(self as *const AinStr as *const str) }
    }

    /// Converts `&self` into a byte slice.
    #[inline]
    pub const fn as_bytes(&self) -> &[u8] {
        // SAFETY: All variants of `AinChar` are valid `u8`, given they're `repr(u8)`.
        unsafe { &*(self as *const AinStr as *const [u8]) }
    }

    /// Returns the entire string as slice of `AinChar`s.
    #[inline]
    pub const fn as_slice(&self) -> &[AinChar] {
        &self.slice
    }

    /// Returns the entire string as mutable slice of `AinChar`s.
    ///
    /// Since all characters are a single byte, all safe mutations of the slice are safe for AinStr.
    #[inline]
    pub const fn as_mut_slice(&mut self) -> &mut [AinChar] {
        &mut self.slice
    }

    /// Gets a pointer to the contents of the `AinStr`.
    #[inline]
    pub const fn as_ptr(&self) -> *const AinChar {
        self.as_slice().as_ptr()
    }

    /// Returns a mutable pointer to the `AinStr`'s contents.
    #[inline]
    pub const fn as_mut_ptr(&mut self) -> *mut AinChar {
        self.as_mut_slice().as_mut_ptr()
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
    pub const fn is_empty(&self) -> bool {
        self.slice.is_empty()
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

    /// Find the index of the first occurence of the given character in this AinStr.
    #[inline]
    pub const fn find(&self, ch: AinChar) -> Option<usize> {
        let mut idx = 0;
        while idx < self.len() {
            // we have to index as a slice and can't use == in order for this to work in const.
            if self.slice[idx].eq(ch) {
                return Some(idx);
            }
            idx += 1;
        }
        None
    }

    /// Find the index of the last occurence of the given character in this AinStr.
    #[inline]
    pub const fn rfind(&self, ch: AinChar) -> Option<usize> {
        let mut idx = self.len();
        while idx > 0 {
            idx -= 1;
            // we have to index as a slice and can't use == in order for this to work in const.
            if self.slice[idx].eq(ch) {
                return Some(idx);
            }
        }
        None
    }

    /// Split this string slice at the given index.
    #[inline]
    pub const fn split_at(&self, idx: usize) -> (&Self, &Self) {
        let (l, r) = self.slice.split_at(idx);
        (Self::from_slice(l), Self::from_slice(r))
    }

    /// Split this string slice at the given index.
    #[inline]
    pub const fn split_at_mut(&mut self, idx: usize) -> (&mut Self, &mut Self) {
        let (l, r) = self.slice.split_at_mut(idx);
        (Self::from_mut_slice(l), Self::from_mut_slice(r))
    }

    /// Returns an iterator over parts of the `AinStr` separated by a character.
    ///
    /// If the split character appears multiple times in a row, it will produce empty strings in
    /// between copies of the separator.
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
    #[inline]
    pub fn split(&self, on: AinChar) -> Split<'_> {
        Split {
            separator: on,
            remaining: Some(self),
        }
    }

    /// Returns an iterator over mutable parts of the `AinStr` separated by a character.
    ///
    /// If the split character appears multiple times in a row, it will produce empty strings in
    /// between copies of the separator.
    ///
    /// # Examples
    /// ```
    /// # use ain::{AinStr, AinChar};
    /// let words = AinStr::from_ascii("apple banana lemon").unwrap()
    ///     .split_mut(AinChar::Space)
    ///     .map(|a| a.as_str())
    ///     .collect::<Vec<_>>();
    /// assert_eq!(words, ["apple", "banana", "lemon"]);
    /// ```
    #[inline]
    pub fn split_mut(&mut self, on: AinChar) -> SplitMut<'_> {
        SplitMut {
            separator: on,
            remaining: Some(self),
        }
    }

    /// Returns an iterator over the lines of the `AinStr`, which are themselves `AinStr`s.
    ///
    /// Lines are ended with either `LineFeed` (`\n`), or `CarriageReturn` then `LineFeed` (`\r\n`).
    ///
    /// The final line ending is optional. A string that ends with a final line ending will return
    /// the same lines as an otherwise identical string without a final line ending.
    #[inline]
    pub fn lines(&self) -> Lines<'_> {
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
    #[inline]
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
    /// # Examples
    /// ```
    /// # use ain::AinStr;
    /// let example = AinStr::from_ascii("  \twhite \tspace  \t").unwrap();
    /// assert_eq!("  \twhite \tspace", example.trim_end());
    /// ```
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
    #[inline]
    pub const fn first(&self) -> Option<AinChar> {
        self.slice.first().copied()
    }

    /// Returns the last character if the string is not empty.
    #[inline]
    pub const fn last(&self) -> Option<AinChar> {
        self.slice.last().copied()
    }
}

impl fmt::Debug for AinStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl fmt::Display for AinStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

impl PartialEq for AinStr {
    #[inline]
    fn eq(&self, other: &AinStr) -> bool {
        self.as_bytes().eq_ignore_ascii_case(other.as_bytes())
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
    fn eq_ignore_ascii_inner<const L: usize>(lhs: &[AinChar; L], rhs: &[AinChar; L]) -> bool {
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
        if !eq_ignore_ascii_inner(lhs_chunk, rhs_chunk) {
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
        ) {
            if !eq_ignore_ascii_inner(lhs_tail, rhs_tail) {
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

/// Trait to help map from slice-index types to the correct output types of [AinStr] indexing.
pub trait AinSliceIndexOutputMap {
    type Output: ?Sized + 'static;

    fn convert_slice_index_output(&self) -> &Self::Output;

    fn convert_slice_index_output_mut(&mut self) -> &mut Self::Output;
}

impl AinSliceIndexOutputMap for [AinChar] {
    type Output = AinStr;

    #[inline]
    fn convert_slice_index_output(&self) -> &Self::Output {
        AinStr::from_slice(self)
    }

    #[inline]
    fn convert_slice_index_output_mut(&mut self) -> &mut Self::Output {
        AinStr::from_mut_slice(self)
    }
}

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
        AinStr::from_mut_ascii(value)
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
        AinStr::from_mut_str(value)
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

impl AsRef<AinStr> for [AinChar] {
    #[inline]
    fn as_ref(&self) -> &AinStr {
        AinStr::from_slice(self)
    }
}

impl AsMut<AinStr> for [AinChar] {
    #[inline]
    fn as_mut(&mut self) -> &mut AinStr {
        AinStr::from_mut_slice(self)
    }
}

borrow_from_as_ref!([AinChar], AinStr);
from_ref_from_as_ref!([AinChar], AinStr);

impl Default for &'static AinStr {
    #[inline]
    fn default() -> &'static AinStr {
        AinStr::EMPTY
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
        CharsRef {
            inner: self.slice.iter(),
        }
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
pub struct Chars<'a> {
    inner: Iter<'a, AinChar>,
}

impl<'a> Chars<'a> {
    /// Returns the ascii string slice with the remaining characters.
    #[inline]
    pub fn as_ain_str(&self) -> &'a AinStr {
        AinStr::from_slice(self.inner.as_slice())
    }
}

impl<'a> Iterator for Chars<'a> {
    type Item = AinChar;

    #[inline]
    fn next(&mut self) -> Option<AinChar> {
        self.inner.next().copied()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.inner.count()
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        self.inner.last().copied()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.inner.nth(n).copied()
    }

    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.inner.copied().fold(init, f)
    }
}

impl<'a> DoubleEndedIterator for Chars<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<AinChar> {
        self.inner.next_back().copied()
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.inner.nth_back(n).copied()
    }

    #[inline]
    fn rfold<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.inner.copied().rfold(init, f)
    }
}

impl<'a> ExactSizeIterator for Chars<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'a> FusedIterator for Chars<'a> {}

/// A mutable iterator over the characters of an `AinStr`.
#[derive(Debug)]
pub struct CharsMut<'a> {
    inner: IterMut<'a, AinChar>,
}

impl<'a> CharsMut<'a> {
    /// Returns the ascii string slice with the remaining characters, using the original lifetime by
    /// destroying the iterator.
    #[inline]
    pub fn into_ain_str(self) -> &'a mut AinStr {
        AinStr::from_mut_slice(self.inner.into_slice())
    }

    /// Returns the ascii string slice with the remaining characters.
    #[inline]
    pub fn as_ain_str(&self) -> &AinStr {
        AinStr::from_slice(self.inner.as_slice())
    }
}

impl<'a> Iterator for CharsMut<'a> {
    type Item = &'a mut AinChar;

    #[inline]
    fn next(&mut self) -> Option<&'a mut AinChar> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.inner.count()
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        self.inner.last()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.inner.nth(n)
    }

    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.inner.fold(init, f)
    }
}

impl<'a> DoubleEndedIterator for CharsMut<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a mut AinChar> {
        self.inner.next_back()
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.inner.nth_back(n)
    }

    #[inline]
    fn rfold<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.inner.rfold(init, f)
    }
}

impl<'a> ExactSizeIterator for CharsMut<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'a> FusedIterator for CharsMut<'a> {}

/// An immutable iterator over the characters of an `AinStr`.
#[derive(Clone, Debug)]
pub struct CharsRef<'a> {
    inner: Iter<'a, AinChar>,
}

impl<'a> CharsRef<'a> {
    /// Returns the ascii string slice with the remaining characters.
    #[inline]
    pub fn as_ain_str(&self) -> &'a AinStr {
        AinStr::from_slice(self.inner.as_slice())
    }
}
impl<'a> Iterator for CharsRef<'a> {
    type Item = &'a AinChar;

    #[inline]
    fn next(&mut self) -> Option<&'a AinChar> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.inner.count()
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        self.inner.last()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.inner.nth(n)
    }

    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.inner.fold(init, f)
    }
}

impl<'a> DoubleEndedIterator for CharsRef<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a AinChar> {
        self.inner.next_back()
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.inner.nth_back(n)
    }

    #[inline]
    fn rfold<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.inner.rfold(init, f)
    }
}

impl<'a> ExactSizeIterator for CharsRef<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'a> FusedIterator for CharsRef<'a> {}

/// An iterator over parts of an `AinStr` separated by an `AinChar`.
///
/// This type is created by [`AinStr::split`]
#[derive(Clone, Debug)]
pub struct Split<'a> {
    separator: AinChar,
    remaining: Option<&'a AinStr>,
}
impl<'a> Iterator for Split<'a> {
    type Item = &'a AinStr;

    fn next(&mut self) -> Option<&'a AinStr> {
        if let Some(remaining) = self.remaining {
            if let Some(idx) = remaining.find(self.separator) {
                // The index of the separtor must be < len, so this addition is always <= len.
                self.remaining = Some(&remaining[idx + 1..]);
                Some(&remaining[..idx])
            } else {
                self.remaining = None;
                Some(remaining)
            }
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.remaining {
            None => (0, Some(0)),
            Some(remaining) => {
                // If there are no remaining separators in the string, we will return 1 element,
                // even if it's empty. If every remaining character is a separator, we will produce
                // 1 output between each pair of separators as well as the start and end, which is
                // one more than the number of separators (fence post problem).
                (1, remaining.len().checked_add(1))
            }
        }
    }
}

impl<'a> DoubleEndedIterator for Split<'a> {
    fn next_back(&mut self) -> Option<&'a AinStr> {
        if let Some(remaining) = self.remaining {
            if let Some(idx) = remaining.rfind(self.separator) {
                self.remaining = Some(&remaining[..idx]);
                // The index of the separtor must be < len, so this addition is always <= len.
                Some(&remaining[idx + 1..])
            } else {
                self.remaining = None;
                Some(remaining)
            }
        } else {
            None
        }
    }
}

impl<'a> FusedIterator for Split<'a> {}

/// An iterator over mutable parts of an `AinStr` separated by an `AinChar`.
///
/// This type is created by [`AinChar::split()`](struct.AinChar.html#method.split).
#[derive(Debug)]
pub struct SplitMut<'a> {
    separator: AinChar,
    remaining: Option<&'a mut AinStr>,
}
impl<'a> Iterator for SplitMut<'a> {
    type Item = &'a mut AinStr;

    fn next(&mut self) -> Option<&'a mut AinStr> {
        if let Some(remaining) = self.remaining.take() {
            if let Some(idx) = remaining.find(self.separator) {
                let (start, rest) = remaining.split_at_mut(idx);
                // Rest contains the separator, so it must have len >= 1
                self.remaining = Some(&mut rest[1..]);
                Some(start)
            } else {
                Some(remaining)
            }
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.remaining {
            None => (0, Some(0)),
            Some(remaining) => {
                // If there are no remaining separators in the string, we will return 1 element, even if
                // it's empty. If every remaining character is a separator, we will produce 1 output
                // between each pair of separators as well as the start and end, which is one more than
                // the number of separators (fence post problem).
                (1, remaining.len().checked_add(1))
            }
        }
    }
}

impl<'a> DoubleEndedIterator for SplitMut<'a> {
    fn next_back(&mut self) -> Option<&'a mut AinStr> {
        if let Some(remaining) = self.remaining.take() {
            if let Some(idx) = remaining.rfind(self.separator) {
                let (rest, end) = remaining.split_at_mut(idx);
                self.remaining = Some(rest);
                // End contains the separator so it must have len >= 1
                Some(&mut end[1..])
            } else {
                Some(remaining)
            }
        } else {
            None
        }
    }
}

impl<'a> FusedIterator for SplitMut<'a> {}

/// An iterator over the lines of the internal character array.
#[derive(Clone, Debug)]
pub struct Lines<'a> {
    string: &'a AinStr,
}
impl<'a> Iterator for Lines<'a> {
    type Item = &'a AinStr;

    fn next(&mut self) -> Option<&'a AinStr> {
        if let Some(idx) = self.string.find(AinChar::LineFeed) {
            let line = if idx > 0 && self.string[idx - 1] == AinChar::CarriageReturn {
                &self.string[..idx - 1]
            } else {
                &self.string[..idx]
            };
            self.string = &self.string[idx + 1..];
            Some(line)
        } else if self.string.is_empty() {
            None
        } else {
            let line = self.string;
            self.string = AinStr::EMPTY;
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
            // `last()` returned `Some`, so our len is at least 1.
            self.string = &self.string[..self.string.len() - 1];

            if self.string.last() == Some(AinChar::CarriageReturn) {
                // `last()` returned `Some`, so our len is at least 1.
                self.string = &self.string[..self.string.len() - 1];
            }
        }

        // Get the position of the first `LF` from the end.
        match self.string.rfind(AinChar::LineFeed) {
            Some(idx) => {
                let line = &self.string[idx + 1..];
                // We keep the LF in the previous string so that we are aware of it in case we need
                // to strip a CR later.
                self.string = &self.string[..idx + 1];
                Some(line)
            }
            None => Some(mem::replace(&mut self.string, AinStr::EMPTY)),
        }
    }
}

impl<'a> FusedIterator for Lines<'a> {}
