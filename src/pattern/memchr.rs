use crate::{AinChar, AinStr};

/// Helper for running memchr on possibly-case-insensitve data.
pub(super) trait MemchrMode {
    /// Runs memchr but differs depending on the case mode.
    fn memchr(ch: AinChar, haystack: &AinStr) -> Option<usize>;

    /// Runs memrchr but differes depending on the case mode.
    fn memrchr(ch: AinChar, haystack: &AinStr) -> Option<usize>;
}

/// Runs memchr always case-sensitive.
pub(super) struct CasedMemchr;

impl MemchrMode for CasedMemchr {
    #[inline(always)]
    fn memchr(ch: AinChar, haystack: &AinStr) -> Option<usize> {
        memchr::memchr(ch.to_u8(), haystack.as_bytes())
    }

    #[inline(always)]
    fn memrchr(ch: AinChar, haystack: &AinStr) -> Option<usize> {
        memchr::memrchr(ch.to_u8(), haystack.as_bytes())
    }
}

/// Runs memchr case-insensitive, always using two bytes.
pub(super) struct UncasedMemchr;

impl MemchrMode for UncasedMemchr {
    #[inline(always)]
    fn memchr(ch: AinChar, haystack: &AinStr) -> Option<usize> {
        memchr::memchr2(
            ch.to_lowercase().to_u8(),
            ch.to_uppercase().to_u8(),
            haystack.as_bytes(),
        )
    }

    #[inline(always)]
    fn memrchr(ch: AinChar, haystack: &AinStr) -> Option<usize> {
        memchr::memrchr2(
            ch.to_lowercase().to_u8(),
            ch.to_uppercase().to_u8(),
            haystack.as_bytes(),
        )
    }
}

/// Runs memchr case-insensitive, but selecting MemchrMode based whether the input is alphabetic.
pub(super) struct AutoMemchr;

impl MemchrMode for AutoMemchr {
    #[inline(always)]
    fn memchr(ch: AinChar, haystack: &AinStr) -> Option<usize> {
        if ch.is_alphabetic() {
            UncasedMemchr::memchr(ch, haystack)
        } else {
            CasedMemchr::memchr(ch, haystack)
        }
    }

    #[inline(always)]
    fn memrchr(ch: AinChar, haystack: &AinStr) -> Option<usize> {
        if ch.is_alphabetic() {
            UncasedMemchr::memrchr(ch, haystack)
        } else {
            CasedMemchr::memrchr(ch, haystack)
        }
    }
}
