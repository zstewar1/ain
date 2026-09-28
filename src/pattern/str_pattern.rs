use core::ops::Range;

use crate::{AinChar, AinStr};

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
        <&AinStr as Pattern>::into_searcher(&*self)
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
            StrSearcherImpl::Memchr(MemchrBloom::new(needle))
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
            StrSearcherImpl::Memchr(ref memchr) => {
                if haystack.len() < self.needle.len() {
                    return None;
                }
                // If len is 1, we can divert to char searcher.
                if self.needle.len() == 1 {
                    return self.needle[0]
                        .into_searcher()
                        .next_match_from_start(haystack);
                }

                /// Match impl for needles of at least two characters.
                #[inline(always)]
                fn needle2_match_impl<M: MemchrMode>(
                    first: AinChar,
                    needle: &AinStr,
                    memchr: &MemchrBloom,
                    haystack: &AinStr,
                ) -> Option<Range<usize>> {
                    // The last possible index where the needle could start.
                    let max_needle_start_index = haystack.len() - needle.len();
                    // Offset tracks how far into the haystack we've searched so far.
                    let mut offset = 0;
                    while offset <= max_needle_start_index {
                        // Note: inclusive range, because the needle could start at that index.
                        let search_space = &haystack[offset..=max_needle_start_index];
                        offset += M::memchr(first, search_space)?;
                        let potential_end = offset + needle.len();
                        let end_char = haystack[potential_end - 1];
                        if memchr.bloom_matches(end_char) {
                            if haystack[offset..potential_end] == needle {
                                return Some(offset..potential_end);
                            }
                            // If the string doesn't match the needle, but the end char was in the
                            // bloom filter set, advance by one, because we could have an
                            // overlapping match.
                            offset += 1;
                        } else {
                            // The end char wasn't in the bloom filter set at all, so we can skip
                            // forward by the length of the needle.
                            offset = potential_end;
                        }
                    }
                    None
                }

                let first = self.needle[0];
                if first.is_alphabetic() {
                    needle2_match_impl::<UncasedMemchr>(first, self.needle, memchr, haystack)
                } else {
                    needle2_match_impl::<CasedMemchr>(first, self.needle, memchr, haystack)
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
            StrSearcherImpl::Memchr(ref memchr) => {
                if haystack.len() < self.needle.len() {
                    return None;
                }
                // If len is 1, we can divert to char searcher.
                if self.needle.len() == 1 {
                    return self.needle[0].into_searcher().next_match_from_end(haystack);
                }

                /// Match impl for needles of at least two characters.
                #[inline(always)]
                fn needle2_rmatch_impl<M: MemchrMode>(
                    last: AinChar,
                    needle: &AinStr,
                    memchr: &MemchrBloom,
                    haystack: &AinStr,
                ) -> Option<Range<usize>> {
                    // The lowest index which could include the needle's end character.
                    let min_needle_end_idx = needle.len() - 1;
                    // The exclusive upper bound of characters which are still potentially being
                    // searched.
                    let mut end_offset = haystack.len();
                    while min_needle_end_idx < end_offset {
                        // Note: inclusive range, because the needle could end at that index.
                        let search_space = &haystack[min_needle_end_idx..end_offset];
                        // Add 1 here to keep this as an exclusive upper bound. In effect, this says
                        // that end_offset is the possible 'end index' of needle.
                        end_offset = min_needle_end_idx + M::memrchr(last, search_space)? + 1;
                        let potential_start = end_offset - needle.len();
                        let start_char = haystack[potential_start];
                        if memchr.bloom_matches(start_char) {
                            if haystack[potential_start..end_offset] == needle {
                                return Some(potential_start..end_offset);
                            }
                            // If the string doesn't match the needle, but the start char was in the
                            // bloom filter set, advance by one, because we could have an
                            // overlapping match.
                            end_offset -= 1;
                        } else {
                            // The start char wasn't in the bloom filter set at all, so we can skip
                            // forward by the length of the needle.
                            end_offset = potential_start;
                        }
                    }
                    None
                }

                let last = self.needle[self.needle.len() - 1];
                if last.is_alphabetic() {
                    needle2_rmatch_impl::<UncasedMemchr>(last, self.needle, memchr, haystack)
                } else {
                    needle2_rmatch_impl::<CasedMemchr>(last, self.needle, memchr, haystack)
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
    Memchr(MemchrBloom),
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

#[derive(Debug, Clone)]
struct MemchrBloom {
    /// A quick and dirty bloom filter ish thing of all the characters in the needle.
    ///
    /// Whenever we find the start character of the needle, we use this to quickly check whether the
    /// end character appears anywhere in this bitset. If it doesn't appear in this bitset, then we
    /// can skip forward by the length of the needle.
    ///
    /// We wrap the characters by masking with 0x1f, so we map 4  of each of the possible 128 bytes
    /// to each bloom filter slot. However, this wrapping is convenient for us because it happens to
    /// wrap uppercase and lowercase letters to the same slots! So For alphabetic data, the number
    /// of colliding characters is only 3, and one of them is a punctuation mark.
    bloom_filter: u32,
}

impl MemchrBloom {
    /// Create a new MemchrBloom by computing the mask over all the bytes of the needle.
    #[inline]
    fn new(needle: &AinStr) -> Self {
        let bloom_filter = needle
            .chars()
            .fold(0u32, |filter, ch| filter | Self::char_to_mask(ch));
        Self { bloom_filter }
    }

    /// Wrap the given char and use it to create a bitmask that selects the corresponding bit
    /// number.
    #[inline]
    fn char_to_mask(ch: AinChar) -> u32 {
        1 << (ch.to_u8() & 0x1f)
    }

    /// Checks if the given char matches at the filter.
    #[inline]
    fn bloom_matches(&self, ch: AinChar) -> bool {
        (self.bloom_filter & Self::char_to_mask(ch)) != 0
    }
}
