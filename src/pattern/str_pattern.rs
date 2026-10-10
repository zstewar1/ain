use core::ops::Range;

use crate::{AinChar, AinStr};

use super::byte_rank::rank;
use super::memchr::{CasedMemchr, MemchrMode, UncasedMemchr};
use super::{Pattern, Sealed, Searcher};

impl Sealed for &&AinStr {}
impl<'a> Pattern for &&'a AinStr {
    type Searcher = StrSearcher<'a>;

    fn into_searcher(self) -> Self::Searcher {
        <&AinStr as Pattern>::into_searcher(*self)
    }

    fn start_match_split_point(&mut self, haystack: &AinStr) -> Option<usize> {
        let mut s = **self;
        <&AinStr as Pattern>::start_match_split_point(&mut s, haystack)
    }

    fn start_matches_split_point(&mut self, haystack: &AinStr) -> usize {
        let mut s = **self;
        <&AinStr as Pattern>::start_matches_split_point(&mut s, haystack)
    }

    fn end_match_split_point(&mut self, haystack: &AinStr) -> Option<usize> {
        let mut s = **self;
        <&AinStr as Pattern>::end_match_split_point(&mut s, haystack)
    }

    fn end_matches_split_point(&mut self, haystack: &AinStr) -> usize {
        let mut s = **self;
        <&AinStr as Pattern>::end_matches_split_point(&mut s, haystack)
    }
}

#[cfg(feature = "alloc")]
impl Sealed for &crate::AinString {}
#[cfg(feature = "alloc")]
impl<'a> Pattern for &'a crate::AinString {
    type Searcher = StrSearcher<'a>;

    fn into_searcher(self) -> Self::Searcher {
        <&AinStr as Pattern>::into_searcher(self)
    }

    fn start_match_split_point(&mut self, haystack: &AinStr) -> Option<usize> {
        let mut s = &***self;
        <&AinStr as Pattern>::start_match_split_point(&mut s, haystack)
    }

    fn start_matches_split_point(&mut self, haystack: &AinStr) -> usize {
        let mut s = &***self;
        <&AinStr as Pattern>::start_matches_split_point(&mut s, haystack)
    }

    fn end_match_split_point(&mut self, haystack: &AinStr) -> Option<usize> {
        let mut s = &***self;
        <&AinStr as Pattern>::end_match_split_point(&mut s, haystack)
    }

    fn end_matches_split_point(&mut self, haystack: &AinStr) -> usize {
        let mut s = &***self;
        <&AinStr as Pattern>::end_matches_split_point(&mut s, haystack)
    }
}

impl Sealed for &AinStr {}
impl<'a> Pattern for &'a AinStr {
    type Searcher = StrSearcher<'a>;

    fn into_searcher(self) -> Self::Searcher {
        StrSearcher::new(self)
    }

    fn start_match_split_point(&mut self, haystack: &AinStr) -> Option<usize> {
        if haystack.get(..self.len()) == Some(*self) {
            Some(self.len())
        } else {
            None
        }
    }

    fn start_matches_split_point(&mut self, haystack: &AinStr) -> usize {
        if self.is_empty() {
            return 0;
        }
        let mut split_point = 0;
        while let Some(next_split_point) = self.start_match_split_point(&haystack[split_point..]) {
            split_point += next_split_point;
        }
        split_point
    }

    fn end_match_split_point(&mut self, haystack: &AinStr) -> Option<usize> {
        if haystack.len() < self.len() {
            return None;
        }
        let check_start = haystack.len() - self.len();
        if haystack[check_start..] == *self {
            Some(check_start)
        } else {
            None
        }
    }

    fn end_matches_split_point(&mut self, haystack: &AinStr) -> usize {
        if self.is_empty() {
            return haystack.len();
        }
        let mut split_point = haystack.len();
        while let Some(next_split_point) = self.end_match_split_point(&haystack[..split_point]) {
            split_point = next_split_point;
        }
        split_point
    }
}

/// Searches for a particular substring.
#[derive(Debug, Clone)]
pub struct StrSearcher<'a> {
    needle: &'a AinStr,
    inner: StrSearcherImpl,
}

impl<'a> StrSearcher<'a> {
    /// Create a new StrSearcher for the given needle.
    #[inline]
    fn new(needle: &'a AinStr) -> Self {
        let inner = if needle.is_empty() {
            StrSearcherImpl::Empty(EmptyNeedle {
                skip_before_fw_match: false,
                skip_before_bw_match: false,
                done: false,
            })
        } else {
            StrSearcherImpl::RareChar(RareChar::new(needle))
        };
        Self { needle, inner }
    }
}

impl<'a> Sealed for StrSearcher<'a> {}
impl<'a> Searcher for StrSearcher<'a> {
    #[inline]
    fn next_match_from_start(&mut self, haystack: &AinStr) -> Option<Range<usize>> {
        match self.inner {
            StrSearcherImpl::Empty(ref mut empty) => {
                if empty.done {
                    return None;
                }
                if empty.skip_before_fw_match {
                    // We should not end up with a zero-len haystack with skip_before_fw_match =
                    // true without also setting done = true unless the caller is violating the
                    // rules about what to pass to `next_match_from_start`.
                    debug_assert!(!haystack.is_empty());
                    if haystack.len() == 1 {
                        empty.done = true;
                    }
                    Some(1..1)
                } else {
                    if haystack.is_empty() {
                        empty.done = true;
                    }
                    // From now on forward searching will try match at index 1.
                    empty.skip_before_fw_match = true;
                    Some(0..0)
                }
            }
            StrSearcherImpl::RareChar(rare) => {
                if haystack.len() < self.needle.len() {
                    return None;
                }
                // If len is 1, we can divert to char searcher.
                if self.needle.len() == 1 {
                    return self.needle[0]
                        .into_searcher()
                        .next_match_from_start(haystack);
                }
                if rare.ch.is_alphabetic() {
                    rare_match_from_start::<UncasedMemchr>(rare, self.needle, haystack)
                } else {
                    rare_match_from_start::<CasedMemchr>(rare, self.needle, haystack)
                }
            }
        }
    }

    #[inline]
    fn next_match_from_end(&mut self, haystack: &AinStr) -> Option<Range<usize>> {
        match self.inner {
            StrSearcherImpl::Empty(ref mut empty) => {
                if empty.done {
                    return None;
                }
                if empty.skip_before_bw_match {
                    // We should not end up with a zero-len haystack with skip_before_fw_match =
                    // true without also setting done = true unless the caller is violating the
                    // rules about what to pass to `next_match_from_start`.
                    debug_assert!(!haystack.is_empty());
                    if haystack.len() == 1 {
                        empty.done = true;
                    }
                    let idx = haystack.len() - 1;
                    Some(idx..idx)
                } else {
                    if haystack.is_empty() {
                        empty.done = true;
                    }
                    // From now on forward searching will try match at index 1.
                    empty.skip_before_bw_match = true;
                    let idx = haystack.len();
                    Some(idx..idx)
                }
            }
            StrSearcherImpl::RareChar(rare) => {
                if haystack.len() < self.needle.len() {
                    return None;
                }
                // If len is 1, we can divert to char searcher.
                if self.needle.len() == 1 {
                    return self.needle[0].into_searcher().next_match_from_end(haystack);
                }
                if rare.ch.is_alphabetic() {
                    rare_match_from_end::<UncasedMemchr>(rare, self.needle, haystack)
                } else {
                    rare_match_from_end::<CasedMemchr>(rare, self.needle, haystack)
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
enum StrSearcherImpl {
    /// Needle is empty, so we track state to control when to advance and when to return the empty
    /// match.
    Empty(EmptyNeedle),
    /// Needle is non-empty: scan for its rarest character and verify each candidate position.
    RareChar(RareChar),
}

#[derive(Debug, Clone)]
struct EmptyNeedle {
    /// If the str to match is empty, then every position matches. We use this flag to decide
    /// whether to skip forward when choosing the next match index.
    skip_before_fw_match: bool,
    /// Works like skip_beforw_fw_match but for reverse matching.
    skip_before_bw_match: bool,
    /// Used to handle when we get down to an empty needle. We *could* track this based on the skip
    /// fw and skip bw states and the length of the remaining haystack, but its much easier to
    /// reason about if we just have a single done flag.
    done: bool,
}

/// The character of the needle expected to be rarest in typical haystacks (see [`rank`]), and its
/// offset within the needle.
///
/// The haystack is scanned for this one character with memchr, and the whole needle is compared
/// at each position found. Choosing the rarest character keeps the number of stops (and thus
/// comparisons) low: scanning for `z` in `"Elizabeth"` stops far less often than scanning for `e`.
#[derive(Debug, Clone, Copy)]
struct RareChar {
    ch: AinChar,
    offset: usize,
}

impl RareChar {
    /// Picks the lowest-ranked character of a non-empty needle (the first such character on ties).
    #[inline]
    fn new(needle: &AinStr) -> Self {
        needle
            .chars()
            .enumerate()
            .map(|(offset, ch)| RareChar { ch, offset })
            .min_by_key(|rare| rank(rare.ch))
            .expect("StrSearcher only uses RareChar for non-empty needles")
    }

    /// The part of `haystack` where the rare character of an actual match can be found.
    ///
    /// A match starting at `pos` has its rare character at `pos + offset`, and `pos` ranges over
    /// `0..=haystack.len() - needle_len`, so the rare character of any match lies within
    /// `haystack[offset..=haystack.len() - needle_len + offset]`. Conveniently, the index of a
    /// rare character *within that window* is exactly the start index of the candidate match.
    ///
    /// The caller must ensure `haystack.len() >= needle_len`.
    #[inline(always)]
    fn window(self, needle_len: usize, haystack: &AinStr) -> &AinStr {
        &haystack[self.offset..=haystack.len() - needle_len + self.offset]
    }
}

/// Forward search for needles of at least one character; the caller must ensure
/// `haystack.len() >= needle.len()`.
#[inline(always)]
fn rare_match_from_start<M: MemchrMode>(
    rare: RareChar,
    needle: &AinStr,
    haystack: &AinStr,
) -> Option<Range<usize>> {
    let window = rare.window(needle.len(), haystack);
    // Index within the window of the next candidate to consider. This is also the index within the
    // haystack of the start of the corresponding needle.
    let mut pos = 0;
    while pos < window.len() {
        // Probe one character before paying for a memchr call: memchr has a fixed per-call cost
        // far above a single character compare, and when candidates are adjacent (a run of the
        // rare character) that cost would otherwise be paid for every position.
        if window[pos] != rare.ch {
            pos += M::memchr(rare.ch, &window[pos..])?;
        }
        if haystack[pos..pos + needle.len()] == needle {
            return Some(pos..pos + needle.len());
        }
        pos += 1;
    }
    None
}

/// Reverse search for needles of at least one character; the caller must ensure
/// `haystack.len() >= needle.len()`.
#[inline(always)]
fn rare_match_from_end<M: MemchrMode>(
    rare: RareChar,
    needle: &AinStr,
    haystack: &AinStr,
) -> Option<Range<usize>> {
    let window = rare.window(needle.len(), haystack);
    let mut pos = window.len();
    while pos > 0 {
        pos -= 1;
        // Same one-character probe as the forward search.
        if window[pos] != rare.ch {
            pos = M::memrchr(rare.ch, &window[..pos])?;
        }
        if haystack[pos..pos + needle.len()] == needle {
            return Some(pos..pos + needle.len());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rare(needle: &str) -> (char, usize) {
        let rare = RareChar::new(AinStr::from_str(needle).unwrap());
        (rare.ch.to_char(), rare.offset)
    }

    #[test]
    fn rare_char_prefers_uncommon_characters_and_the_first_on_ties() {
        assert_eq!(rare("Elizabeth"), ('z', 3));
        assert_eq!(rare("the quick brown fox"), ('q', 4));
        assert_eq!(rare("aaa"), ('a', 0));
        assert_eq!(rare("zz"), ('z', 0));
        // Ranking folds case, but the character is kept as written.
        assert_eq!(rare("ZaB"), ('Z', 0));
        assert_eq!(rare("a"), ('a', 0));
    }
}
