use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
#[cfg(feature = "std")]
use core::any::Any;
use core::borrow::{Borrow, BorrowMut};
use core::fmt;
use core::ops::{Add, AddAssign, Deref, DerefMut, Index, IndexMut};
use core::str::FromStr;
#[cfg(feature = "std")]
use std::error::Error;

use ascii::{AsAsciiStr, AsciiString};

use crate::{AinChar, AinStr, AsAinStr, AsAinStrError};

/// A growable string stored as a case-insensitive ASCII encoded buffer.
///
/// Since the range of allowed values is exactly the same as [`AinString`] it is always safe to
/// convert between them bidirectionally.
// Because AinChar implements eq, ord, and hash with case-insensitive checks, we can derive these
// here and get correct case-insensitive behavior.
#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    #[must_use]
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
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        AinString {
            vec: Vec::with_capacity(capacity),
        }
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
    #[must_use]
    pub unsafe fn from_raw_parts(buf: *mut AinChar, length: usize, capacity: usize) -> Self {
        AinString {
            // SAFETY: Caller guarantees that `buf` was previously allocated by this library,
            //         that `buf` contains `length` valid ascii elements and has a total capacity
            //         of `capacity` elements, and that nothing else is using the momory.
            vec: unsafe { Vec::from_raw_parts(buf, length, capacity) },
        }
    }

    /// Converts a vector of bytes to an `AinString` without checking for non-ASCII characters.
    ///
    /// # Safety
    /// This function is unsafe because it does not check that the bytes passed to it are valid
    /// ASCII characters. If this constraint is violated, it may cause memory unsafety issues with
    /// future of the `AinString`, as the rest of this library assumes that `AinString`s are
    /// ASCII encoded.
    #[inline]
    #[must_use]
    pub unsafe fn from_ascii_unchecked<B>(bytes: B) -> Self
    where
        B: Into<Vec<u8>>,
    {
        let bytes = bytes.into();
        // SAFETY: The caller guarantees all bytes are valid ascii bytes.
        let (ptr, len, cap) = bytes.into_raw_parts();
        let ptr = ptr.cast::<AinChar>();

        // SAFETY: We guarantee all invariants, as we got the
        //         pointer, length and capacity from a `Vec`,
        //         and we also guarantee the pointer is valid per
        //         the `SAFETY` notice above.
        let vec = unsafe { Vec::from_raw_parts(ptr, len, cap) };

        Self { vec }
    }

    /// Converts anything that can represent a byte buffer into an `AinString`.
    ///
    /// # Errors
    /// Returns the byte buffer if not all of the bytes are ASCII characters.
    ///
    /// # Examples
    /// ```
    /// # use ain::AinString;
    /// let foo = AinString::from_ascii("foo".to_string()).unwrap();
    /// let err = AinString::from_ascii("Ŋ".to_string()).unwrap_err();
    /// assert_eq!(foo.as_str(), "foo");
    /// assert_eq!(err.into_source(), "Ŋ");
    /// ```
    pub fn from_ascii<B>(bytes: B) -> Result<AinString, IntoAinStringError<B>>
    where
        B: Into<Vec<u8>> + AsRef<[u8]>,
    {
        match bytes.as_ref().as_ascii_str() {
            // SAFETY: `as_ascii_str` guarantees all bytes are valid ascii bytes.
            Ok(_) => Ok(unsafe { AinString::from_ascii_unchecked(bytes) }),
            Err(e) => Err(IntoAinStringError {
                error: e,
                owner: bytes,
            }),
        }
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
        self.vec.extend(string.chars());
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
        self.vec.splice(idx..idx, string.into_iter().copied());
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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

    /// Returns the number of bytes in this ASCII string.
    ///
    /// # Examples
    /// ```
    /// # use ain::AinString;
    /// let s = AinString::from_ascii("foo").unwrap();
    /// assert_eq!(s.len(), 3);
    /// ```
    #[inline]
    #[must_use]
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
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
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

    /// Converts this [`AinString`] into a [`Box`]`<`[`AinStr`]`>`.
    ///
    /// This will drop any excess capacity
    #[inline]
    #[must_use]
    pub fn into_boxed_ain_str(self) -> Box<AinStr> {
        let slice = self.vec.into_boxed_slice();
        Box::from(slice)
    }
}

impl Deref for AinString {
    type Target = AinStr;

    #[inline]
    fn deref(&self) -> &AinStr {
        self.vec.as_slice().as_ref()
    }
}

impl DerefMut for AinString {
    #[inline]
    fn deref_mut(&mut self) -> &mut AinStr {
        self.vec.as_mut_slice().as_mut()
    }
}

macro_rules! impl_eq {
    ($lhs:ty, $rhs:ty) => {
        impl PartialEq<$rhs> for $lhs {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool {
                PartialEq::eq(&**self, &**other)
            }
        }
    };
}

impl_eq! { &AinStr, AinString }
impl_eq! { AinString, &AinStr }

impl Borrow<AinStr> for AinString {
    #[inline]
    fn borrow(&self) -> &AinStr {
        &**self
    }
}

impl BorrowMut<AinStr> for AinString {
    #[inline]
    fn borrow_mut(&mut self) -> &mut AinStr {
        &mut **self
    }
}

impl From<Vec<AinChar>> for AinString {
    #[inline]
    fn from(vec: Vec<AinChar>) -> Self {
        AinString { vec }
    }
}

impl From<AinChar> for AinString {
    #[inline]
    fn from(ch: AinChar) -> Self {
        AinString { vec: vec![ch] }
    }
}

impl From<AinString> for Vec<u8> {
    fn from(s: AinString) -> Vec<u8> {
        // SAFETY: All ascii bytes are valid `u8`, as we are `repr(u8)`.
        // Note: We forget `self` to avoid `self.vec` from being deallocated.
        let (ptr, len, cap) = s.vec.into_raw_parts();
        let ptr = ptr.cast::<u8>();

        // SAFETY: We guarantee all invariants due to getting `ptr`, `length`
        //         and `capacity` from a `Vec`. We also guarantee `ptr` is valid
        //         due to the `SAFETY` block above.
        unsafe { Vec::from_raw_parts(ptr, len, cap) }
    }
}

impl From<AinString> for AsciiString {
    #[inline]
    fn from(s: AinString) -> AsciiString {
        let vec: Vec<u8> = s.into();
        // SAFETY: AinString is always valid Ascii
        unsafe { AsciiString::from_ascii_unchecked(vec) }
    }
}

impl From<AsciiString> for AinString {
    #[inline]
    fn from(s: AsciiString) -> AinString {
        let vec: Vec<u8> = s.into();
        // SAFETY: AinString is always valid Ascii
        unsafe { AinString::from_ascii_unchecked(vec) }
    }
}

impl From<AinString> for Vec<AinChar> {
    fn from(s: AinString) -> Vec<AinChar> {
        s.vec
    }
}

impl<'a> From<&'a AinStr> for AinString {
    #[inline]
    fn from(s: &'a AinStr) -> Self {
        s.to_ain_string()
    }
}

impl<'a> From<&'a [AinChar]> for AinString {
    #[inline]
    fn from(s: &'a [AinChar]) -> AinString {
        s.iter().copied().collect()
    }
}

impl From<AinString> for String {
    #[inline]
    fn from(s: AinString) -> String {
        // SAFETY: All ascii bytes are `utf8`.
        unsafe { String::from_utf8_unchecked(s.into()) }
    }
}

impl From<Box<AinStr>> for AinString {
    #[inline]
    fn from(boxed: Box<AinStr>) -> Self {
        boxed.into_ain_string()
    }
}

impl From<AinString> for Box<AinStr> {
    #[inline]
    fn from(string: AinString) -> Self {
        string.into_boxed_ain_str()
    }
}

impl From<AinString> for Rc<AinStr> {
    fn from(s: AinString) -> Rc<AinStr> {
        let var: Rc<[AinChar]> = s.vec.into();
        // SAFETY: AinStr is repr(transparent) and thus has the same layout as [AinChar]
        unsafe { Rc::from_raw(Rc::into_raw(var) as *const AinStr) }
    }
}

impl From<AinString> for Arc<AinStr> {
    fn from(s: AinString) -> Arc<AinStr> {
        let var: Arc<[AinChar]> = s.vec.into();
        // SAFETY: AinStr is repr(transparent) and thus has the same layout as [AinChar]
        unsafe { Arc::from_raw(Arc::into_raw(var) as *const AinStr) }
    }
}

impl<'a> From<Cow<'a, AinStr>> for AinString {
    fn from(cow: Cow<'a, AinStr>) -> AinString {
        cow.into_owned()
    }
}

impl From<AinString> for Cow<'static, AinStr> {
    fn from(string: AinString) -> Cow<'static, AinStr> {
        Cow::Owned(string)
    }
}

impl<'a> From<&'a AinStr> for Cow<'a, AinStr> {
    fn from(s: &'a AinStr) -> Cow<'a, AinStr> {
        Cow::Borrowed(s)
    }
}

impl AsRef<AinStr> for AinString {
    #[inline]
    fn as_ref(&self) -> &AinStr {
        &**self
    }
}

impl AsRef<[AinChar]> for AinString {
    #[inline]
    fn as_ref(&self) -> &[AinChar] {
        &self.vec
    }
}

impl AsRef<[u8]> for AinString {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl AsRef<str> for AinString {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsMut<AinStr> for AinString {
    #[inline]
    fn as_mut(&mut self) -> &mut AinStr {
        &mut *self
    }
}

impl AsMut<[AinChar]> for AinString {
    #[inline]
    fn as_mut(&mut self) -> &mut [AinChar] {
        &mut self.vec
    }
}

impl FromStr for AinString {
    type Err = AsAinStrError;

    fn from_str(s: &str) -> Result<AinString, AsAinStrError> {
        s.as_ain_str().map(AinStr::to_ain_string)
    }
}

impl fmt::Display for AinString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl fmt::Debug for AinString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

/// Please note that the `std::fmt::Result` returned by these methods does not support
/// transmission of an error other than that an error occurred.
impl fmt::Write for AinString {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if let Ok(astr) = AinStr::from_ascii(s) {
            self.push_str(astr);
            Ok(())
        } else {
            Err(fmt::Error)
        }
    }

    fn write_char(&mut self, c: char) -> fmt::Result {
        if let Ok(achar) = AinChar::from_ascii(c) {
            self.push(achar);
            Ok(())
        } else {
            Err(fmt::Error)
        }
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

#[allow(clippy::indexing_slicing)] // In `Index`, if it's out of bounds, panic is the default
impl<T> Index<T> for AinString
where
    AinStr: Index<T>,
{
    type Output = <AinStr as Index<T>>::Output;

    #[inline]
    fn index(&self, index: T) -> &<AinStr as Index<T>>::Output {
        &(**self)[index]
    }
}

#[allow(clippy::indexing_slicing)] // In `IndexMut`, if it's out of bounds, panic is the default
impl<T> IndexMut<T> for AinString
where
    AinStr: IndexMut<T>,
{
    #[inline]
    fn index_mut(&mut self, index: T) -> &mut <AinStr as Index<T>>::Output {
        &mut (**self)[index]
    }
}

/// A possible error value when converting into an  `AinString` from a byte vector or string.
/// It wraps an `AsAinStrError` which you can get through the `ain_error()` method.
///
/// This is the error type for `AinString::from_ascii()` and
/// `IntoAinString::into_ain_string()`. They will never clone or touch the content of the
/// original type; It can be extracted by the `into_source` method.
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
    error: AsAinStrError,
    owner: O,
}
impl<O> IntoAinStringError<O> {
    /// Get the position of the first non-ASCII byte or character.
    #[inline]
    #[must_use]
    pub fn ain_error(&self) -> AsAinStrError {
        self.error
    }
    /// Get back the original, unmodified type.
    #[inline]
    #[must_use]
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
#[cfg(feature = "std")]
impl<O: Any> Error for IntoAinStringError<O> {
    #[inline]
    #[allow(deprecated)] // TODO: Remove deprecation once the earliest version we support deprecates this method.
    fn description(&self) -> &str {
        self.error.description()
    }
    /// Always returns an `AsAinStrError`
    fn cause(&self) -> Option<&dyn Error> {
        Some(&self.error as &dyn Error)
    }
}
