use core::ops::Range;

use crate::AinStr;

pub use self::char_pattern::CharSearcher;
pub use self::char_set_pattern::CharSetSearcher;
pub use self::fnmut_pattern::FnMutSearcher;
use self::haystack::PartitionHaystack;
pub use self::lineend_pattern::{LineEnd, LineEndSearcher};
pub use self::str_pattern::StrSearcher;

trait Sealed {}

mod byte_rank;
mod char_pattern;
mod char_set_pattern;
mod fnmut_pattern;
mod haystack;
mod lineend_pattern;
mod memchr;
mod str_pattern;

/// A string pattern.
///
/// A `Pattern` expresses that the implementing type can be used as a string pattern for searching
/// in a [`&AinStr`][AinStr].
///
/// For example, both `ain!('a')` and `ain!("aa")` are patterns that would match at index `1` in the
/// string `ain!("bAaaab")`.
///
/// Depending on the type of the pattern, the behavior of methods like [`AinStr::find`] and
/// [`AinStr::contains`] can change. The table below describes some of those behaviors.
///
/// | Pattern type                | Match condition                                             |
/// |-----------------------------|-------------------------------------------------------------|
/// | `&AinStr`                   | is substring (case insensitive)                             |
/// | `AinChar`                   | is contained in string (case insensitive)                   |
/// | `&[AinChar]`                | any char in slice is contained in string (case insensitive) |
/// | `F: FnMut(AinChar) -> bool` | `F` returns `true` for a char in string                     |
/// | `&&AinStr`                  | is substring (case insensitive)                             |
/// | `&AinString`                | is substring (case insensitive)                             |
///
/// # Examples
///
/// ```
/// # use ain::{ain, AinChar};
/// // &AinStr
/// assert_eq!(ain!("abaaa").find(ain!("BA")), Some(1));
/// assert_eq!(ain!("abaaa").find(ain!("bac")), None);
///
/// // AinChar
/// assert_eq!(ain!("abaaa").find(ain!('A')), Some(0));
/// assert_eq!(ain!("abaaa").find(ain!('b')), Some(1));
/// assert_eq!(ain!("abaaa").find(ain!('c')), None);
///
/// // &[AinChar; N]
/// assert_eq!(ain!("ab").find(&[ain!('b'), ain!('A')]), Some(0));
/// assert_eq!(ain!("ABAAA").find(&[ain!('a'), ain!('z')]), Some(0));
/// assert_eq!(ain!("abaaa").find(&[ain!('c'), ain!('d')]), None);
///
/// // &[AinChar]
/// assert_eq!(ain!("ab").find(&[ain!('b'), ain!('A')][..]), Some(0));
/// assert_eq!(ain!("ABAAA").find(&[ain!('a'), ain!('z')][..]), Some(0));
/// assert_eq!(ain!("abaaa").find(&[ain!('c'), ain!('d')][..]), None);
///
/// // FnMut(AinChar) -> bool
/// assert_eq!(ain!("ABCDEF_Z").find(|ch| ch > ain!('d') && ch < ain!('y')), Some(4));
/// assert_eq!(ain!("abcddd_z").find(|ch| ch > ain!('d') && ch < ain!('y')), None);
/// // FnMut(AinChar) -> bool can force case sensitivity:
/// assert_eq!(ain!("abcddd_z").find(|ch: AinChar| ch.to_u8() == b'A'), None);
/// ```
#[allow(private_bounds)]
pub trait Pattern: Sized + Sealed {
    /// Searcher used for this pattern.
    type Searcher: Searcher;

    /// Convert this needle into the corresponding searcher type.
    fn into_searcher(self) -> Self::Searcher;

    /// If this pattern matches the start of the given haystack, returns the index which splits the
    /// haystack between that single match and the rest. Otherwise returns None if there is no match
    /// anchored at the start of the haystack.
    ///
    /// The returned index splits the haystack:
    /// `(matched, rest) = (&haystack[..idx], &haystack[idx..]).
    fn start_match_split_point(&mut self, haystack: &AinStr) -> Option<usize>;

    /// Matches this pattern zero or more times contiguously at the start of the haystack and
    /// returns the index which splits the haystack into the contiguous matches of this pattern snd
    /// the rest. If there are no matches (or the pattern is empty), returns 0.
    ///
    /// The returned index splits the haystack:
    /// `(matched, rest) = (&haystack[..idx], &haystack[idx..]).
    fn start_matches_split_point(&mut self, haystack: &AinStr) -> usize;

    /// If this pattern matches the end of the given haystack, returns the index which splits the
    /// haystack between that single match and the rest. Otherwise returns None if there is no match
    /// anchored at the start of the haystack.
    ///
    /// The returned index splits the haystack:
    /// `(rest, matched) = (&haystack[..idx], &haystack[idx..]).
    fn end_match_split_point(&mut self, haystack: &AinStr) -> Option<usize>;

    /// Matches this pattern zero or more times contiguously at the end of the haystack and returns
    /// the index which splits the haystack into the contiguous matches of this pattern and the
    /// rest. If there are no matches (or the pattern is empty), returns `haystack.len()`.
    ///
    /// The returned index partitions the haystack:
    /// `(rest, matched) = (&haystack[..idx], &haystack[idx..]).
    fn end_matches_split_point(&mut self, haystack: &AinStr) -> usize;
}

/// Marker trait that indicates that a [`Pattern`] will produce the same match ranges when
/// iterating forward as iterating backwards. If forward and backward matching produce the same
/// result, it like [split][AinStr::split] can implement [`DoubleEndedIterator`].
///
/// Matching on a [`&AinStr`][AinStr] is not guaranteed to produce the same match ranges: for
/// example, matching `"aa"` against `"aaa"` produces range `0..2` when matching forwards but range
/// `1..3` when matching backwards. So we avoid implementing `DoubleEndedIterator` since it would
/// produce inconsistent results depending on the order of calls between [`next`][Iterator::next]
/// and [`next_back`][DoubleEndedIterator::next_back].
///
/// Note that FnMut patterns implement DoubleEndedPattern since they match on a single character at
/// a time, even though the mutability could be used to cause incosistent forward-backward
/// operation. Any such inconsistency due to mutation belongs to the caller; if the caller provides
/// a pure function, then forward and backward iteration do produce the same results.
pub trait DoubleEndedPattern: Pattern {}

/// Manages internal state or auxiliary data used to implement the search for a given pattern. For
/// example, the Searcher type may store data derived from the needle which helps accelerate the
/// search.
///
/// When a search starts, the entire `haystack` should be passed to
/// [`next_match_from_start`][Searcher::next_match_from_start] or
/// [`next_match_from_end`][Searcher::next_match_from_end]. After each call, everything from the
/// start (or end, depending on ddirection) of the haystack through the match should be removed
/// before passing the remainder to the next call to `next_match_from_start` or
/// `next_match_from_end`.
///
/// That is, if `next_match_from_start` returns some range `match_start..match_end`, then the next
/// call to either `next_match_from_start` or `next_match_from_end` should be provided only
/// `haystack[match_end..]`.
///
/// Conversely if `next_match_from_end` returns some range `match_start..match_end`, then the next
/// call to either `next_match_from_start` or `next_match_from_end` should be provided only
/// `haystack[..match_start]`.
#[allow(private_bounds)]
pub trait Searcher: Sized + Sealed {
    /// Search the given [`&AinStr`][AinStr] for the next match of this pattern starting from the
    /// beginning. Returns the first matching range.
    ///
    /// Everything up to the start of the returned range is rejected as non-matching. The provided
    /// range is matched.
    fn next_match_from_start(&mut self, haystack: &AinStr) -> Option<Range<usize>>;

    /// Search the given [`&AinStr`][AinStr] for the next match of this pattern starting from the
    /// end. Returns the first matching range from the back (that is, the last place in the string
    /// that could match).
    ///
    /// Everything after the end of the returned range is rejected as non-matching. The provided
    /// range is matched.
    fn next_match_from_end(&mut self, haystack: &AinStr) -> Option<Range<usize>>;
}

/// Holds pattern searching state along with the non-rejected portion of the haystack.
#[derive(Copy, Clone, Debug)]
pub(crate) struct SearchHolder<S, H> {
    searcher: S,
    haystack: H,
    // Set when a next_match/next_back_match produces None to indicate that the empty haystack
    // represents a 'done' state rather than an empty search.
    done: bool,
}

impl<S, H> SearchHolder<S, H> {
    #[inline]
    pub(crate) fn new<P: Pattern<Searcher = S>>(pattern: P, haystack: H) -> Self {
        Self {
            searcher: pattern.into_searcher(),
            haystack,
            done: false,
        }
    }
}

impl<S, H: Default> SearchHolder<S, H> {
    /// Create a new Searcher in the already-finished state.
    #[inline]
    pub(crate) fn new_done<P: Pattern<Searcher = S>>(pattern: P) -> Self {
        Self {
            searcher: pattern.into_searcher(),
            haystack: Default::default(),
            done: true,
        }
    }
}

impl<S: Searcher, H: AsRef<AinStr>> SearchHolder<S, H> {
    /// Computes the next match but does not advance the haystack.
    #[inline(always)]
    fn next_match_from_start(&mut self) -> Option<Range<usize>> {
        self.searcher.next_match_from_start(self.haystack.as_ref())
    }

    /// Computes the next back match but does not advance the haystack.
    #[inline(always)]
    fn next_match_from_end(&mut self) -> Option<Range<usize>> {
        self.searcher.next_match_from_end(self.haystack.as_ref())
    }
}

#[allow(private_bounds)]
impl<S: Searcher, H: PartitionHaystack> SearchHolder<S, H> {
    /// Take whatever is left in this search holder, if it isn't done.
    ///
    /// This does not update any other searcher state because it is intended to be used only when
    /// you are never going to make another call to the searcher. Calling searcher methods after
    /// this may produce unexpected results.
    #[inline]
    pub(crate) fn take_remainder(&mut self) -> Option<H> {
        if self.done {
            return None;
        }
        self.done = true;
        Some(self.haystack.take())
    }

    /// Find the first match's start index.
    ///
    /// This is intended to be the only call on this search holder so doesn't check if done.
    #[inline]
    pub(crate) fn find_impl(mut self) -> Option<usize> {
        // Since this consumes the StateHolder, don't bother advancing the haystack.
        self.next_match_from_start().map(|range| range.start)
    }

    /// Find the last match's start index.
    ///
    /// This is intended to be the only call on this search holder so doesn't check if done.
    #[inline]
    pub(crate) fn rfind_impl(mut self) -> Option<usize> {
        // Since this consumes the StateHolder, don't bother advancing the haystack.
        self.next_match_from_end().map(|range| range.start)
    }

    /// Implements splitting iterators splitting from the front of the string.
    #[inline]
    pub(crate) fn split_next_impl(&mut self) -> Option<H> {
        if self.done {
            return None;
        }
        let matching = match self.next_match_from_start() {
            Some(matching) => matching,
            None => {
                // Wasn't done before, so split next needs to still return the tail.
                self.done = true;
                return Some(self.haystack.take());
            }
        };
        let (reject, _match) = self
            .haystack
            .take_before(matching.end)
            .split_at(matching.start);
        Some(reject)
    }

    /// Implements splitting iterators splitting from the back of the string.
    #[inline]
    pub(crate) fn rsplit_next_impl(&mut self) -> Option<H> {
        if self.done {
            return None;
        }
        let matching = match self.next_match_from_end() {
            Some(matching) => matching,
            None => {
                // Wasn't done before, so split next needs to still return the tail.
                self.done = true;
                return Some(self.haystack.take());
            }
        };
        let (_match, reject) = self
            .haystack
            .take_after(matching.start)
            .split_at(matching.end - matching.start);
        Some(reject)
    }
}
