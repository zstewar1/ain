use crate::{AinChar, AinStr};
use ascii::{AsciiChar, AsciiStr};

impl AinStr {
    /// Gets a view of self as an [AsciiStr] slice.
    #[inline]
    pub fn as_ascii_str(&self) -> &AsciiStr {
        // SAFETY: AinChar and AsciiChar have the same repr and range, so casting between them
        // is safe.
        let ptr = self.as_slice() as *const [AinChar] as *const [AsciiChar];
        let slice = unsafe { &*ptr };
        <&AsciiStr>::from(slice)
    }

    /// Gets a view of self as a mutable [AsciiStr] slice.
    #[inline]
    pub fn as_ascii_str_mut(&mut self) -> &mut AsciiStr {
        // SAFETY: AinChar and AsciiChar have the same repr and range, so casting between them
        // is safe.
        let ptr = self.as_mut_slice() as *mut [AinChar] as *mut [AsciiChar];
        let slice = unsafe { &mut *ptr };
        <&mut AsciiStr>::from(slice)
    }

    /// Creates an AinStr slice from an [AsciiStr] slice.
    pub const fn from_ascii_str(value: &AsciiStr) -> &Self {
        // SAFETY: AinChar and AsciiChar have the same repr and range, so casting between them
        // is safe.
        let ptr = value.as_slice() as *const [AsciiChar] as *const [AinChar];
        let slice = unsafe { &*ptr };
        AinStr::from_slice(slice)
    }

    /// Creates a mutable AinStr slice from a mutable [AsciiStr] slice.
    pub fn from_ascii_str_mut(value: &mut AsciiStr) -> &mut Self {
        // SAFETY: AinChar and AsciiChar have the same repr and range, so casting between them
        // is safe.
        let ptr = value.as_mut_slice() as *mut [AsciiChar] as *mut [AinChar];
        let slice = unsafe { &mut *ptr };
        AinStr::from_mut_slice(slice)
    }
}

// AinStr to AsciiStr
impl AsRef<AsciiStr> for AinStr {
    #[inline]
    fn as_ref(&self) -> &AsciiStr {
        self.as_ascii_str()
    }
}

impl AsMut<AsciiStr> for AinStr {
    #[inline]
    fn as_mut(&mut self) -> &mut AsciiStr {
        self.as_ascii_str_mut()
    }
}

from_ref_from_as_ref!(AsciiStr, AinStr);

// AsciiChar to AinChar/AinStr
impl AsRef<AinStr> for AsciiStr {
    #[inline]
    fn as_ref(&self) -> &AinStr {
        AinStr::from_ascii_str(self)
    }
}

impl AsMut<AinStr> for AsciiStr {
    #[inline]
    fn as_mut(&mut self) -> &mut AinStr {
        AinStr::from_ascii_str_mut(self)
    }
}

from_ref_from_as_ref!(AinStr, AsciiStr);
