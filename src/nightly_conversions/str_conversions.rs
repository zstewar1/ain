use core::ascii::Char as AsciiChar;

use crate::{AinChar, AinStr};

impl AinStr {
    /// Converts &self into a slice of std ascii chars.
    #[inline]
    pub const fn as_std_ascii(&self) -> &[AsciiChar] {
        // SAFETY: both AsciiChar and AinChar have the exact same repr and range, so its safe to
        // convert between them. The pointer comes from a reference so it is safe to dereference.
        // The lifetime is preserved.
        unsafe { &*(self.as_slice() as *const [AinChar] as *const [AsciiChar]) }
    }

    /// Converts &mut self into a mutable slice of std ascii chars.
    ///
    /// Since both types have the exact same range, it is always safe to view one as the other and
    /// replacing AsciiChars in the output never invalidates the AinStr.
    #[inline]
    pub const fn as_std_ascii_mut(&mut self) -> &mut [AsciiChar] {
        // SAFETY: both AsciiChar and AinChar have the exact same repr and range, so its safe to
        // convert between them. The pointer comes from a reference so it is safe to dereference.
        // The lifetime is preserved.
        unsafe { &mut *(self.as_mut_slice() as *mut [AinChar] as *mut [AsciiChar]) }
    }

    /// Converts a slice of std ascii chars into an AinStr
    #[inline]
    pub const fn from_std_ascii(slice: &[AsciiChar]) -> &Self {
        // SAFETY: both AsciiChar and AinChar have the exact same repr and range, so its safe to
        // convert between them. The pointer comes from a reference so it is safe to dereference.
        // The lifetime is preserved.
        Self::from_slice(unsafe { &*(slice as *const [AsciiChar] as *const [AinChar]) })
    }

    /// Converts a mutable slice of std ascii chars into an &mut AinStr
    ///
    /// Since both types have the exact same range, it is always safe to view one as the other and
    /// replacing AsciiChars in the output never invalidates the AinStr.
    #[inline]
    pub const fn from_std_ascii_mut(slice: &mut [AsciiChar]) -> &mut Self {
        // SAFETY: both AsciiChar and AinChar have the exact same repr and range, so its safe to
        // convert between them. The pointer comes from a reference so it is safe to dereference.
        // The lifetime is preserved.
        Self::from_mut_slice(unsafe { &mut *(slice as *mut [AsciiChar] as *mut [AinChar]) })
    }
}

impl AsRef<[AsciiChar]> for AinStr {
    #[inline]
    fn as_ref(&self) -> &[AsciiChar] {
        self.as_std_ascii()
    }
}

impl AsMut<[AsciiChar]> for AinStr {
    #[inline]
    fn as_mut(&mut self) -> &mut [AsciiChar] {
        self.as_std_ascii_mut()
    }
}

from_ref_from_as_ref!(AinStr, [AsciiChar]);

impl AsRef<AinStr> for [AsciiChar] {
    #[inline]
    fn as_ref(&self) -> &AinStr {
        AinStr::from_std_ascii(self)
    }
}

impl AsMut<AinStr> for [AsciiChar] {
    #[inline]
    fn as_mut(&mut self) -> &mut AinStr {
        AinStr::from_std_ascii_mut(self)
    }
}

from_ref_from_as_ref!([AsciiChar], AinStr);
