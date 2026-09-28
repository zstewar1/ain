use alloc::borrow::{Cow, ToOwned};
use alloc::boxed::Box;

use crate::{AinChar, AinStr};

use crate::AinString;

impl AinChar {
    /// Creates an AinString containing this single character.
    #[must_use]
    #[inline]
    pub fn to_ain_string(self) -> AinString {
        let mut s = AinString::with_capacity(1);
        s.push(self);
        s
    }
}

impl From<AinChar> for AinString {
    fn from(value: AinChar) -> Self {
        value.to_ain_string()
    }
}

impl AinStr {
    /// Copies the content of this `AinStr` into an owned `AinString`.
    #[must_use]
    #[inline]
    pub fn to_ain_string(&self) -> AinString {
        AinString::from_vec(self.slice.to_vec())
    }

    /// Returns a copy of this string where letters 'a' to 'z' are mapped to 'A' to 'Z'.
    #[must_use]
    #[inline]
    pub fn to_uppercase(&self) -> AinString {
        let mut ain_string = self.to_ain_string();
        ain_string.make_uppercase();
        ain_string
    }

    /// Returns a copy of this string where letters 'A' to 'Z' are mapped to 'a' to 'z'.
    #[must_use]
    #[inline]
    pub fn to_lowercase(&self) -> AinString {
        let mut ain_string = self.to_ain_string();
        ain_string.make_lowercase();
        ain_string
    }

    /// Converts a [`Box<AinStr>`] into a [`Box<[AinChar]>`] without copying or allocating.
    #[must_use]
    #[inline]
    pub fn into_boxed_slice(self: Box<Self>) -> Box<[AinChar]> {
        // SAFETY: AinStr is repr(transparent) to [AinChar] so this pointer cast is safe. The rest of
        // the box preconditions are trivially satisfied since we got the pointer from a Box.
        let ptr = Box::into_raw(self) as *mut [AinChar];
        unsafe { Box::from_raw(ptr) }
    }

    /// Converts a [`Box<AinStr>`] into a [`AinString`] without copying or allocating.
    #[must_use]
    #[inline]
    pub fn into_ain_string(self: Box<Self>) -> AinString {
        AinString::from_boxed_ain_str(self)
    }
}

impl From<&AinStr> for AinString {
    #[inline]
    fn from(value: &AinStr) -> Self {
        value.to_ain_string()
    }
}

impl<'a> From<&'a AinStr> for Cow<'a, AinStr> {
    #[inline]
    fn from(value: &'a AinStr) -> Self {
        Cow::Borrowed(value)
    }
}

impl ToOwned for AinStr {
    type Owned = AinString;

    #[inline]
    fn to_owned(&self) -> AinString {
        self.to_ain_string()
    }
}
