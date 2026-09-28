use core::ops::Range;

use crate::AinStr;

use super::{DoubleEndedPattern, Pattern, Sealed, SearchHolder, Searcher};

/// A pattern that matches either `"\n"` or `"\r\n"`.
///
/// Note that [`s.split(LineEnd)`][AinStr::split] is slightly different from
/// [`s.lines()`][AinStr::lines], because `split` returns an empty string after a trailing new line
/// at the end of the string, while `lines` does not include a blank line after a trailing newline
/// at the end of the string.
#[derive(Debug, Clone, Copy)]
pub struct LineEnd;

impl LineEnd {
    const SEARCHER: LineEndSearcher = LineEndSearcher { _private: () };

    /// Create a new [SearchHolder] in the already-done state.
    #[inline]
    pub(crate) fn create_done_searcher<H: Default>() -> SearchHolder<LineEndSearcher, H> {
        SearchHolder::new_done(Self)
    }

    /// Create a new [SearchHolder] for the given haystack.
    #[inline]
    pub(crate) fn search<H>(haystack: H) -> SearchHolder<LineEndSearcher, H> {
        SearchHolder::new(Self, haystack)
    }
}

impl Sealed for LineEnd {}
impl DoubleEndedPattern for LineEnd {}
impl Pattern for LineEnd {
    type Searcher = LineEndSearcher;

    #[inline]
    fn into_searcher(self) -> Self::Searcher {
        Self::SEARCHER
    }

    #[inline]
    fn start_match_split_point(&mut self, haystack: &AinStr) -> Option<usize> {
        match haystack.as_bytes() {
            [b'\n', ..] => Some(1),
            [b'\r', b'\n', ..] => Some(2),
            _ => None,
        }
    }

    #[inline]
    fn start_matches_split_point(&mut self, haystack: &AinStr) -> usize {
        let haystack = haystack.as_bytes();
        let mut rest = haystack;
        loop {
            rest = match rest {
                [b'\r', b'\n', rest @ ..] | [b'\n', rest @ ..] => rest,
                _ => break,
            };
        }
        haystack.len() - rest.len()
    }

    #[inline]
    fn end_match_split_point(&mut self, haystack: &AinStr) -> Option<usize> {
        match haystack.as_bytes() {
            // Match for \r\n must come first so that we do a greedy match, since [.., \n] would
            // also match \r\n.
            [.., b'\r', b'\n'] => Some(haystack.len() - 2),
            [.., b'\n'] => Some(haystack.len() - 1),
            _ => None,
        }
    }

    #[inline]
    fn end_matches_split_point(&mut self, haystack: &AinStr) -> usize {
        let haystack = haystack.as_bytes();
        let mut rest = haystack;
        loop {
            rest = match rest {
                // Order is important here, since [.., \n] would also match [.., \r\n].
                [rest @ .., b'\r', b'\n'] | [rest @ .., b'\n'] => rest,
                _ => break,
            };
        }
        rest.len()
    }
}

/// Searcher for [`LineEnd`].
#[derive(Debug, Clone, Copy)]
pub struct LineEndSearcher {
    _private: (),
}

impl Sealed for LineEndSearcher {}
// Forward and reverse matches cannot overlap.
impl Searcher for LineEndSearcher {
    #[inline]
    fn next_match_from_start(&mut self, haystack: &AinStr) -> Option<Range<usize>> {
        match memchr::memchr(b'\n', haystack.as_bytes()) {
            None => None,
            Some(idx) if idx > 0 && haystack[idx - 1].to_u8() == b'\r' => Some(idx - 1..idx + 1),
            Some(idx) => Some(idx..idx + 1),
        }
    }

    #[inline]
    fn next_match_from_end(&mut self, haystack: &AinStr) -> Option<Range<usize>> {
        match memchr::memrchr(b'\n', haystack.as_bytes()) {
            None => None,
            Some(idx) if idx > 0 && haystack[idx - 1].to_u8() == b'\r' => Some(idx - 1..idx + 1),
            Some(idx) => Some(idx..idx + 1),
        }
    }
}
