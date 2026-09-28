use core::ascii::Char as AsciiChar;

use crate::AinChar;

const _: () = {
    assert!(core::mem::size_of::<AinChar>() == core::mem::size_of::<AsciiChar>());
    assert!(core::mem::align_of::<AinChar>() == core::mem::align_of::<AsciiChar>());
};

impl AinChar {
    /// Converts self to a [std::ascii::Char][AsciiChar].
    #[must_use]
    #[inline]
    pub const fn to_std_ascii(self) -> AsciiChar {
        // SAFETY: both AsciiChar and AinChar have the exact same repr and range
        unsafe { AsciiChar::from_u8_unchecked(self.to_u8()) }
    }

    /// Converts a [std::ascii::Char][AsciiChar] to an [AinChar].
    #[must_use]
    #[inline]
    pub const fn from_std_ascii(char: AsciiChar) -> Self {
        // SAFETY: both AsciiChar and AinChar have the exact same repr and range
        unsafe { Self::from_u8_unchecked(char.to_u8()) }
    }
}

impl AsRef<AsciiChar> for AinChar {
    #[inline]
    fn as_ref(&self) -> &AsciiChar {
        // SAFETY: pointer is from a reference, both types have identical repr and valid range.
        unsafe { &*(self as *const AinChar as *const AsciiChar) }
    }
}

impl AsMut<AsciiChar> for AinChar {
    #[inline]
    fn as_mut(&mut self) -> &mut AsciiChar {
        // SAFETY: pointer is from a reference, both types have identical repr and valid range.
        unsafe { &mut *(self as *mut AinChar as *mut AsciiChar) }
    }
}

impl AsRef<AinChar> for AsciiChar {
    #[inline]
    fn as_ref(&self) -> &AinChar {
        // SAFETY: pointer is from a reference, both types have identical repr and valid range.
        unsafe { &*(self as *const AsciiChar as *const AinChar) }
    }
}

impl AsMut<AinChar> for AsciiChar {
    #[inline]
    fn as_mut(&mut self) -> &mut AinChar {
        // SAFETY: pointer is from a reference, both types have identical repr and valid range.
        unsafe { &mut *(self as *mut AsciiChar as *mut AinChar) }
    }
}

impl From<AsciiChar> for AinChar {
    #[inline]
    fn from(value: AsciiChar) -> Self {
        Self::from_std_ascii(value)
    }
}

impl From<AinChar> for AsciiChar {
    #[inline]
    fn from(value: AinChar) -> Self {
        value.to_std_ascii()
    }
}
