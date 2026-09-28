use crate::{AinChar, AinStr};

macro_rules! borrow_from_as_ref {
    ($from:ty, $to:ty) => {
        borrow_from_as_ref!(const $from, $to);
        borrow_from_as_ref!(mut $from, $to);
    };
    (const $from:ty, $to:ty) => {
        impl Borrow<$to> for $from {
            #[inline]
            fn borrow(&self) -> &$to {
                self.as_ref()
            }
        }
    };
    (mut $from:ty, $to:ty) => {
        impl BorrowMut<$to> for $from {
            #[inline]
            fn borrow_mut(&mut self) -> &mut $to {
                self.as_mut()
            }
        }
    };
}

macro_rules! from_ref_from_as_ref {
    ($from:ty, $to:ty) => {
        from_ref_from_as_ref!(const $from, $to);
        from_ref_from_as_ref!(mut $from, $to);
    };
    (const $from:ty, $to:ty) => {
        impl<'a> From<&'a $from> for &'a $to {
            #[inline]
            fn from(value: &$from) -> &$to {
                value.as_ref()
            }
        }
    };
    (mut $from:ty, $to:ty) => {
        impl<'a> From<&'a mut $from> for &'a mut $to {
            #[inline]
            fn from(value: &mut $from) -> &mut $to {
                value.as_mut()
            }
        }
    };
}

/// Creates an AinStr or AinChar constant from a `&str`, `char`, `u8`, or `&[u8]` literal.
///
/// # Examples:
///
/// ```
/// # use ain::{AinStr, AinChar, ain};
/// const STR: &AinStr = ain!("Hello World");
/// const CHAR: AinChar = ain!('C');
/// const BYTES: &AinStr = ain!(&[60, 64, 72, 47]);
/// const BYTE: AinChar = ain!(76);
/// const EMPTY_BYTES: &AinStr = ain!(&[]);
/// const BYTE_STR: &AinStr = ain!(b"Something");
/// const BYTE_CHAR: AinChar = ain!(b'S');
/// const ARRAY: &AinStr = ain!(&[0; 1024]);
/// const AIN_ARRAY: &AinStr = ain!(&['a'; 1024]);
/// const AIN_SLICE: &AinStr = ain!(&['a', 'b']);
/// ```
#[macro_export]
macro_rules! ain {
    ($s:literal) => {
        const { $crate::Literal($s).value() }
    };
    (&[$($c:literal),* $(,)?]) => {
        const { $crate::Literal::<&'static [$crate::AinChar]>(&[$($crate::ain!($c),)*]).value() }
    };
    (&[$c:literal; $len:expr]) => {
        const { $crate::Literal::<&'static [$crate::AinChar]>(&[$crate::ain!($c); $len]).value() }
    };
}

/// Declares an [`&mut AinStr`][AinStr] stored in a stack-local variable. Equivalent to declaring a
/// stack-local array variable and then copying the given literal into it.
///
/// Intended primarily for testing. Normally, if you need a mutable string, you'd use [AinString],
/// but since our doc tests need to pass even in no_std and without alloc, we write our doc tests
/// using this macro to quickly declare a variable holding `&mut AinStr` without the usual
/// difficulty of stack allocation.
///
/// These are equivalent:
///
/// ```
/// # use ain::{ain, ain_mut, AinStr};
/// // This:
/// ain_mut!(some_var, "some value");
///
/// // Expands to this:
/// let mut some_var = [ain!('\0'); ain!("some value").len()];
/// some_var.copy_from_slice(ain!("some value").as_slice());
/// let some_var = AinStr::from_slice_mut(&mut some_var);
/// ```
///
/// # Examples:
///
/// ```
/// # use ain::ain_mut;
/// ain_mut!(str, "Hello World");
/// ain_mut!(bytes, &[60, 64, 72, 47]);
/// ain_mut!(empty_bytes, &[]);
/// ain_mut!(byte_string, b"Something");
///
/// // Can be used to create a mutable buffer for storing an AinStr.
/// ain_mut!(buffer, &[0; 1024]);
/// ```
#[macro_export]
macro_rules! ain_mut {
    ($id:ident, $($v:tt)*) => {
        let mut $id = [$crate::ain!('\0'); $crate::ain!($($v)*).len()];
        $id.copy_from_slice($crate::ain!($($v)*).as_slice());
        let $id = $crate::AinStr::from_slice_mut(&mut $id);
    };
}

/// Helper for the ain macro to select between AinChar and AinStr when converting literals.
#[doc(hidden)]
pub struct Literal<T>(pub T);

impl Literal<&'static str> {
    pub const fn value(self) -> &'static AinStr {
        match AinStr::from_str(self.0) {
            Ok(s) => s,
            Err(_) => panic!("AinStr literal contained invalid ASCII"),
        }
    }
}

impl Literal<char> {
    pub const fn value(self) -> AinChar {
        match AinChar::from_char(self.0) {
            Some(c) => c,
            None => panic!("AinChar literal was not valid ASCII"),
        }
    }
}

impl<const N: usize> Literal<&'static [u8; N]> {
    pub const fn value(self) -> &'static AinStr {
        Literal::<&'static [u8]>(self.0).value()
    }
}

impl Literal<&'static [AinChar]> {
    pub const fn value(self) -> &'static AinStr {
        AinStr::from_slice(self.0)
    }
}

impl Literal<&'static [u8]> {
    pub const fn value(self) -> &'static AinStr {
        match AinStr::from_ascii(self.0) {
            Ok(s) => s,
            Err(_) => panic!("AinStr literal contained invalid ASCII"),
        }
    }
}

impl Literal<u8> {
    pub const fn value(self) -> AinChar {
        match AinChar::from_u8(self.0) {
            Some(c) => c,
            None => panic!("AinChar literal was not valid ASCII"),
        }
    }
}
