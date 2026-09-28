use crate::{AinChar, AinStr};
use ascii::{AsciiChar, AsciiStr};

impl AinChar {
    /// Converts self to an [AsciiChar].
    #[inline]
    pub fn to_ascii_char(self) -> AsciiChar {
        // SAFETY: both AsciiChar and AinChar have the exact same range
        unsafe { AsciiChar::from_ascii_unchecked(self.to_u8()) }
    }

    /// Converts an [AsciiChar] to case-insensitive.
    #[inline]
    pub const fn from_ascii_char(value: AsciiChar) -> Self {
        // SAFETY: both AsciiChar and AinChar have the exact same range
        unsafe { Self::from_u8_unchecked(value.as_byte()) }
    }
}

// AsRef/AsMut

// AinChar to AsciiChar/AsciiStr
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

impl AsRef<AsciiStr> for AinChar {
    #[inline]
    fn as_ref(&self) -> &AsciiStr {
        let char: &[AsciiChar] = core::slice::from_ref(self.as_ref());
        <&AsciiStr>::from(char)
    }
}

impl AsMut<AsciiStr> for AinChar {
    #[inline]
    fn as_mut(&mut self) -> &mut AsciiStr {
        let char: &mut [AsciiChar] = core::slice::from_mut(self.as_mut());
        <&mut AsciiStr>::from(char)
    }
}

// AsciiChar to AinChar/AinStr
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

impl AsRef<AinStr> for AsciiChar {
    #[inline]
    fn as_ref(&self) -> &AinStr {
        let chars: &[AinChar] = core::slice::from_ref(self.as_ref());
        AinStr::from_slice(chars)
    }
}

impl AsMut<AinStr> for AsciiChar {
    #[inline]
    fn as_mut(&mut self) -> &mut AinStr {
        let char: &mut [AinChar] = core::slice::from_mut(self.as_mut());
        AinStr::from_mut_slice(char)
    }
}

// From

// char to char
impl From<AsciiChar> for AinChar {
    #[inline]
    fn from(value: AsciiChar) -> Self {
        Self::from_ascii_char(value)
    }
}

impl From<AinChar> for AsciiChar {
    #[inline]
    fn from(value: AinChar) -> Self {
        value.to_ascii_char()
    }
}
