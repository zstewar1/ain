use core::ops::Range;

use crate::{AinChar, AinStr};

use super::{DoubleEndedPattern, Pattern, Sealed, Searcher};

impl<const N: usize> Sealed for &[AinChar; N] {}
impl<const N: usize> DoubleEndedPattern for &[AinChar; N] {}
impl<'a, const N: usize> Pattern for &'a [AinChar; N] {
    type Searcher = CharSetSearcher;

    #[inline]
    fn into_searcher(self) -> Self::Searcher {
        CharSetSearcher::new(self)
    }

    #[inline]
    fn start_match_split_point(&mut self, haystack: &AinStr) -> Option<usize> {
        let mut set = &self[..];
        <&[AinChar]>::start_match_split_point(&mut set, haystack)
    }

    #[inline]
    fn start_matches_split_point(&mut self, haystack: &AinStr) -> usize {
        let mut set = &self[..];
        <&[AinChar]>::start_matches_split_point(&mut set, haystack)
    }

    #[inline]
    fn end_match_split_point(&mut self, haystack: &AinStr) -> Option<usize> {
        let mut set = &self[..];
        <&[AinChar]>::end_match_split_point(&mut set, haystack)
    }

    #[inline]
    fn end_matches_split_point(&mut self, haystack: &AinStr) -> usize {
        let mut set = &self[..];
        <&[AinChar]>::end_matches_split_point(&mut set, haystack)
    }
}

impl Sealed for &[AinChar] {}
impl DoubleEndedPattern for &[AinChar] {}
impl Pattern for &[AinChar] {
    type Searcher = CharSetSearcher;

    #[inline]
    fn into_searcher(self) -> Self::Searcher {
        CharSetSearcher::new(self)
    }

    #[inline]
    fn start_match_split_point(&mut self, haystack: &AinStr) -> Option<usize> {
        haystack
            .first()
            .and_then(|c| if self.contains(&c) { Some(1) } else { None })
    }

    #[inline]
    fn start_matches_split_point(&mut self, haystack: &AinStr) -> usize {
        // Arbitrary heuristic to decide when to build a CharSetSearcher for a one-off split-point
        // search.
        //
        // It takes O(`self.len()`) to build the CharSetSearcher, plus we will search at most
        // O(`haystack.len()`) times, so if we don't build a CharSetSearcher, we will spend worst
        // case O(`self.len() * haystack.len()`) time searching. If we do build a CharSetSearcher,
        // we will spend O(`self.len() + haystack.len()`) searching, however there is an overhead to
        // u128 operations which could be worse depending on the CPU. If the needle is fewer bytes
        // than u128, don't bother with a CharSetSearcher. That choice is arbitrary unless we get
        // some benchmarks saying where the heuristic cut-off point is such that building the u128
        // saves time.
        if self.len() < core::mem::size_of::<u128>() {
            haystack
                .chars()
                .position(|c| !self.contains(&c))
                .unwrap_or(haystack.len())
        } else {
            let searcher = CharSetSearcher::new(self);
            haystack
                .chars()
                .position(|c| !searcher.matches(c))
                .unwrap_or(haystack.len())
        }
    }

    #[inline]
    fn end_match_split_point(&mut self, haystack: &AinStr) -> Option<usize> {
        haystack.last().and_then(|c| {
            if self.contains(&c) {
                Some(haystack.len() - 1)
            } else {
                None
            }
        })
    }

    #[inline]
    fn end_matches_split_point(&mut self, haystack: &AinStr) -> usize {
        if self.len() < core::mem::size_of::<u128>() {
            haystack
                .chars()
                .rposition(|c| !self.contains(&c))
                // The start of the match is the index where the non-matching char was found + 1.
                // If no non-matching char was found, the start of the match is 0 (default).
                .map_or(0, |idx| idx + 1)
        } else {
            let searcher = CharSetSearcher::new(self);
            haystack
                .chars()
                .rposition(|c| !searcher.matches(c))
                .map_or(0, |idx| idx + 1)
        }
    }
}

/// Internal searcher type for AinChar.
#[derive(Debug, Copy, Clone)]
pub struct CharSetSearcher {
    needles: u128,
}

impl CharSetSearcher {
    #[inline]
    fn new(needles: &[AinChar]) -> Self {
        let needles = needles.iter().fold(0u128, |needles, &ch| {
            needles
                | Self::char_to_lookup_mask(ch.to_lowercase())
                | Self::char_to_lookup_mask(ch.to_uppercase())
        });
        Self { needles }
    }

    /// Create a lookup mask that check against the contained needles. This skips case folding,
    /// because the needles bitset contains case-folded characters.
    #[inline]
    fn char_to_lookup_mask(ch: AinChar) -> u128 {
        1u128 << ch.to_u8()
    }

    #[inline]
    fn matches(&self, ch: AinChar) -> bool {
        (self.needles & Self::char_to_lookup_mask(ch)) != 0
    }
}

impl Sealed for CharSetSearcher {}
impl Searcher for CharSetSearcher {
    #[inline]
    fn next_match_from_start(&mut self, haystack: &AinStr) -> Option<Range<usize>> {
        haystack
            .chars()
            .position(|ch| self.matches(ch))
            // idx < len, so idx+1 is at most len.
            .map(|idx| idx..idx + 1)
    }

    #[inline]
    fn next_match_from_end(&mut self, haystack: &AinStr) -> Option<Range<usize>> {
        haystack
            .chars()
            .rposition(|ch| self.matches(ch))
            // idx < len, so idx+1 is at most len.
            .map(|idx| idx..idx + 1)
    }
}
