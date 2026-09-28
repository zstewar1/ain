use core::cmp::Ordering;
use core::hash::{Hash, Hasher};
use core::{fmt, mem};

use crate::AinStr;
use crate::repr::AinCharRepr;

/// An ASCII-only character that is natively case-insensitive.
///
/// It can be created from a literal with the [`ain!`] macro or by checked conversion from `char` or
/// `u8`.
///
/// We don't allow comparisons with case-sensitive types including u8 and char because case handling
/// could be ambiguous and would break transitive equality. E.g. `'a' == ain!('a')`, `ain!('a') ==
/// ain!('A')`, and `ain!('A') == 'A'` would be true regardless of how we handled equality, but `'a'
/// != 'A'` even though every equality step in the middle is true.
///
/// For Ord, the 'case insensitive' order is to treat all characters as uppercase. This affects the
/// sort order of letters relative to the following symbols, which lie between uppercase and
/// lowercase ascii: `` [\]^_` ``
///
/// ## Repr
///
/// `AinChar` is guaranteed to have a size and align of 1 byte, and to have a valid value range of
/// `0..128`. It is safe to transmute or pointer cast any size 1 + align 1 value to `AinChar`
/// provided its `u8` value is < 128. Transmuting or casting any byte value >= 128 to `AinChar` is
/// immediate Undefined Behavior.
#[derive(Default, Copy, Clone)]
#[repr(transparent)]
pub struct AinChar {
    /// The char that this AinChar represents. We use an enum to get restricted range (enables niche
    /// optimization) over a u8, but we keep the enum private to give us flexibility with respect to
    /// public API. Additionally, pattern matching on an enum would be case-sensitive, which is
    /// confusing when every other usage is case-insensitive.
    char: crate::repr::AinCharRepr,
}

const _: () = {
    assert!(core::mem::size_of::<AinChar>() == 1, "AinChar mis-sized!");
    assert!(core::mem::align_of::<AinChar>() == 1, "AinChar mis-aligned");
};

impl fmt::Debug for AinChar {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.to_char(), f)
    }
}

impl fmt::Display for AinChar {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.to_char(), f)
    }
}

impl PartialEq for AinChar {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        (*self).eq(*other)
    }
}

impl Eq for AinChar {}

impl Hash for AinChar {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        // We normalize to uppercase since those are numerically lower and so sort before some
        // punctuation marks that normally end up before lowercase ascii.
        (self.char as u8).to_ascii_uppercase().hash(state);
    }

    #[inline]
    fn hash_slice<H: Hasher>(data: &[Self], state: &mut H) {
        const BLOCK_SIZE: usize = 64;
        let mut buf = [0u8; BLOCK_SIZE];
        let (chunks, tail) = data.as_chunks::<BLOCK_SIZE>();
        for chunk in chunks {
            for idx in 0..BLOCK_SIZE {
                buf[idx] = chunk[idx].to_u8().to_ascii_uppercase();
            }
            state.write(&buf);
        }
        if !tail.is_empty() {
            for (idx, value) in tail.iter().enumerate() {
                // tail.len() is always less than BLOCK_SIZE.
                buf[idx] = value.to_u8().to_ascii_uppercase();
            }
            state.write(&buf[..tail.len()])
        }
    }
}

impl PartialOrd for AinChar {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(Ord::cmp(self, other))
    }
}

impl Ord for AinChar {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        // We normalize to uppercase since those are numerically lower and so sort before some
        // punctuation marks that normally end up before lowercase ascii.
        self.to_uppercase()
            .to_u8()
            .cmp(&other.to_uppercase().to_u8())
    }
}

impl AinChar {
    /// The character with the lowest ASCII code. (Null = 0)
    pub const MIN: Self = Self {
        char: AinCharRepr::Null,
    };

    /// The character with the highest ASCII code. (Delete = 127)
    pub const MAX: Self = Self {
        char: AinCharRepr::Delete,
    };

    /// Converts the given character to an AinChar if it is in range.
    #[must_use]
    #[inline]
    pub const fn from_char(ch: char) -> Option<Self> {
        if ch <= Self::MAX.to_char() {
            Some(unsafe { Self::from_u8_unchecked(ch as u8) })
        } else {
            None
        }
    }

    /// Converts a bytes aon an AinChar if it is in range.
    #[must_use]
    #[inline]
    pub const fn from_u8(b: u8) -> Option<Self> {
        if b <= Self::MAX.to_u8() {
            // SAFETY: we just checked that the byte is in the valid range.
            Some(unsafe { Self::from_u8_unchecked(b) })
        } else {
            None
        }
    }

    /// Creates an ASCII character from the byte `b`, without checking whether it's valid.
    ///
    /// # Safety
    ///
    /// `b` must be in `0..=127`, or else this is UB.
    #[must_use]
    #[inline]
    pub const unsafe fn from_u8_unchecked(b: u8) -> Self {
        // The safety precondition is trivially verifiable, so we check it in debug builds.
        debug_assert!(
            b <= Self::MAX.to_u8(),
            "`ain::AinChar::from_u8_unchecked` input cannot exceed 127.",
        );
        // SAFETY: Our safety precondition is that `b` is in-range.
        unsafe { mem::transmute::<u8, Self>(b) }
    }

    /// Converts numbers 0-9 into the digit characters '0' - '9'
    #[must_use]
    #[inline]
    pub const fn digit(d: u8) -> Option<AinChar> {
        if d < 10 {
            // SAFETY: Just checked that the value is in range.
            Some(unsafe { Self::digit_unchecked(d) })
        } else {
            None
        }
    }

    /// When passed the *number* `0`, `1`, …, `9`, returns the *character* `'0'`, `'1'`, …, `'9'`
    /// respectively, without checking that it's in-range.
    ///
    /// # Safety
    ///
    /// Value must be less than 10.
    #[must_use]
    #[inline]
    pub const unsafe fn digit_unchecked(d: u8) -> AinChar {
        // SAFETY: `'0'` through `'9'` are U+00030 through U+0039, so because `d` must be less than
        // 10 the addition can return at most 57 (0x39), which doesn't overflow and is within the
        // ASCII range.
        unsafe {
            let ch = b'0'.unchecked_add(d);
            Self::from_u8_unchecked(ch)
        }
    }

    /// Converts an ASCII character into a `u8`.
    #[must_use]
    #[inline]
    pub const fn to_u8(self) -> u8 {
        self.char as u8
    }

    /// Converts an ASCII character into a `char`.
    #[must_use]
    #[inline]
    pub const fn to_char(self) -> char {
        self.to_u8() as char
    }

    /// Views this ASCII character as a one-character string.
    #[must_use]
    #[inline]
    pub const fn as_str(&self) -> &str {
        self.as_ain_str().as_str()
    }

    /// View this AinChar as a single-character AinStr
    #[must_use]
    #[inline]
    pub const fn as_ain_str(&self) -> &AinStr {
        AinStr::from_slice(core::slice::from_ref(self))
    }

    /// View this AinChar as a mutable single-character AinStr
    #[must_use]
    #[inline]
    pub const fn as_mut_ain_str(&mut self) -> &mut AinStr {
        AinStr::from_slice_mut(core::slice::from_mut(self))
    }

    /// Maps letters a-z to A-Z and returns any other character unchanged.
    ///
    /// Note: because `AinChar` is not case sensitive, the result will always compare equal to the
    /// input.
    ///
    /// # Examples
    /// ```
    /// # use ain::ain;
    /// assert_eq!(ain!('u').to_uppercase().to_char(), 'U');
    /// assert_eq!(ain!('U').to_uppercase().to_char(), 'U');
    /// assert_eq!(ain!('2').to_uppercase().to_char(), '2');
    /// assert_eq!(ain!('=').to_uppercase().to_char(), '=');
    /// assert_eq!(ain!('[').to_uppercase().to_char(), '[');
    /// ```
    #[must_use]
    #[inline]
    pub const fn to_uppercase(self) -> Self {
        // SAFETY: case conversion does not make valid ascii into invalid ascii
        unsafe { Self::from_u8_unchecked(self.to_u8().to_ascii_uppercase()) }
    }

    /// Maps letters A-Z to a-z and returns any other character unchanged.
    ///
    /// Note: because `AinChar` is not case sensitive, the result will always compare equal to the
    /// input.
    ///
    /// # Examples
    /// ```
    /// # use ain::ain;
    /// assert_eq!(ain!('U').to_lowercase().to_char(), 'u');
    /// assert_eq!(ain!('u').to_lowercase().to_char(), 'u');
    /// assert_eq!(ain!('2').to_lowercase().to_char(), '2');
    /// assert_eq!(ain!('^').to_lowercase().to_char(), '^');
    /// assert_eq!(ain!('\x7f').to_lowercase().to_char(), '\x7f');
    /// ```
    #[must_use]
    #[inline]
    pub const fn to_lowercase(self) -> Self {
        // SAFETY: case conversion does not make valid ascii into invalid ascii
        unsafe { Self::from_u8_unchecked(self.to_u8().to_ascii_lowercase()) }
    }

    /// Replaces letters `a` to `z` with `A` to `Z`
    ///
    /// Note: because this type is case-insensitive, this does not change the equals or hash.
    #[inline]
    pub const fn make_uppercase(&mut self) {
        *self = self.to_uppercase()
    }

    /// Replaces letters `A` to `Z` with `a` to `z`
    ///
    /// Note: because this type is case-insensitive, this does not change the equals or hash.
    #[inline]
    pub const fn make_lowercase(&mut self) {
        *self = self.to_lowercase();
    }

    /// Check if the character is a letter (a-z, A-Z)
    #[must_use]
    #[inline]
    pub const fn is_alphabetic(self) -> bool {
        self.to_u8().is_ascii_alphabetic()
    }

    /// Checks if the character is alphabetic and uppercase (A-Z).
    ///
    /// # Examples
    /// ```
    /// # use ain::ain;
    /// assert_eq!(ain!('A').is_uppercase(), true);
    /// assert_eq!(ain!('a').is_uppercase(), false);
    /// assert_eq!(ain!('@').is_uppercase(), false);
    /// ```
    #[must_use]
    #[inline]
    pub const fn is_uppercase(self) -> bool {
        self.to_u8().is_ascii_uppercase()
    }

    /// Checks if the character is alphabetic and lowercase (a-z).
    #[must_use]
    #[inline]
    pub const fn is_lowercase(self) -> bool {
        self.to_u8().is_ascii_lowercase()
    }

    /// Check if the character is a letter or decimal digit.
    #[must_use]
    #[inline]
    pub const fn is_alphanumeric(self) -> bool {
        self.to_u8().is_ascii_alphanumeric()
    }

    /// Check if the character is a number (0-9)
    ///
    /// # Examples
    /// ```
    /// # use ain::ain;
    /// assert_eq!(ain!('0').is_digit(), true);
    /// assert_eq!(ain!('9').is_digit(), true);
    /// assert_eq!(ain!('a').is_digit(), false);
    /// assert_eq!(ain!('A').is_digit(), false);
    /// assert_eq!(ain!('/').is_digit(), false);
    /// ```
    #[must_use]
    #[inline]
    pub const fn is_digit(self) -> bool {
        self.to_u8().is_ascii_digit()
    }

    /// Checks if the character is a valid octal digit
    ///
    /// # Examples
    /// ```
    /// # use ain::ain;
    /// assert_eq!(ain!('5').is_octdigit(), true);
    /// assert_eq!(ain!('a').is_octdigit(), false);
    /// assert_eq!(ain!('F').is_octdigit(), false);
    /// assert_eq!(ain!('8').is_octdigit(), false);
    /// assert_eq!(ain!('G').is_octdigit(), false);
    /// assert_eq!(ain!(' ').is_octdigit(), false);
    /// ```
    #[must_use]
    #[inline]
    pub const fn is_octdigit(self) -> bool {
        self.is_digit() && self.to_u8() < b'8'
    }

    /// Checks if the character is a valid hex digit
    ///
    /// # Examples
    /// ```
    /// # use ain::ain;
    /// assert_eq!(ain!('5').is_hexdigit(), true);
    /// assert_eq!(ain!('a').is_hexdigit(), true);
    /// assert_eq!(ain!('F').is_hexdigit(), true);
    /// assert_eq!(ain!('G').is_hexdigit(), false);
    /// assert_eq!(ain!(' ').is_hexdigit(), false);
    /// ```
    #[must_use]
    #[inline]
    pub const fn is_hexdigit(self) -> bool {
        self.to_u8().is_ascii_hexdigit()
    }

    /// Checks if the character is punctuation
    ///
    /// # Examples
    /// ```
    /// # use ain::ain;
    /// assert_eq!(ain!('n').is_punctuation(), false);
    /// assert_eq!(ain!(' ').is_punctuation(), false);
    /// assert_eq!(ain!('_').is_punctuation(), true);
    /// assert_eq!(ain!('~').is_punctuation(), true);
    /// ```
    #[must_use]
    #[inline]
    pub const fn is_punctuation(self) -> bool {
        self.to_u8().is_ascii_punctuation()
    }

    /// Checks if the character is printable (except space)
    ///
    /// # Examples
    /// ```
    /// # use ain::ain;
    /// assert_eq!(ain!('n').is_graphic(), true);
    /// assert_eq!(ain!(' ').is_graphic(), false);
    /// assert_eq!(ain!('\n').is_graphic(), false);
    /// ```
    #[must_use]
    #[inline]
    pub const fn is_graphic(self) -> bool {
        self.to_u8().is_ascii_graphic()
    }

    /// Checks if the value is an ASCII whitespace character: U+0020 SPACE, U+0009 HORIZONTAL TAB,
    /// U+000A LINE FEED, U+000C FORM FEED, or U+000D CARRIAGE RETURN.
    ///
    /// **Warning:** Because the list above excludes U+000B VERTICAL TAB, `c.is_whitespace()`
    /// is **not** equivalent to `c.to_char().is_whitespace()`.
    ///
    /// Rust uses the WhatWG Infra Standard's [definition of ASCII whitespace][infra-aw]. There are
    /// several other definitions in wide use. For instance, [the POSIX locale][pct] includes U+000B
    /// VERTICAL TAB as well as all the above characters, but—from the very same specification—[the
    /// default rule for "field splitting" in the Bourne shell][bfs] considers *only* SPACE,
    /// HORIZONTAL TAB, and LINE FEED as whitespace.
    ///
    /// If you are writing a program that will process an existing file format, check what that
    /// format's definition of whitespace is before using this function.
    ///
    /// [infra-aw]: https://infra.spec.whatwg.org/#ascii-whitespace
    /// [pct]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap07.html#tag_07_03_01
    /// [bfs]: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html#tag_18_06_05
    #[must_use]
    #[inline]
    pub const fn is_whitespace(self) -> bool {
        self.to_u8().is_ascii_whitespace()
    }

    /// Check if the character is a control character
    ///
    /// # Examples
    /// ```
    /// # use ain::ain;
    /// assert_eq!(ain!('\0').is_control(), true);
    /// assert_eq!(ain!('n').is_control(), false);
    /// assert_eq!(ain!(' ').is_control(), false);
    /// assert_eq!(ain!('\n').is_control(), true);
    /// assert_eq!(ain!('\t').is_control(), true);
    /// assert_eq!(ain!('\x04').is_control(), true);
    /// ```
    #[must_use]
    #[inline]
    pub const fn is_control(self) -> bool {
        self.to_u8().is_ascii_control()
    }

    /// Provides a const way to perform equality on AinChar values.
    #[must_use]
    #[inline]
    pub(crate) const fn eq(self, other: Self) -> bool {
        self.to_u8().eq_ignore_ascii_case(&other.to_u8())
    }
}

impl AsRef<AinStr> for AinChar {
    #[inline]
    fn as_ref(&self) -> &AinStr {
        self.as_ain_str()
    }
}

impl AsMut<AinStr> for AinChar {
    #[inline]
    fn as_mut(&mut self) -> &mut AinStr {
        self.as_mut_ain_str()
    }
}

impl AsRef<str> for AinChar {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<AinChar> for char {
    #[inline]
    fn from(value: AinChar) -> Self {
        value.to_char()
    }
}

impl From<AinChar> for u8 {
    #[inline]
    fn from(value: AinChar) -> Self {
        value.to_u8()
    }
}

/// Error used for [TryFrom]/[TryInto] implementations for [AinChar].
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TryIntoAinCharError {
    _private: (),
}

impl fmt::Display for TryIntoAinCharError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Input was not a valid ASCII character")
    }
}

impl core::error::Error for TryIntoAinCharError {}

impl TryFrom<u8> for AinChar {
    type Error = TryIntoAinCharError;

    #[inline]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::from_u8(value).ok_or(TryIntoAinCharError { _private: () })
    }
}

impl TryFrom<char> for AinChar {
    type Error = TryIntoAinCharError;

    #[inline]
    fn try_from(value: char) -> Result<Self, Self::Error> {
        Self::from_char(value).ok_or(TryIntoAinCharError { _private: () })
    }
}

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use alloc::collections::BTreeSet;
    use hashbrown::HashSet;

    use super::AinChar;

    #[test]
    fn hash_case_independent() {
        let mut set = HashSet::new();
        for ch in '\0'..='\x7f' {
            let inch = AinChar::from_char(ch).unwrap();
            // ASCII lowercase is higher than lowercase, so we expect the lowercase letters to all
            // be skipped.
            if ch.is_ascii_lowercase() {
                let existing: AinChar = *set.get(&inch).unwrap();
                assert_eq!(existing.to_char(), ch.to_ascii_uppercase());
            } else {
                assert!(set.insert(inch));
            }
        }
        assert_eq!(set.len(), 102);
    }

    #[test]
    fn cmp_case_independent() {
        let mut set = BTreeSet::new();
        for ch in '\0'..='\x7f' {
            let inch = AinChar::from_char(ch).unwrap();
            // ASCII lowercase is higher than lowercase, so we expect the lowercase letters to all
            // be skipped.
            if ch.is_ascii_lowercase() {
                let existing: AinChar = *set.get(&inch).unwrap();
                assert_eq!(existing.to_char(), ch.to_ascii_uppercase());
            } else {
                assert!(set.insert(inch));
            }
        }
        assert_eq!(set.len(), 102);
    }
}
