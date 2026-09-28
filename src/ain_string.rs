use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::borrow::{Borrow, BorrowMut};
use core::cmp::Ordering;
use core::error::Error;
use core::fmt;
use core::ops::{Add, AddAssign, Deref, DerefMut, Index, IndexMut};
use core::slice::SliceIndex;
use core::str::FromStr;

use crate::validation::run_ain_validation;
use crate::{AinChar, AinSliceIndexOutputMap, AinStr, AinValidationError};

/// A growable string stored as a case-insensitive ASCII encoded buffer.
///
/// For Ord, the 'case insensitive' order is to treat all characters as uppercase. This affects the
/// sort order of letters relative to the following symbols, which lie between uppercase and
/// lowercase ascii: `` [\]^_` ``
// We could derive PartialEq, Ord, and PartialOrd, but we better ensure autovectorization
// by writing them ourselves. For Hash, we derive it and rely on AinChar implementing hash_slice.
#[derive(Default, Clone, Eq, Hash)]
#[repr(transparent)]
pub struct AinString {
    vec: Vec<AinChar>,
}

// This is largely copied from ascii::AinString, with modification to change the types to the
// types from this library and the names based on personal preferences.
impl AinString {
    /// Creates a new, empty ASCII string buffer without allocating.
    ///
    /// # Examples
    /// ```
    /// # use ain::AinString;
    /// let mut s = AinString::new();
    /// ```
    #[inline]
    pub const fn new() -> Self {
        AinString { vec: Vec::new() }
    }

    /// Creates a new ASCII string buffer with the given capacity.
    /// The string will be able to hold exactly `capacity` bytes without reallocating.
    /// If `capacity` is 0, the ASCII string will not allocate.
    ///
    /// # Examples
    /// ```
    /// # use ain::AinString;
    /// let mut s = AinString::with_capacity(10);
    /// ```
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        AinString {
            vec: Vec::with_capacity(capacity),
        }
    }

    /// Convert a vector of AinChar into an AinString.
    #[inline]
    pub const fn from_vec(vec: Vec<AinChar>) -> Self {
        Self { vec }
    }

    /// Convert a string to an AinStr
    #[inline]
    pub fn from_string(s: String) -> Result<Self, IntoAinStringError<String>> {
        match run_ain_validation(s.as_bytes()) {
            Ok(()) => {
                let bytes = s.into_bytes();
                // SAFETY: we just checked that the bytes are valid.
                Ok(unsafe { Self::from_ascii_unchecked(bytes) })
            }
            Err(error) => Err(IntoAinStringError { error, owner: s }),
        }
    }

    /// Convert a vector of ASCII bytes into an AinString.
    pub fn from_ascii(bytes: Vec<u8>) -> Result<Self, IntoAinStringError<Vec<u8>>> {
        match run_ain_validation(&bytes) {
            Ok(()) => {
                // SAFETY: we just checked that the bytes are valid.
                Ok(unsafe { Self::from_ascii_unchecked(bytes) })
            }
            Err(error) => Err(IntoAinStringError {
                error,
                owner: bytes,
            }),
        }
    }

    /// Creates an `AinString` from a boxed `AinStr` slice without copying or allocating.
    pub fn from_boxed_ain_str(boxed: Box<AinStr>) -> Self {
        Self::from_vec(boxed.into_boxed_slice().into_vec())
    }

    /// Converts a vector of ascii bytes into an AinString without checking whether they are valid.
    ///
    /// # Safety
    ///
    /// The input vector must contain only ASCII bytes otherwise undefined behavior occurs.
    #[inline]
    pub unsafe fn from_ascii_unchecked(vec: Vec<u8>) -> Self {
        let (ptr, len, cap) = vec.into_raw_parts();
        // SAFETY: we already validated that the contents are all valid ASCII, and otherwise
        // AinChar is identical repr and stuch with u8.
        let ptr = ptr.cast::<AinChar>();
        let vec = unsafe { Vec::from_raw_parts(ptr, len, cap) };
        Self::from_vec(vec)
    }

    /// Creates a new `AinString` from a length, capacity and pointer.
    ///
    /// # Safety
    ///
    /// This is highly unsafe, due to the number of invariants that aren't checked:
    ///
    /// * The memory at `buf` need to have been previously allocated by the same allocator this
    ///   library uses, with an alignment of 1.
    /// * `length` needs to be less than or equal to `capacity`.
    /// * `capacity` needs to be the correct value.
    /// * `buf` must have `length` valid ascii elements and contain a total of `capacity` total,
    ///   possibly, uninitialized, elements.
    /// * Nothing else must be using the memory `buf` points to.
    ///
    /// Violating these may cause problems like corrupting the allocator's internal data structures.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use ain::AinString;
    /// use std::mem;
    ///
    /// unsafe {
    ///    let mut s = AinString::from_ascii("hello").unwrap();
    ///    let ptr = s.as_mut_ptr();
    ///    let len = s.len();
    ///    let capacity = s.capacity();
    ///
    ///    mem::forget(s);
    ///
    ///    let s = AinString::from_raw_parts(ptr, len, capacity);
    ///
    ///    assert_eq!(AinString::from_ascii("hello").unwrap(), s);
    /// }
    /// ```
    #[inline]
    pub unsafe fn from_raw_parts(buf: *mut AinChar, length: usize, capacity: usize) -> Self {
        AinString {
            // SAFETY: Caller guarantees that `buf` was previously allocated by this library, that
            // `buf` contains `length` valid ascii elements and has a total capacity of `capacity`
            // elements, and that nothing else is using the momory.
            vec: unsafe { Vec::from_raw_parts(buf, length, capacity) },
        }
    }

    /// Converts this `AinString` into a `Vec<AinChar>` without copying or allocating.
    #[inline]
    pub fn into_vec(self) -> Vec<AinChar> {
        self.vec
    }

    /// Convert this `AinString` into a `String` without copying or allocating.
    ///
    /// Since ASCII is a subset of utf-8, this is guaranteed to succeed, not utf-8 validation
    /// needed.
    #[inline]
    pub fn into_string(self) -> String {
        let bytes = self.into_bytes();
        // SAFETY: the bytes are guaranteed ASCII, so they must also be valud utf-8.
        unsafe { String::from_utf8_unchecked(bytes) }
    }

    /// Convert this `AinString` into a vector of bytes without copying or allocating.
    #[inline]
    pub fn into_bytes(self) -> Vec<u8> {
        let (ptr, len, cap) = self.vec.into_raw_parts();
        // SAFETY: every AinChar is a valid u8 since its repr(u8). Since we just got the data from a
        // vec, it is safe to turn it back into a vec.
        let ptr = ptr.cast::<u8>();
        unsafe { Vec::from_raw_parts(ptr, len, cap) }
    }

    /// Converts this [`AinString`] into a [`Box`]`<`[`AinStr`]`>`.
    ///
    /// This will drop any excess capacity
    #[inline]
    pub fn into_boxed_ain_str(self) -> Box<AinStr> {
        let slice = self.vec.into_boxed_slice();
        let ptr = Box::into_raw(slice) as *mut AinStr;
        // SAFETY: AinStr has the same repr as [AinChar]
        unsafe { Box::from_raw(ptr) }
    }

    /// Gets self as an [AinStr] slice.
    pub const fn as_ain_str(&self) -> &AinStr {
        AinStr::from_slice(self.vec.as_slice())
    }

    /// Gets self as a mutable [AinStr] slice.
    pub const fn as_mut_ain_str(&mut self) -> &mut AinStr {
        AinStr::from_mut_slice(self.vec.as_mut_slice())
    }

    /// Decomposes an `AinString` into its raw components: (pointer, length, capacity).
    ///
    /// Returns the raw pointer to the underlying data, the length of the string (in bytes), and the
    /// allocated capacity of the data (in bytes). These are the same arguments in the same order as
    /// the arguments to [from_raw_parts][Self::from_raw_parts].
    ///
    /// After calling this function, the caller is responsible for the memory previously managed by
    /// the `AinString`. The only way to do this is to convert the raw pointer, length, and capacity
    /// back into an `AinString` with the `from_raw_parts` function, allowing the destructor to
    /// perform the cleanup.
    #[inline]
    pub fn into_raw_parts(self) -> (*mut AinChar, usize, usize) {
        self.vec.into_raw_parts()
    }

    /// Returns the number of bytes in this ASCII string.
    ///
    /// # Examples
    /// ```
    /// # use ain::AinString;
    /// let s = AinString::from_ascii("foo").unwrap();
    /// assert_eq!(s.len(), 3);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    /// Returns true if the ASCII string contains zero bytes.
    ///
    /// # Examples
    /// ```
    /// # use ain::{AsciiChar, AinString};
    /// let mut s = AinString::new();
    /// assert!(s.is_empty());
    /// s.push(AsciiChar::from_ascii('a').unwrap());
    /// assert!(!s.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    /// Returns the number of bytes that this ASCII string buffer can hold without reallocating.
    ///
    /// # Examples
    /// ```
    /// # use ain::AinString;
    /// let s = String::with_capacity(10);
    /// assert!(s.capacity() >= 10);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        self.vec.capacity()
    }

    /// Reserves capacity for at least `additional` more bytes to be inserted in the given
    /// `AinString`. The collection may reserve more space to avoid frequent reallocations.
    ///
    /// # Panics
    /// Panics if the new capacity overflows `usize`.
    ///
    /// # Examples
    /// ```
    /// # use ain::AinString;
    /// let mut s = AinString::new();
    /// s.reserve(10);
    /// assert!(s.capacity() >= 10);
    /// ```
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.vec.reserve(additional);
    }

    /// Reserves the minimum capacity for exactly `additional` more bytes to be inserted in the
    /// given `AinString`. Does nothing if the capacity is already sufficient.
    ///
    /// Note that the allocator may give the collection more space than it requests. Therefore
    /// capacity can not be relied upon to be precisely minimal. Prefer `reserve` if future
    /// insertions are expected.
    ///
    /// # Panics
    /// Panics if the new capacity overflows `usize`.
    ///
    /// # Examples
    /// ```
    /// # use ain::AinString;
    /// let mut s = AinString::new();
    /// s.reserve_exact(10);
    /// assert!(s.capacity() >= 10);
    /// ```
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.vec.reserve_exact(additional);
    }

    /// Shrinks the capacity of this ASCII string buffer to match it's length.
    ///
    /// # Examples
    /// ```
    /// # use ain::AinString;
    /// use std::str::FromStr;
    /// let mut s = AinString::from_str("foo").unwrap();
    /// s.reserve(100);
    /// assert!(s.capacity() >= 100);
    /// s.shrink_to_fit();
    /// assert_eq!(s.capacity(), 3);
    /// ```
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.vec.shrink_to_fit();
    }

    /// Adds the given ASCII character to the end of the ASCII string.
    ///
    /// # Examples
    /// ```
    /// # use ain::{ AsciiChar, AinString};
    /// let mut s = AinString::from_ascii("abc").unwrap();
    /// s.push(AsciiChar::from_ascii('1').unwrap());
    /// s.push(AsciiChar::from_ascii('2').unwrap());
    /// s.push(AsciiChar::from_ascii('3').unwrap());
    /// assert_eq!(s, "abc123");
    /// ```
    #[inline]
    pub fn push(&mut self, ch: AinChar) {
        self.vec.push(ch);
    }

    /// Pushes the given ASCII string onto this ASCII string buffer.
    ///
    /// # Examples
    /// ```
    /// # use ain::{AinString, AsAinStr};
    /// use std::str::FromStr;
    /// let mut s = AinString::from_str("foo").unwrap();
    /// s.push_str("bar".as_ascii_str().unwrap());
    /// assert_eq!(s, "foobar".as_ascii_str().unwrap());
    /// ```
    #[inline]
    pub fn push_str(&mut self, string: &AinStr) {
        self.vec.extend_from_slice(string.as_slice())
    }

    /// Inserts the given ASCII string at the given place in this ASCII string buffer.
    ///
    /// # Panics
    ///
    /// Panics if `idx` is larger than the `AinString`'s length.
    ///
    /// # Examples
    /// ```
    /// # use ain::{AinString, AsAinStr};
    /// use std::str::FromStr;
    /// let mut s = AinString::from_str("abc").unwrap();
    /// s.insert_str(1, "def".as_ascii_str().unwrap());
    /// assert_eq!(&*s, "adefbc");
    #[inline]
    pub fn insert_str(&mut self, idx: usize, string: &AinStr) {
        self.vec.reserve(string.len());
        self.vec.splice(idx..idx, string.chars());
    }

    /// Shortens a ASCII string to the specified length.
    ///
    /// # Panics
    /// Panics if `new_len` > current length.
    ///
    /// # Examples
    /// ```
    /// # use ain::AinString;
    /// let mut s = AinString::from_ascii("hello").unwrap();
    /// s.truncate(2);
    /// assert_eq!(s, "he");
    /// ```
    #[inline]
    pub fn truncate(&mut self, new_len: usize) {
        self.vec.truncate(new_len);
    }

    /// Removes the last character from the ASCII string buffer and returns it.
    /// Returns `None` if this string buffer is empty.
    ///
    /// # Examples
    /// ```
    /// # use ain::AinString;
    /// let mut s = AinString::from_ascii("foo").unwrap();
    /// assert_eq!(s.pop().map(|c| c.as_char()), Some('o'));
    /// assert_eq!(s.pop().map(|c| c.as_char()), Some('o'));
    /// assert_eq!(s.pop().map(|c| c.as_char()), Some('f'));
    /// assert_eq!(s.pop(), None);
    /// ```
    #[inline]
    pub fn pop(&mut self) -> Option<AinChar> {
        self.vec.pop()
    }

    /// Removes the ASCII character at position `idx` from the buffer and returns it.
    ///
    /// # Warning
    /// This is an O(n) operation as it requires copying every element in the buffer.
    ///
    /// # Panics
    /// If `idx` is out of bounds this function will panic.
    ///
    /// # Examples
    /// ```
    /// # use ain::AinString;
    /// let mut s = AinString::from_ascii("foo").unwrap();
    /// assert_eq!(s.remove(0).as_char(), 'f');
    /// assert_eq!(s.remove(1).as_char(), 'o');
    /// assert_eq!(s.remove(0).as_char(), 'o');
    /// ```
    #[inline]
    pub fn remove(&mut self, idx: usize) -> AinChar {
        self.vec.remove(idx)
    }

    /// Inserts an ASCII character into the buffer at position `idx`.
    ///
    /// # Warning
    /// This is an O(n) operation as it requires copying every element in the buffer.
    ///
    /// # Panics
    /// If `idx` is out of bounds this function will panic.
    ///
    /// # Examples
    /// ```
    /// # use ain::{AinString,AsciiChar};
    /// let mut s = AinString::from_ascii("foo").unwrap();
    /// s.insert(2, AsciiChar::b);
    /// assert_eq!(s, "fobo");
    /// ```
    #[inline]
    pub fn insert(&mut self, idx: usize, ch: AinChar) {
        self.vec.insert(idx, ch);
    }

    /// Truncates the ASCII string, setting length (but not capacity) to zero.
    ///
    /// # Examples
    /// ```
    /// # use ain::AinString;
    /// let mut s = AinString::from_ascii("foo").unwrap();
    /// s.clear();
    /// assert!(s.is_empty());
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        self.vec.clear();
    }
}

impl Deref for AinString {
    type Target = AinStr;

    #[inline]
    fn deref(&self) -> &AinStr {
        self.as_ain_str()
    }
}

impl DerefMut for AinString {
    #[inline]
    fn deref_mut(&mut self) -> &mut AinStr {
        self.as_mut_ain_str()
    }
}

impl fmt::Debug for AinString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_ain_str(), f)
    }
}

impl fmt::Display for AinString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_ain_str(), f)
    }
}

impl PartialEq for AinString {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(self.as_ain_str(), other.as_ain_str())
    }
}

impl PartialOrd for AinString {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(Ord::cmp(self, other))
    }
}

impl Ord for AinString {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(self.as_ain_str(), other.as_ain_str())
    }
}

macro_rules! impl_eq {
    (@byval $lhs:ty, $rhs:ty) => {
        impl PartialEq<$rhs> for $lhs {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool {
                PartialEq::eq(&**self, other)
            }
        }

        impl PartialEq<$lhs> for $rhs {
            #[inline]
            fn eq(&self, other: &$lhs) -> bool {
                PartialEq::eq(self, &**other)
            }
        }

        impl PartialOrd<$rhs> for $lhs {
            #[inline]
            fn partial_cmp(&self, other: &$rhs) -> Option<Ordering> {
                PartialOrd::partial_cmp(&**self, other)
            }
        }

        impl PartialOrd<$lhs> for $rhs {
            #[inline]
            fn partial_cmp(&self, other: &$lhs) -> Option<Ordering> {
                PartialOrd::partial_cmp(self, &**other)
            }
        }
    };
    (@byref $lhs:ty, $rhs:ty) => {
        impl PartialEq<$rhs> for $lhs {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool {
                PartialEq::eq(&**self, &**other)
            }
        }

        impl PartialOrd<$rhs> for $lhs {
            #[inline]
            fn partial_cmp(&self, other: &$rhs) -> Option<Ordering> {
                PartialOrd::partial_cmp(&**self, &**other)
            }
        }
    };
}

impl_eq! { @byval AinString, AinStr }
impl_eq! { @byref &AinStr, AinString }
impl_eq! { @byref AinString, &AinStr }

impl_eq! { @byval AinString, [AinChar] }
impl_eq! { @byref &[AinChar], AinString }
impl_eq! { @byref AinString, &[AinChar] }

impl_eq! { @byval Cow<'_, AinStr>, AinStr }
impl_eq! { @byref AinString, Cow<'_, AinStr> }
impl_eq! { @byref Cow<'_, AinStr>, AinString }
impl_eq! { @byref &AinStr, Cow<'_, AinStr> }
impl_eq! { @byref Cow<'_, AinStr>, &AinStr }

impl<S> Index<S> for AinString
where
    S: SliceIndex<[AinChar]>,
    S::Output: AinSliceIndexOutputMap + 'static,
{
    type Output = <S::Output as AinSliceIndexOutputMap>::Output;

    #[inline]
    fn index(&self, index: S) -> &Self::Output {
        &self.as_ain_str()[index]
    }
}

impl<S> IndexMut<S> for AinString
where
    S: SliceIndex<[AinChar]>,
    S::Output: AinSliceIndexOutputMap + 'static,
{
    #[inline]
    fn index_mut(&mut self, index: S) -> &mut Self::Output {
        &mut self.as_mut_ain_str()[index]
    }
}

impl AsRef<[u8]> for AinString {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl From<AinString> for Vec<u8> {
    #[inline]
    fn from(value: AinString) -> Self {
        value.into_bytes()
    }
}

impl TryFrom<Vec<u8>> for AinString {
    type Error = IntoAinStringError<Vec<u8>>;

    #[inline]
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Self::from_ascii(value)
    }
}

impl AsRef<str> for AinString {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<AinString> for String {
    #[inline]
    fn from(value: AinString) -> Self {
        value.into_string()
    }
}

impl TryFrom<String> for AinString {
    type Error = IntoAinStringError<String>;

    #[inline]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_string(value)
    }
}

impl AsRef<AinStr> for AinString {
    #[inline]
    fn as_ref(&self) -> &AinStr {
        self.as_ain_str()
    }
}

impl AsMut<AinStr> for AinString {
    #[inline]
    fn as_mut(&mut self) -> &mut AinStr {
        self.as_mut_ain_str()
    }
}

borrow_from_as_ref!(AinString, AinStr);

impl AsRef<[AinChar]> for AinString {
    #[inline]
    fn as_ref(&self) -> &[AinChar] {
        self.as_slice()
    }
}

impl AsMut<[AinChar]> for AinString {
    #[inline]
    fn as_mut(&mut self) -> &mut [AinChar] {
        self.as_mut_slice()
    }
}

borrow_from_as_ref!(AinString, [AinChar]);

impl From<Vec<AinChar>> for AinString {
    #[inline]
    fn from(value: Vec<AinChar>) -> Self {
        Self::from_vec(value)
    }
}

impl From<AinString> for Vec<AinChar> {
    #[inline]
    fn from(value: AinString) -> Self {
        value.into_vec()
    }
}

impl From<AinString> for Box<AinStr> {
    #[inline]
    fn from(value: AinString) -> Self {
        value.into_boxed_ain_str()
    }
}

impl From<Box<AinStr>> for AinString {
    #[inline]
    fn from(value: Box<AinStr>) -> Self {
        value.into_ain_string()
    }
}

impl<'a> From<&'a AinString> for Cow<'a, AinStr> {
    #[inline]
    fn from(value: &'a AinString) -> Self {
        Cow::Borrowed(value.as_ain_str())
    }
}

impl From<AinString> for Cow<'static, AinStr> {
    #[inline]
    fn from(value: AinString) -> Self {
        Cow::Owned(value)
    }
}

impl From<AinString> for Arc<AinStr> {
    #[inline]
    fn from(value: AinString) -> Self {
        let arc: Arc<[AinChar]> = Arc::from(value.vec);
        // SAFETY: [AinChar] and AinStr have the same repr.
        let ptr = Arc::into_raw(arc) as *const AinStr;
        unsafe { Arc::from_raw(ptr) }
    }
}

impl From<AinString> for Rc<AinStr> {
    #[inline]
    fn from(value: AinString) -> Self {
        let arc: Rc<[AinChar]> = Rc::from(value.vec);
        // SAFETY: [AinChar] and AinStr have the same repr.
        let ptr = Rc::into_raw(arc) as *const AinStr;
        unsafe { Rc::from_raw(ptr) }
    }
}

impl FromStr for AinString {
    type Err = AinValidationError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        AinStr::from_str(s).map(AinStr::to_ain_string)
    }
}

/// A possible error value when converting into an  `AinString` from a byte vector or string.
/// It wraps an [`AinValidationError`] which you can get through the `validation_error()` method.
///
/// #Examples
/// ```
/// # use ain::IntoAinString;
/// let err = "bø!".to_string().into_ascii_string().unwrap_err();
/// assert_eq!(err.ascii_error().valid_up_to(), 1);
/// assert_eq!(err.into_source(), "bø!".to_string());
/// ```
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct IntoAinStringError<O> {
    error: AinValidationError,
    owner: O,
}
impl<O> IntoAinStringError<O> {
    /// Get the position of the first non-ASCII byte or character.
    #[inline]
    pub fn validation_error(&self) -> AinValidationError {
        self.error
    }
    /// Get back the original, unmodified type.
    #[inline]
    pub fn into_source(self) -> O {
        self.owner
    }
}
impl<O> fmt::Debug for IntoAinStringError<O> {
    #[inline]
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.error, fmtr)
    }
}
impl<O> fmt::Display for IntoAinStringError<O> {
    #[inline]
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.error, fmtr)
    }
}

impl<O> Error for IntoAinStringError<O> {
    /// Always returns an [`AinValidationError`]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.error)
    }
}

/// Please note that the `std::fmt::Result` returned by these methods does not support
/// transmission of an error other than that an error occurred.
impl fmt::Write for AinString {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let astr = AinStr::from_str(s)?;
        self.push_str(astr);
        Ok(())
    }

    fn write_char(&mut self, c: char) -> fmt::Result {
        let achar = AinChar::from_char(c).ok_or(fmt::Error)?;
        self.push(achar);
        Ok(())
    }
}

impl<A: AsRef<AinStr>> FromIterator<A> for AinString {
    fn from_iter<I: IntoIterator<Item = A>>(iter: I) -> AinString {
        let mut buf = AinString::new();
        buf.extend(iter);
        buf
    }
}

impl<A: AsRef<AinStr>> Extend<A> for AinString {
    fn extend<I: IntoIterator<Item = A>>(&mut self, iterable: I) {
        let iterator = iterable.into_iter();
        let (lower_bound, _) = iterator.size_hint();
        self.reserve(lower_bound);
        for item in iterator {
            self.push_str(item.as_ref());
        }
    }
}

impl<'a> Add<&'a AinStr> for AinString {
    type Output = AinString;

    #[inline]
    fn add(mut self, other: &AinStr) -> AinString {
        self.push_str(other);
        self
    }
}

impl<'a> AddAssign<&'a AinStr> for AinString {
    #[inline]
    fn add_assign(&mut self, other: &AinStr) {
        self.push_str(other);
    }
}
