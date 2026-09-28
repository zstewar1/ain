use core::ascii::Char as AsciiChar;

use alloc::vec::Vec;

use crate::{AinChar, AinString};

impl AinString {
    /// Converts self into a vector of std ascii chars without copying or allocating.
    #[must_use]
    #[inline]
    pub fn into_std_ascii(self) -> Vec<AsciiChar> {
        let (ptr, len, cap) = self.into_raw_parts();
        // SAFETY: both AsciiChar and AinChar have the exact same repr and range, so its safe to
        // convert between them. The pointer comes from a vector so it is safe to convert back to a
        // vector with the same len and cap.
        let ptr = ptr.cast::<AsciiChar>();
        unsafe { Vec::from_raw_parts(ptr, len, cap) }
    }

    /// Converts a vector of std ascii chars into an AinString without copying or allocating.
    #[must_use]
    #[inline]
    pub fn from_std_ascii(vec: Vec<AsciiChar>) -> Self {
        let (ptr, len, cap) = vec.into_raw_parts();
        // SAFETY: both AsciiChar and AinChar have the exact same repr and range, so its safe to
        // convert between them. The pointer comes from a vector so it is safe to convert back to a
        // vector with the same len and cap.
        let ptr = ptr.cast::<AinChar>();
        unsafe { Self::from_raw_parts(ptr, len, cap) }
    }
}

impl From<AinString> for Vec<AsciiChar> {
    #[inline]
    fn from(value: AinString) -> Self {
        value.into_std_ascii()
    }
}

impl From<Vec<AsciiChar>> for AinString {
    #[inline]
    fn from(value: Vec<AsciiChar>) -> Self {
        Self::from_std_ascii(value)
    }
}
