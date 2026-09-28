use core::ops::Range;

use crate::{AinChar, AinStr};

use super::memchr::{AutoMemchr, MemchrMode};
use super::{DoubleEndedPattern, Pattern, Sealed, Searcher};

impl Sealed for AinChar {}
impl DoubleEndedPattern for AinChar {}
impl Pattern for AinChar {
    type Searcher = CharSearcher;

    #[inline]
    fn into_searcher(self) -> Self::Searcher {
        CharSearcher { needle: self }
    }

    #[inline]
    fn start_match_split_point(&mut self, haystack: &AinStr) -> Option<usize> {
        haystack
            .first()
            .and_then(|c| if c == *self { Some(1) } else { None })
    }

    #[inline]
    fn start_matches_split_point(&mut self, haystack: &AinStr) -> usize {
        haystack
            .chars()
            .position(|c| c != *self)
            .unwrap_or(haystack.len())
    }

    #[inline]
    fn end_match_split_point(&mut self, haystack: &AinStr) -> Option<usize> {
        haystack.last().and_then(|c| {
            if c == *self {
                Some(haystack.len() - 1)
            } else {
                None
            }
        })
    }

    #[inline]
    fn end_matches_split_point(&mut self, haystack: &AinStr) -> usize {
        haystack
            .chars()
            .rposition(|c| c != *self)
            // The start of the match is the index where the non-matching char was found + 1.
            // If no non-matching char was found, the start of the match is 0 (default).
            .map_or(0, |idx| idx + 1)
    }
}

/// Internal searcher type for AinChar.
#[derive(Debug, Copy, Clone)]
pub struct CharSearcher {
    needle: AinChar,
}

impl Sealed for CharSearcher {}
impl Searcher for CharSearcher {
    #[inline]
    fn next_match_from_start(&mut self, haystack: &AinStr) -> Option<Range<usize>> {
        AutoMemchr::memchr(self.needle, haystack)
            // idx < len, so idx+1 is at most len.
            .map(|idx| idx..idx + 1)
    }

    #[inline]
    fn next_match_from_end(&mut self, haystack: &AinStr) -> Option<Range<usize>> {
        AutoMemchr::memrchr(self.needle, haystack)
            // idx < len, so idx+1 is at most len.
            .map(|idx| idx..idx + 1)
    }
}
