use alloc::vec::Vec;

use ascii::{AsciiChar, AsciiString};

use crate::{AinChar, AinString};

impl AinString {
    /// Converts this AinString into an [AsciiString] without copying or allocating.
    #[must_use]
    #[inline]
    pub fn into_ascii_string(self) -> AsciiString {
        let (ptr, len, cap) = self.into_raw_parts();
        // SAFETY: both AsciiChar and AinChar have the exact same repr and range, so its safe to
        // convert between them. The pointer comes from a vector so it is safe to convert back to a
        // vector with the same len and cap.
        let ptr = ptr.cast::<AsciiChar>();
        unsafe { AsciiString::from_raw_parts(ptr, len, cap) }
    }

    /// Converts an [AsciiString] into an `AinString` without copying or allocating.
    #[must_use]
    #[inline]
    pub fn from_ascii_string(s: AsciiString) -> Self {
        // AsciiString doesn't have into_raw_parts and its as_mut_ptr supplies a slice-derived
        // pointer, which with tree borrows is only valid for len, not cap. We can convert through
        // Vec to ensure we are technically meeting provenance requirements.
        let (ptr, len, cap) = Vec::<AsciiChar>::from(s).into_raw_parts();
        // SAFETY: both AsciiChar and AinChar have the exact same repr and range, so its safe to
        // convert between them. The pointer comes from a vector so it is safe to convert back to a
        // vector with the same len and cap.
        let ptr = ptr.cast::<AinChar>();
        unsafe { Self::from_raw_parts(ptr, len, cap) }
    }
}

impl From<AinString> for AsciiString {
    #[inline]
    fn from(value: AinString) -> Self {
        value.into_ascii_string()
    }
}

impl From<AsciiString> for AinString {
    #[inline]
    fn from(value: AsciiString) -> Self {
        Self::from_ascii_string(value)
    }
}
