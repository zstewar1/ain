use core::cmp::Ordering;
use core::hash::{Hash, Hasher};
use core::{fmt, mem};

pub use ascii::ToAsciiCharError as ToAinCharError;
use ascii::{AsciiChar, ToAsciiChar};

/// Wrapper which makes an [AsciiChar] have case-insensitive comparisons.
///
/// This type is `repr(transparent)` with [AsciiChar].
///
/// Since the range of allowed values is exactly the same as `AsciiChar` it is always safe to
/// convert between them bidirectionally.
///
/// Unlike `AsciiChar`, we don't allow comparisons between `AinChar` and `char` or `u8` because it
/// is unclear whether comparisons would be case insensitive or not. Either option would break
/// transitive equality.
#[derive(Default, Copy, Clone, Eq)]
#[repr(transparent)]
pub struct AinChar(AsciiChar);

impl fmt::Debug for AinChar {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <AsciiChar as fmt::Debug>::fmt(&self.0, f)
    }
}

impl fmt::Display for AinChar {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <AsciiChar as fmt::Display>::fmt(&self.0, f)
    }
}

impl PartialEq for AinChar {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }
}

impl Hash for AinChar {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // We normalize to uppercase since those are numerically lower and so sort before some
        // punctuation marks that normally end up before lowercase ascii.
        self.0.to_ascii_uppercase().hash(state);
    }
}

impl PartialOrd for AinChar {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AinChar {
    fn cmp(&self, other: &Self) -> Ordering {
        // We normalize to uppercase since those are numerically lower and so sort before some
        // punctuation marks that normally end up before lowercase ascii.
        self.0
            .to_ascii_uppercase()
            .cmp(&other.0.to_ascii_uppercase())
    }
}

impl From<AsciiChar> for AinChar {
    #[inline]
    fn from(value: AsciiChar) -> Self {
        Self(value)
    }
}

impl From<AinChar> for AsciiChar {
    #[inline]
    fn from(value: AinChar) -> Self {
        value.0
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

/// Defines constants for every character.
#[allow(non_upper_case_globals)]
impl AinChar {
    /// `'\0'`
    pub const Null: AinChar = AinChar(AsciiChar::Null);
    /// [Start Of Heading](http://en.wikipedia.org/wiki/Start_of_Heading)
    pub const SOH: AinChar = AinChar(AsciiChar::SOH);
    /// [Start Of teXt](http://en.wikipedia.org/wiki/Start_of_Text)
    pub const SOX: AinChar = AinChar(AsciiChar::SOX);
    /// [End of TeXt](http://en.wikipedia.org/wiki/End-of-Text_character)
    pub const ETX: AinChar = AinChar(AsciiChar::ETX);
    /// [End Of Transmission](http://en.wikipedia.org/wiki/End-of-Transmission_character)
    pub const EOT: AinChar = AinChar(AsciiChar::EOT);
    /// [Enquiry](http://en.wikipedia.org/wiki/Enquiry_character)
    pub const ENQ: AinChar = AinChar(AsciiChar::ENQ);
    /// [Acknowledgement](http://en.wikipedia.org/wiki/Acknowledge_character)
    pub const ACK: AinChar = AinChar(AsciiChar::ACK);
    /// [bell / alarm / audible](http://en.wikipedia.org/wiki/Bell_character)
    ///
    /// `'\a'` is not recognized by Rust.
    pub const Bell: AinChar = AinChar(AsciiChar::Bell);
    /// [Backspace](http://en.wikipedia.org/wiki/Backspace)
    ///
    /// `'\b'` is not recognized by Rust.
    pub const BackSpace: AinChar = AinChar(AsciiChar::BackSpace);
    /// `'\t'`
    pub const Tab: AinChar = AinChar(AsciiChar::Tab);
    /// `'\n'`
    pub const LineFeed: AinChar = AinChar(AsciiChar::LineFeed);
    /// [Vertical tab](http://en.wikipedia.org/wiki/Vertical_Tab)
    ///
    /// `'\v'` is not recognized by Rust.
    pub const VT: AinChar = AinChar(AsciiChar::VT);
    /// [Form Feed](http://en.wikipedia.org/wiki/Form_Feed)
    ///
    /// `'\f'` is not recognized by Rust.
    pub const FF: AinChar = AinChar(AsciiChar::FF);
    /// `'\r'`
    pub const CarriageReturn: AinChar = AinChar(AsciiChar::CarriageReturn);
    /// [Shift In](http://en.wikipedia.org/wiki/Shift_Out_and_Shift_In_characters)
    pub const SI: AinChar = AinChar(AsciiChar::SI);
    /// [Shift Out](http://en.wikipedia.org/wiki/Shift_Out_and_Shift_In_characters)
    pub const SO: AinChar = AinChar(AsciiChar::SO);
    /// [Data Link Escape](http://en.wikipedia.org/wiki/Data_Link_Escape)
    pub const DLE: AinChar = AinChar(AsciiChar::DLE);
    /// [Device control 1, often XON](http://en.wikipedia.org/wiki/Device_Control_1)
    pub const DC1: AinChar = AinChar(AsciiChar::DC1);
    /// Device control 2
    pub const DC2: AinChar = AinChar(AsciiChar::DC2);
    /// Device control 3, Often XOFF
    pub const DC3: AinChar = AinChar(AsciiChar::DC3);
    /// Device control 4
    pub const DC4: AinChar = AinChar(AsciiChar::DC4);
    /// [Negative AcKnowledgement](http://en.wikipedia.org/wiki/Negative-acknowledge_character)
    pub const NAK: AinChar = AinChar(AsciiChar::NAK);
    /// [Synchronous idle](http://en.wikipedia.org/wiki/Synchronous_Idle)
    pub const SYN: AinChar = AinChar(AsciiChar::SYN);
    /// [End of Transmission Block](http://en.wikipedia.org/wiki/End-of-Transmission-Block_character)
    pub const ETB: AinChar = AinChar(AsciiChar::ETB);
    /// [Cancel](http://en.wikipedia.org/wiki/Cancel_character)
    pub const CAN: AinChar = AinChar(AsciiChar::CAN);
    /// [End of Medium](http://en.wikipedia.org/wiki/End_of_Medium)
    pub const EM: AinChar = AinChar(AsciiChar::EM);
    /// [Substitute](http://en.wikipedia.org/wiki/Substitute_character)
    pub const SUB: AinChar = AinChar(AsciiChar::SUB);
    /// [Escape](http://en.wikipedia.org/wiki/Escape_character)
    ///
    /// `'\e'` is not recognized by Rust.
    pub const ESC: AinChar = AinChar(AsciiChar::ESC);
    /// [File Separator](http://en.wikipedia.org/wiki/File_separator)
    pub const FS: AinChar = AinChar(AsciiChar::FS);
    /// [Group Separator](http://en.wikipedia.org/wiki/Group_separator)
    pub const GS: AinChar = AinChar(AsciiChar::GS);
    /// [Record Separator](http://en.wikipedia.org/wiki/Record_separator)
    pub const RS: AinChar = AinChar(AsciiChar::RS);
    /// [Unit Separator](http://en.wikipedia.org/wiki/Unit_separator)
    pub const US: AinChar = AinChar(AsciiChar::US);
    /// `' '`
    pub const Space: AinChar = AinChar(AsciiChar::Space);
    /// `'!'`
    pub const Exclamation: AinChar = AinChar(AsciiChar::Exclamation);
    /// `'"'`
    pub const Quotation: AinChar = AinChar(AsciiChar::Quotation);
    /// `'#'`
    pub const Hash: AinChar = AinChar(AsciiChar::Hash);
    /// `'$'`
    pub const Dollar: AinChar = AinChar(AsciiChar::Dollar);
    /// `'%'`
    pub const Percent: AinChar = AinChar(AsciiChar::Percent);
    /// `'&'`
    pub const Ampersand: AinChar = AinChar(AsciiChar::Ampersand);
    /// `'\''`
    pub const Apostrophe: AinChar = AinChar(AsciiChar::Apostrophe);
    /// `'('`
    pub const ParenOpen: AinChar = AinChar(AsciiChar::ParenOpen);
    /// `')'`
    pub const ParenClose: AinChar = AinChar(AsciiChar::ParenClose);
    /// `'*'`
    pub const Asterisk: AinChar = AinChar(AsciiChar::Asterisk);
    /// `'+'`
    pub const Plus: AinChar = AinChar(AsciiChar::Plus);
    /// `','`
    pub const Comma: AinChar = AinChar(AsciiChar::Comma);
    /// `'-'`
    pub const Minus: AinChar = AinChar(AsciiChar::Minus);
    /// `'.'`
    pub const Dot: AinChar = AinChar(AsciiChar::Dot);
    /// `'/'`
    pub const Slash: AinChar = AinChar(AsciiChar::Slash);
    /// `'0'`
    pub const D0: AinChar = AinChar(AsciiChar::_0);
    /// `'1'`
    pub const D1: AinChar = AinChar(AsciiChar::_1);
    /// `'2'`
    pub const D2: AinChar = AinChar(AsciiChar::_2);
    /// `'3'`
    pub const D3: AinChar = AinChar(AsciiChar::_3);
    /// `'4'`
    pub const D4: AinChar = AinChar(AsciiChar::_4);
    /// `'5'`
    pub const D5: AinChar = AinChar(AsciiChar::_5);
    /// `'6'`
    pub const D6: AinChar = AinChar(AsciiChar::_6);
    /// `'7'`
    pub const D7: AinChar = AinChar(AsciiChar::_7);
    /// `'8'`
    pub const D8: AinChar = AinChar(AsciiChar::_8);
    /// `'9'`
    pub const D9: AinChar = AinChar(AsciiChar::_9);
    /// `':'`
    pub const Colon: AinChar = AinChar(AsciiChar::Colon);
    /// `';'`
    pub const Semicolon: AinChar = AinChar(AsciiChar::Semicolon);
    /// `'<'`
    pub const LessThan: AinChar = AinChar(AsciiChar::LessThan);
    /// `'='`
    pub const Equal: AinChar = AinChar(AsciiChar::Equal);
    /// `'>'`
    pub const GreaterThan: AinChar = AinChar(AsciiChar::GreaterThan);
    /// `'?'`
    pub const Question: AinChar = AinChar(AsciiChar::Question);
    /// `'@'`
    pub const At: AinChar = AinChar(AsciiChar::At);
    /// `'A'`
    pub const A: AinChar = AinChar(AsciiChar::A);
    /// `'B'`
    pub const B: AinChar = AinChar(AsciiChar::B);
    /// `'C'`
    pub const C: AinChar = AinChar(AsciiChar::C);
    /// `'D'`
    pub const D: AinChar = AinChar(AsciiChar::D);
    /// `'E'`
    pub const E: AinChar = AinChar(AsciiChar::E);
    /// `'F'`
    pub const F: AinChar = AinChar(AsciiChar::F);
    /// `'G'`
    pub const G: AinChar = AinChar(AsciiChar::G);
    /// `'H'`
    pub const H: AinChar = AinChar(AsciiChar::H);
    /// `'I'`
    pub const I: AinChar = AinChar(AsciiChar::I);
    /// `'J'`
    pub const J: AinChar = AinChar(AsciiChar::J);
    /// `'K'`
    pub const K: AinChar = AinChar(AsciiChar::K);
    /// `'L'`
    pub const L: AinChar = AinChar(AsciiChar::L);
    /// `'M'`
    pub const M: AinChar = AinChar(AsciiChar::M);
    /// `'N'`
    pub const N: AinChar = AinChar(AsciiChar::N);
    /// `'O'`
    pub const O: AinChar = AinChar(AsciiChar::O);
    /// `'P'`
    pub const P: AinChar = AinChar(AsciiChar::P);
    /// `'Q'`
    pub const Q: AinChar = AinChar(AsciiChar::Q);
    /// `'R'`
    pub const R: AinChar = AinChar(AsciiChar::R);
    /// `'S'`
    pub const S: AinChar = AinChar(AsciiChar::S);
    /// `'T'`
    pub const T: AinChar = AinChar(AsciiChar::T);
    /// `'U'`
    pub const U: AinChar = AinChar(AsciiChar::U);
    /// `'V'`
    pub const V: AinChar = AinChar(AsciiChar::V);
    /// `'W'`
    pub const W: AinChar = AinChar(AsciiChar::W);
    /// `'X'`
    pub const X: AinChar = AinChar(AsciiChar::X);
    /// `'Y'`
    pub const Y: AinChar = AinChar(AsciiChar::Y);
    /// `'Z'`
    pub const Z: AinChar = AinChar(AsciiChar::Z);
    /// `'['`
    pub const BracketOpen: AinChar = AinChar(AsciiChar::BracketOpen);
    /// `'\'`
    pub const BackSlash: AinChar = AinChar(AsciiChar::BackSlash);
    /// `']'`
    pub const BracketClose: AinChar = AinChar(AsciiChar::BracketClose);
    /// `'^'`
    pub const Caret: AinChar = AinChar(AsciiChar::Caret);
    /// `'_'`
    pub const UnderScore: AinChar = AinChar(AsciiChar::UnderScore);
    /// `'`'`
    pub const Grave: AinChar = AinChar(AsciiChar::Grave);
    /// `'a'`
    pub const a: AinChar = AinChar(AsciiChar::a);
    /// `'b'`
    pub const b: AinChar = AinChar(AsciiChar::b);
    /// `'c'`
    pub const c: AinChar = AinChar(AsciiChar::c);
    /// `'d'`
    pub const d: AinChar = AinChar(AsciiChar::d);
    /// `'e'`
    pub const e: AinChar = AinChar(AsciiChar::e);
    /// `'f'`
    pub const f: AinChar = AinChar(AsciiChar::f);
    /// `'g'`
    pub const g: AinChar = AinChar(AsciiChar::g);
    /// `'h'`
    pub const h: AinChar = AinChar(AsciiChar::h);
    /// `'i'`
    pub const i: AinChar = AinChar(AsciiChar::i);
    /// `'j'`
    pub const j: AinChar = AinChar(AsciiChar::j);
    /// `'k'`
    pub const k: AinChar = AinChar(AsciiChar::k);
    /// `'l'`
    pub const l: AinChar = AinChar(AsciiChar::l);
    /// `'m'`
    pub const m: AinChar = AinChar(AsciiChar::m);
    /// `'n'`
    pub const n: AinChar = AinChar(AsciiChar::n);
    /// `'o'`
    pub const o: AinChar = AinChar(AsciiChar::o);
    /// `'p'`
    pub const p: AinChar = AinChar(AsciiChar::p);
    /// `'q'`
    pub const q: AinChar = AinChar(AsciiChar::q);
    /// `'r'`
    pub const r: AinChar = AinChar(AsciiChar::r);
    /// `'s'`
    pub const s: AinChar = AinChar(AsciiChar::s);
    /// `'t'`
    pub const t: AinChar = AinChar(AsciiChar::t);
    /// `'u'`
    pub const u: AinChar = AinChar(AsciiChar::u);
    /// `'v'`
    pub const v: AinChar = AinChar(AsciiChar::v);
    /// `'w'`
    pub const w: AinChar = AinChar(AsciiChar::w);
    /// `'x'`
    pub const x: AinChar = AinChar(AsciiChar::x);
    /// `'y'`
    pub const y: AinChar = AinChar(AsciiChar::y);
    /// `'z'`
    pub const z: AinChar = AinChar(AsciiChar::z);
    /// `'{'`
    pub const CurlyBraceOpen: AinChar = AinChar(AsciiChar::CurlyBraceOpen);
    /// `'|'`
    pub const VerticalBar: AinChar = AinChar(AsciiChar::VerticalBar);
    /// `'}'`
    pub const CurlyBraceClose: AinChar = AinChar(AsciiChar::CurlyBraceClose);
    /// `'~'`
    pub const Tilde: AinChar = AinChar(AsciiChar::Tilde);
    /// [Delete](http://en.wikipedia.org/wiki/Delete_character)
    pub const DEL: AinChar = AinChar(AsciiChar::DEL);
}

impl AinChar {
    /// Constructs an ASCII character from a `u8`, `char` or other character type.
    ///
    /// # Errors
    /// Returns `Err(())` if the character can't be ASCII encoded.
    ///
    /// # Example
    /// ```
    /// # use ain::AinChar;
    /// let a = AinChar::from_ascii('g').unwrap();
    /// assert_eq!(a.to_char(), 'g');
    /// ```
    #[inline]
    pub fn from_ascii<C: ToAinChar>(ch: C) -> Result<Self, ToAinCharError> {
        ch.to_ain_char()
    }

    /// Create an `AinChar` from a `char`, panicking if it's not ASCII.
    ///
    /// This function is intended for creating `AinChar` values from hardcoded known-good character
    /// literals such as `'K'`, `'-'` or `'\0'`, and for use in `const` contexts. Use
    /// [`from_ascii()`][Self::from_ascii] instead when you're not certain the character is ASCII.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ain::AinChar;
    /// assert_eq!(AinChar::new('@'), AinChar::At);
    /// assert_eq!(AinChar::new('C').to_char(), 'C');
    /// ```
    ///
    /// In a constant:
    /// ```
    /// # use ain::AinChar;
    /// const SPLIT_ON: AinChar = AinChar::new(',');
    /// ```
    ///
    /// This will not compile:
    /// ```compile_fail
    /// # use ain::AinChar;
    /// const BAD: AinChar = AinChar::new('Ø');
    /// ```
    ///
    /// # Panics
    ///
    /// This function will panic if passed a non-ASCII character.
    ///
    /// The panic message might not be the most descriptive due to the current limitations of `const
    /// fn`.
    #[inline]
    #[must_use]
    pub const fn new(ch: char) -> AinChar {
        Self(AsciiChar::new(ch))
    }

    /// Constructs an ASCII character from a `u8`, `char` or othe rcharacter type without any
    /// checks.
    ///
    /// # Safety
    ///
    /// This function is very unsafe as it can create invalid enum discriminants, which instantly
    /// creates undefined behavior. (`let _ = AinChar::from_ascii_unchecked(200);` alone is UB).
    ///
    /// The undefined behavior is not just theoretical either: For example, `[0;
    /// 128][AsciiChar::from_ascii_unchecked(255) as u8 as usize] = 0` might not panic, creating a
    /// buffer overflow, and `Some(AinChar::from_ascii_unchecked(128))` might be `None`.
    #[inline]
    #[must_use]
    pub unsafe fn from_ascii_unchecked<C: ToAinChar>(ch: C) -> Self {
        // SAFETY: Caller guarantees `ch` is within bounds of ascii.
        unsafe { ch.to_ain_char_unchecked() }
    }

    /// Const-fn variant of from_ascii_unchecked.
    ///
    /// # Safety
    ///
    /// ch must be < 128.
    pub const unsafe fn from_byte_unchecked(ch: u8) -> Self {
        // SAFETY: caller guarantees self is within the Ascii range.
        unsafe { mem::transmute::<u8, Self>(ch) }
    }

    /// Converts numbers 0-9 into the digit characters '0' - '9'
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
    pub const unsafe fn digit_unchecked(d: u8) -> AinChar {
        // SAFETY: `'0'` through `'9'` are U+00030 through U+0039, so because `d` must be less than
        // 10the addition can return at most 112 (0x70), which doesn't overflow and is within the
        // ASCII range.
        unsafe {
            let ch = b'0'.unchecked_add(d);
            Self::from_byte_unchecked(ch)
        }
    }

    /// Converts an ASCII character into a `u8`.
    #[inline]
    #[must_use]
    pub const fn to_u8(self) -> u8 {
        self.0.as_byte()
    }

    /// Converts an ASCII character into a `char`.
    #[inline]
    #[must_use]
    pub const fn to_char(self) -> char {
        self.0.as_char()
    }

    /// Views this ASCII character as a one-character string.
    pub const fn as_str(&self) -> &str {
        let ptr: *const Self = self;
        // We go through this step to avoid doing a pointer cast on a slice pointer.
        // SAFETY: AinChar is repr(transpaent) to AsciiChar which is repr(u8), so casting to u8 is
        // safe, as long as we don't allow mutation.
        let byte_ptr = ptr.cast::<u8>();
        // SAFETY: the ptr is valid for a read of 1 byte, and it comes from a ref so it must be
        // non-null and properly aligned.
        let slice = unsafe { core::slice::from_raw_parts(byte_ptr, 1) };
        // SAFETY: The one byte slice is valid ascii, so it must be a valid str.
        unsafe { str::from_utf8_unchecked(slice) }
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
    #[must_use]
    pub const fn to_uppercase(self) -> Self {
        Self(self.0.to_ascii_uppercase())
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
    #[must_use]
    pub const fn to_lowercase(self) -> Self {
        Self(self.0.to_ascii_lowercase())
    }

    /// Replaces letters `a` to `z` with `A` to `Z`
    ///
    /// Note: because this type is case-insensitive, this does not change the equals or hash.
    #[inline]
    pub fn make_uppercase(&mut self) {
        self.0.make_ascii_uppercase()
    }

    /// Replaces letters `A` to `Z` with `a` to `z`
    ///
    /// Note: because this type is case-insensitive, this does not change the equals or hash.
    #[inline]
    pub fn make_lowercase(&mut self) {
        self.0.make_ascii_lowercase();
    }

    /// Check if the character is a letter (a-z, A-Z)
    #[inline]
    #[must_use]
    pub const fn is_alphabetic(self) -> bool {
        self.0.is_ascii_alphabetic()
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
    #[must_use]
    pub const fn is_uppercase(self) -> bool {
        self.0.is_ascii_uppercase()
    }

    /// Checks if the character is alphabetic and lowercase (a-z).
    ///
    /// This method is identical to [`is_lowercase()`](#method.is_lowercase)
    #[inline]
    #[must_use]
    pub const fn is_lowercase(self) -> bool {
        self.0.is_ascii_lowercase()
    }

    /// Check if the character is a letter or decimal digit.
    #[inline]
    #[must_use]
    pub const fn is_alphanumeric(self) -> bool {
        self.0.is_ascii_alphanumeric()
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
    #[must_use]
    pub const fn is_digit(self) -> bool {
        self.0.is_ascii_digit()
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
    #[must_use]
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
    #[must_use]
    pub const fn is_hexdigit(self) -> bool {
        self.0.is_ascii_hexdigit()
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
    #[must_use]
    pub const fn is_punctuation(self) -> bool {
        self.0.is_ascii_punctuation()
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
    #[must_use]
    pub const fn is_graphic(self) -> bool {
        self.0.is_ascii_graphic()
    }

    /// Check if the character one of ' ', '\t', '\n', '\r',
    /// '\0xb' (vertical tab) or '\0xc' (form feed).
    #[inline]
    #[must_use]
    pub const fn is_whitespace(self) -> bool {
        self.0.is_ascii_whitespace()
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
    #[must_use]
    pub const fn is_control(self) -> bool {
        self.0.is_ascii_control()
    }
}

/// Convert `char`, `u8` and other character types to `AsciiChar`.
pub trait ToAinChar {
    /// Convert to `AinChar`.
    ///
    /// # Errors
    /// If `self` is outside the valid ascii range, this returns `Err`
    fn to_ain_char(self) -> Result<AinChar, ToAinCharError>;

    /// Convert to `AinChar` without checking that it is an ASCII character.
    ///
    /// # Safety
    /// Calling this function with a value outside of the ascii range, `0x0` to `0x7f` inclusive,
    /// is undefined behavior.
    unsafe fn to_ain_char_unchecked(self) -> AinChar;
}

impl<C: ToAsciiChar> ToAinChar for C {
    #[inline]
    fn to_ain_char(self) -> Result<AinChar, ToAinCharError> {
        self.to_ascii_char().map(AinChar)
    }

    #[inline]
    unsafe fn to_ain_char_unchecked(self) -> AinChar {
        // SAFETY: calling to_ascii_char_unchecked has the same contract as to_ain_char_unchecked.
        AinChar(unsafe { self.to_ascii_char_unchecked() })
    }
}

impl ToAsciiChar for AinChar {
    #[inline]
    fn to_ascii_char(self) -> Result<AsciiChar, ToAinCharError> {
        Ok(self.0)
    }

    #[inline]
    unsafe fn to_ascii_char_unchecked(self) -> AsciiChar {
        self.0
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
            let inch = AinChar::new(ch);
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
            let inch = AinChar::new(ch);
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
