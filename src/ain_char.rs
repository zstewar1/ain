use core::cmp::Ordering;
use core::hash::{Hash, Hasher};
use core::{fmt, mem};

use crate::AinStr;

/// An ASCII-only character that is natively case-insensitive. Mimics the unstable standard library
/// type [std::ascii::Char].
///
/// This type is `repr(u8)`.
///
/// Unlike other character types, we don't allow comparisons with case-sensitive types including u8
/// and char because case handling could be ambiguous and would break transitive equality. E.g. `'a'
/// == AinChar::SmallA`, `AinChar::SmallA == AinChar::CapitalA`, and `AinChar::CapitalA == 'A'`
/// would be true regardless of how we handled equality, but `'a' != 'A'` even though every equality
/// step in the middle is true.
///
/// For Ord, the 'case insensitive' order is to treat all characters as uppercase. This affects the
/// sort order of letters relative to the following symbols, which lie between uppercase and
/// lowercase ascii: `` [\]^_` ``
#[derive(Default, Copy, Clone, Eq)]
pub enum AinChar {
    /// U+0000 (The default variant)
    #[default]
    Null = 0,
    /// U+0001
    StartOfHeading = 1,
    /// U+0002
    StartOfText = 2,
    /// U+0003
    EndOfText = 3,
    /// U+0004
    EndOfTransmission = 4,
    /// U+0005
    Enquiry = 5,
    /// U+0006
    Acknowledge = 6,
    /// U+0007
    Bell = 7,
    /// U+0008
    Backspace = 8,
    /// U+0009
    CharacterTabulation = 9,
    /// U+000A
    LineFeed = 10,
    /// U+000B
    LineTabulation = 11,
    /// U+000C
    FormFeed = 12,
    /// U+000D
    CarriageReturn = 13,
    /// U+000E
    ShiftOut = 14,
    /// U+000F
    ShiftIn = 15,
    /// U+0010
    DataLinkEscape = 16,
    /// U+0011
    DeviceControlOne = 17,
    /// U+0012
    DeviceControlTwo = 18,
    /// U+0013
    DeviceControlThree = 19,
    /// U+0014
    DeviceControlFour = 20,
    /// U+0015
    NegativeAcknowledge = 21,
    /// U+0016
    SynchronousIdle = 22,
    /// U+0017
    EndOfTransmissionBlock = 23,
    /// U+0018
    Cancel = 24,
    /// U+0019
    EndOfMedium = 25,
    /// U+001A
    Substitute = 26,
    /// U+001B
    Escape = 27,
    /// U+001C
    InformationSeparatorFour = 28,
    /// U+001D
    InformationSeparatorThree = 29,
    /// U+001E
    InformationSeparatorTwo = 30,
    /// U+001F
    InformationSeparatorOne = 31,
    /// U+0020
    Space = 32,
    /// U+0021
    ExclamationMark = 33,
    /// U+0022
    QuotationMark = 34,
    /// U+0023
    NumberSign = 35,
    /// U+0024
    DollarSign = 36,
    /// U+0025
    PercentSign = 37,
    /// U+0026
    Ampersand = 38,
    /// U+0027
    Apostrophe = 39,
    /// U+0028
    LeftParenthesis = 40,
    /// U+0029
    RightParenthesis = 41,
    /// U+002A
    Asterisk = 42,
    /// U+002B
    PlusSign = 43,
    /// U+002C
    Comma = 44,
    /// U+002D
    HyphenMinus = 45,
    /// U+002E
    FullStop = 46,
    /// U+002F
    Solidus = 47,
    /// U+0030
    Digit0 = 48,
    /// U+0031
    Digit1 = 49,
    /// U+0032
    Digit2 = 50,
    /// U+0033
    Digit3 = 51,
    /// U+0034
    Digit4 = 52,
    /// U+0035
    Digit5 = 53,
    /// U+0036
    Digit6 = 54,
    /// U+0037
    Digit7 = 55,
    /// U+0038
    Digit8 = 56,
    /// U+0039
    Digit9 = 57,
    /// U+003A
    Colon = 58,
    /// U+003B
    Semicolon = 59,
    /// U+003C
    LessThanSign = 60,
    /// U+003D
    EqualsSign = 61,
    /// U+003E
    GreaterThanSign = 62,
    /// U+003F
    QuestionMark = 63,
    /// U+0040
    CommercialAt = 64,
    /// U+0041
    CapitalA = 65,
    /// U+0042
    CapitalB = 66,
    /// U+0043
    CapitalC = 67,
    /// U+0044
    CapitalD = 68,
    /// U+0045
    CapitalE = 69,
    /// U+0046
    CapitalF = 70,
    /// U+0047
    CapitalG = 71,
    /// U+0048
    CapitalH = 72,
    /// U+0049
    CapitalI = 73,
    /// U+004A
    CapitalJ = 74,
    /// U+004B
    CapitalK = 75,
    /// U+004C
    CapitalL = 76,
    /// U+004D
    CapitalM = 77,
    /// U+004E
    CapitalN = 78,
    /// U+004F
    CapitalO = 79,
    /// U+0050
    CapitalP = 80,
    /// U+0051
    CapitalQ = 81,
    /// U+0052
    CapitalR = 82,
    /// U+0053
    CapitalS = 83,
    /// U+0054
    CapitalT = 84,
    /// U+0055
    CapitalU = 85,
    /// U+0056
    CapitalV = 86,
    /// U+0057
    CapitalW = 87,
    /// U+0058
    CapitalX = 88,
    /// U+0059
    CapitalY = 89,
    /// U+005A
    CapitalZ = 90,
    /// U+005B
    LeftSquareBracket = 91,
    /// U+005C
    ReverseSolidus = 92,
    /// U+005D
    RightSquareBracket = 93,
    /// U+005E
    CircumflexAccent = 94,
    /// U+005F
    LowLine = 95,
    /// U+0060
    GraveAccent = 96,
    /// U+0061
    SmallA = 97,
    /// U+0062
    SmallB = 98,
    /// U+0063
    SmallC = 99,
    /// U+0064
    SmallD = 100,
    /// U+0065
    SmallE = 101,
    /// U+0066
    SmallF = 102,
    /// U+0067
    SmallG = 103,
    /// U+0068
    SmallH = 104,
    /// U+0069
    SmallI = 105,
    /// U+006A
    SmallJ = 106,
    /// U+006B
    SmallK = 107,
    /// U+006C
    SmallL = 108,
    /// U+006D
    SmallM = 109,
    /// U+006E
    SmallN = 110,
    /// U+006F
    SmallO = 111,
    /// U+0070
    SmallP = 112,
    /// U+0071
    SmallQ = 113,
    /// U+0072
    SmallR = 114,
    /// U+0073
    SmallS = 115,
    /// U+0074
    SmallT = 116,
    /// U+0075
    SmallU = 117,
    /// U+0076
    SmallV = 118,
    /// U+0077
    SmallW = 119,
    /// U+0078
    SmallX = 120,
    /// U+0079
    SmallY = 121,
    /// U+007A
    SmallZ = 122,
    /// U+007B
    LeftCurlyBracket = 123,
    /// U+007C
    VerticalLine = 124,
    /// U+007D
    RightCurlyBracket = 125,
    /// U+007E
    Tilde = 126,
    /// U+007F
    Delete = 127,
}

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

impl Hash for AinChar {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        // We normalize to uppercase since those are numerically lower and so sort before some
        // punctuation marks that normally end up before lowercase ascii.
        (*self as u8).to_ascii_uppercase().hash(state);
    }

    fn hash_slice<H: Hasher>(data: &[Self], state: &mut H) {
        const BLOCK_SIZE: usize = 16;
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
    /// The character with the lowest ASCII code.
    pub const MIN: Self = Self::Null;

    /// The character with the highest ASCII code.
    pub const MAX: Self = Self::Delete;

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
        // 10the addition can return at most 112 (0x70), which doesn't overflow and is within the
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
        self as u8
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
    pub const fn as_ain_str_mut(&mut self) -> &mut AinStr {
        AinStr::from_mut_slice(core::slice::from_mut(self))
    }

    /// Maps letters a-z to A-Z and returns any other character unchanged.
    ///
    /// Note: because `AinChar` is not case sensitive, the result will always compare equal to the
    /// input.
    ///
    /// # Examples
    /// ```
    /// # use ain::AinChar;
    /// assert_eq!(AinChar::new('u').to_uppercase().to_char(), 'U');
    /// assert_eq!(AinChar::new('U').to_uppercase().to_char(), 'U');
    /// assert_eq!(AinChar::new('2').to_uppercase().to_char(), '2');
    /// assert_eq!(AinChar::new('=').to_uppercase().to_char(), '=');
    /// assert_eq!(AinChar::new('[').to_uppercase().to_char(), '[');
    /// ```
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
    /// # use ain::AinChar;
    /// assert_eq!(AinChar::new('U').to_lowercase().to_char(), 'u');
    /// assert_eq!(AinChar::new('u').to_lowercase().to_char(), 'u');
    /// assert_eq!(AinChar::new('2').to_lowercase().to_char(), '2');
    /// assert_eq!(AinChar::new('^').to_lowercase().to_char(), '^');
    /// assert_eq!(AinChar::new('\x7f').to_lowercase().to_char(), '\x7f');
    /// ```
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
    #[inline]
    pub const fn is_alphabetic(self) -> bool {
        self.to_u8().is_ascii_alphabetic()
    }

    /// Checks if the character is alphabetic and uppercase (A-Z).
    ///
    /// # Examples
    /// ```
    /// # use ain::AinChar;
    /// assert_eq!(AinChar::new('A').is_uppercase(), true);
    /// assert_eq!(AinChar::new('a').is_uppercase(), false);
    /// assert_eq!(AinChar::new('@').is_uppercase(), false);
    /// ```
    #[inline]
    pub const fn is_uppercase(self) -> bool {
        self.to_u8().is_ascii_uppercase()
    }

    /// Checks if the character is alphabetic and lowercase (a-z).
    ///
    /// This method is identical to [`is_lowercase()`](#method.is_lowercase)
    #[inline]
    pub const fn is_lowercase(self) -> bool {
        self.to_u8().is_ascii_lowercase()
    }

    /// Check if the character is a letter or decimal digit.
    #[inline]
    pub const fn is_alphanumeric(self) -> bool {
        self.to_u8().is_ascii_alphanumeric()
    }

    /// Check if the character is a number (0-9)
    ///
    /// # Examples
    /// ```
    /// # use ain::AinChar;
    /// assert_eq!(AinChar::new('0').is_digit(), true);
    /// assert_eq!(AinChar::new('9').is_digit(), true);
    /// assert_eq!(AinChar::new('a').is_digit(), false);
    /// assert_eq!(AinChar::new('A').is_digit(), false);
    /// assert_eq!(AinChar::new('/').is_digit(), false);
    /// ```
    #[inline]
    pub const fn is_digit(self) -> bool {
        self.to_u8().is_ascii_digit()
    }

    /// Checks if the character is a valid octal digit
    ///
    /// # Examples
    /// ```
    /// # use ain::AinChar;
    /// assert_eq!(AinChar::new('5').is_octdigit(), true);
    /// assert_eq!(AinChar::new('a').is_octdigit(), false);
    /// assert_eq!(AinChar::new('F').is_octdigit(), false);
    /// assert_eq!(AinChar::new('8').is_octdigit(), false);
    /// assert_eq!(AinChar::new('G').is_octdigit(), false);
    /// assert_eq!(AinChar::new(' ').is_octdigit(), false);
    /// ```
    #[inline]
    pub const fn is_octdigit(self) -> bool {
        self.is_digit() && self.to_u8() < b'8'
    }

    /// Checks if the character is a valid hex digit
    ///
    /// # Examples
    /// ```
    /// # use ain::AinChar;
    /// assert_eq!(AinChar::new('5').is_hexdigit(), true);
    /// assert_eq!(AinChar::new('a').is_hexdigit(), true);
    /// assert_eq!(AinChar::new('F').is_hexdigit(), true);
    /// assert_eq!(AinChar::new('G').is_hexdigit(), false);
    /// assert_eq!(AinChar::new(' ').is_hexdigit(), false);
    /// ```
    #[inline]
    pub const fn is_hexdigit(self) -> bool {
        self.to_u8().is_ascii_hexdigit()
    }

    /// Checks if the character is punctuation
    ///
    /// # Examples
    /// ```
    /// # use ain::AinChar;
    /// assert_eq!(AinChar::new('n').is_punctuation(), false);
    /// assert_eq!(AinChar::new(' ').is_punctuation(), false);
    /// assert_eq!(AinChar::new('_').is_punctuation(), true);
    /// assert_eq!(AinChar::new('~').is_punctuation(), true);
    /// ```
    #[inline]
    pub const fn is_punctuation(self) -> bool {
        self.to_u8().is_ascii_punctuation()
    }

    /// Checks if the character is printable (except space)
    ///
    /// # Examples
    /// ```
    /// # use ain::AinChar;
    /// assert_eq!(AinChar::new('n').is_graphic(), true);
    /// assert_eq!(AinChar::new(' ').is_graphic(), false);
    /// assert_eq!(AinChar::new('\n').is_graphic(), false);
    /// ```
    #[inline]
    pub const fn is_graphic(self) -> bool {
        self.to_u8().is_ascii_graphic()
    }

    /// Check if the character one of ' ', '\t', '\n', '\r',
    /// '\0xb' (vertical tab) or '\0xc' (form feed).
    #[inline]
    pub const fn is_whitespace(self) -> bool {
        self.to_u8().is_ascii_whitespace()
    }

    /// Check if the character is a control character
    ///
    /// # Examples
    /// ```
    /// # use ain::AinChar;
    /// assert_eq!(AinChar::new('\0').is_control(), true);
    /// assert_eq!(AinChar::new('n').is_control(), false);
    /// assert_eq!(AinChar::new(' ').is_control(), false);
    /// assert_eq!(AinChar::new('\n').is_control(), true);
    /// assert_eq!(AinChar::new('\t').is_control(), true);
    /// assert_eq!(AinChar::EOT.is_control(), true);
    /// ```
    #[inline]
    pub const fn is_control(self) -> bool {
        self.to_u8().is_ascii_control()
    }

    /// Provides a const way to perform equality on AinChar values.
    #[inline]
    pub(crate) const fn eq(self, other: Self) -> bool {
        self.to_u8().eq_ignore_ascii_case(&other.to_u8())
    }
}

impl AsRef<AinStr> for AinChar {
    fn as_ref(&self) -> &AinStr {
        self.as_ain_str()
    }
}

impl AsMut<AinStr> for AinChar {
    fn as_mut(&mut self) -> &mut AinStr {
        self.as_ain_str_mut()
    }
}

impl AsRef<str> for AinChar {
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Input was not a valid ASCII character")
    }
}

impl core::error::Error for TryIntoAinCharError {}

impl TryFrom<u8> for AinChar {
    type Error = TryIntoAinCharError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::from_u8(value).ok_or(TryIntoAinCharError { _private: () })
    }
}

impl TryFrom<char> for AinChar {
    type Error = TryIntoAinCharError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        Self::from_char(value).ok_or(TryIntoAinCharError { _private: () })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet, HashSet};

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
