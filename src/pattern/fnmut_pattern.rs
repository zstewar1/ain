use core::ops::Range;

use crate::{AinChar, AinStr};

use super::{DoubleEndedPattern, Pattern, Sealed, Searcher};

impl<F> Sealed for F where F: FnMut(AinChar) -> bool {}
impl<F> DoubleEndedPattern for F where F: FnMut(AinChar) -> bool {}
impl<F> Pattern for F
where
    F: FnMut(AinChar) -> bool,
{
    type Searcher = FnMutSearcher<F>;

    #[inline]
    fn into_searcher(self) -> Self::Searcher {
        FnMutSearcher { predicate: self }
    }

    #[inline]
    fn start_match_split_point(&mut self, haystack: &AinStr) -> Option<usize> {
        haystack
            .first()
            .and_then(|c| if self(c) { Some(1) } else { None })
    }

    #[inline]
    fn start_matches_split_point(&mut self, haystack: &AinStr) -> usize {
        haystack
            .chars()
            .position(|c| !self(c))
            .unwrap_or(haystack.len())
    }

    #[inline]
    fn end_match_split_point(&mut self, haystack: &AinStr) -> Option<usize> {
        haystack.last().and_then(|c| {
            if self(c) {
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
            .rposition(|c| !self(c))
            // The start of the match is the index where the non-matching char was found + 1.
            // If no non-matching char was found, the start of the match is 0 (default).
            .map_or(0, |idx| idx + 1)
    }
}

/// Internal searcher type for AinChar.
#[derive(Debug, Copy, Clone)]
pub struct FnMutSearcher<F> {
    predicate: F,
}

impl<F> Sealed for FnMutSearcher<F> where F: FnMut(AinChar) -> bool {}
impl<F> Searcher for FnMutSearcher<F>
where
    F: FnMut(AinChar) -> bool,
{
    #[inline]
    fn next_match_from_start(&mut self, haystack: &AinStr) -> Option<Range<usize>> {
        haystack
            .chars()
            .position(&mut self.predicate)
            // idx < len, so idx+1 is at most len.
            .map(|idx| idx..idx + 1)
    }

    #[inline]
    fn next_match_from_end(&mut self, haystack: &AinStr) -> Option<Range<usize>> {
        haystack
            .chars()
            .rposition(&mut self.predicate)
            // idx < len, so idx+1 is at most len.
            .map(|idx| idx..idx + 1)
    }
}
