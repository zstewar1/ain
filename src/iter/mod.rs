use core::iter::FusedIterator;
use core::slice::{Iter, IterMut};

use crate::pattern::{DoubleEndedPattern, LineEnd, LineEndSearcher, Pattern, SearchHolder};
use crate::{AinChar, AinStr};

#[cfg(feature = "alloc")]
pub use self::alloc_iter::IntoChars;

#[cfg(feature = "alloc")]
mod alloc_iter;

/// A copying iterator over the characters of an `AinStr`.
#[derive(Clone, Debug)]
#[must_use]
pub struct Chars<'a> {
    pub(crate) inner: Iter<'a, AinChar>,
}

impl<'a> Chars<'a> {
    /// Returns the ascii string slice with the remaining characters.
    #[must_use]
    #[inline]
    pub fn as_ain_str(&self) -> &'a AinStr {
        AinStr::from_slice(self.inner.as_slice())
    }
}

impl<'a> Iterator for Chars<'a> {
    type Item = AinChar;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().copied()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.inner.count()
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        self.inner.last().copied()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.inner.nth(n).copied()
    }

    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.inner.copied().fold(init, f)
    }
}

impl<'a> DoubleEndedIterator for Chars<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().copied()
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.inner.nth_back(n).copied()
    }

    #[inline]
    fn rfold<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.inner.copied().rfold(init, f)
    }
}

impl<'a> ExactSizeIterator for Chars<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'a> FusedIterator for Chars<'a> {}

/// A mutable iterator over the characters of an `AinStr`.
#[derive(Debug)]
#[must_use]
pub struct CharsMut<'a> {
    pub(crate) inner: IterMut<'a, AinChar>,
}

impl<'a> CharsMut<'a> {
    /// Returns the ascii string slice with the remaining characters, using the original lifetime by
    /// destroying the iterator.
    #[must_use]
    #[inline]
    pub fn into_ain_str(self) -> &'a mut AinStr {
        AinStr::from_slice_mut(self.inner.into_slice())
    }

    /// Returns the ascii string slice with the remaining characters.
    #[must_use]
    #[inline]
    pub fn as_ain_str(&self) -> &AinStr {
        AinStr::from_slice(self.inner.as_slice())
    }
}

impl<'a> Iterator for CharsMut<'a> {
    type Item = &'a mut AinChar;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.inner.count()
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        self.inner.last()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.inner.nth(n)
    }

    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.inner.fold(init, f)
    }
}

impl<'a> DoubleEndedIterator for CharsMut<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.inner.nth_back(n)
    }

    #[inline]
    fn rfold<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.inner.rfold(init, f)
    }
}

impl<'a> ExactSizeIterator for CharsMut<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'a> FusedIterator for CharsMut<'a> {}

macro_rules! split_impl {
    (
        for <$lt:lifetime, $sty:ty> {
            $(#[$split_attr:meta])*
            $Split:ident;

            $(#[$split_n_attr:meta])*
            $SplitN:ident;
        }
        where next = $next:ident,
            next_back = $next_back:ident $(,)?
    ) => {
        $(#[$split_attr])*
        #[must_use]
        pub struct $Split<$lt, P: Pattern> {
            searcher: SearchHolder<P::Searcher, $sty>,
        }

        impl<$lt, P:Pattern> $Split<$lt, P> {
            pub(crate) fn new(str: $sty, pattern: P) -> Self {
                Self {
                    searcher: SearchHolder::new(pattern, str),
                }
            }
        }

        impl<$lt, P: Pattern> Iterator for $Split<$lt, P> {
            type Item = $sty;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                self.searcher.$next()
            }
        }

        impl<$lt, P: DoubleEndedPattern> DoubleEndedIterator for $Split<$lt, P> {
            #[inline]
            fn next_back(&mut self) -> Option<Self::Item> {
                self.searcher.$next_back()
            }
        }

        impl<$lt, P: Pattern> FusedIterator for $Split<$lt, P> {}

        $(#[$split_n_attr])*
        #[must_use]
        pub struct $SplitN<$lt, P: Pattern> {
            searcher: SearchHolder<P::Searcher, $sty>,
            emitted: usize,
            limit: usize,
        }

        impl<$lt, P: Pattern> $SplitN<$lt, P> {
            /// Create a SplitN over the given string with the given pattern and limit.
            #[inline]
            pub(crate) fn new(str: $sty, limit: usize, pattern: P) -> Self {
                Self {
                    searcher: SearchHolder::new(pattern, str),
                    emitted: 0,
                    limit,
                }
            }
        }

        impl<$lt, P: Pattern> Iterator for $SplitN<$lt, P> {
            type Item = $sty;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                if self.emitted < self.limit {
                    self.emitted += 1;
                    if self.emitted == self.limit {
                        self.searcher.take_remainder()
                    } else {
                        self.searcher.$next()
                    }
                } else {
                    None
                }
            }
        }

        impl<$lt, P: Pattern> FusedIterator for $SplitN<$lt, P> {}
    };
}

split_impl! {
    for <'a, &'a AinStr> {
        /// Splits a str into pieces separated by a [`Pattern`].
        ///
        /// This type is created by [`AinStr::split`]
        #[derive(Debug, Clone)]
        Split;
        /// Splits a str into at most `n` pieces separated by a [`Pattern`].
        ///
        /// This type is created by [`AinStr::splitn`]
        #[derive(Debug, Clone)]
        SplitN;
    }
    where next = split_next_impl,
        next_back = rsplit_next_impl,
}

split_impl! {
    for <'a, &'a mut AinStr> {
        /// Splits a str into pieces separated by a [`Pattern`].
        ///
        /// This type is created by [`AinStr::split_mut`]
        #[derive(Debug)]
        SplitMut;
        /// Splits a str into at most `n` pieces separated by a [`Pattern`].
        ///
        /// This type is created by [`AinStr::splitn_mut`]
        #[derive(Debug)]
        SplitNMut;
    }
    where next = split_next_impl,
        next_back = rsplit_next_impl,
}

split_impl! {
    for <'a, &'a AinStr> {
        /// Splits a str into pieces separated by a [`Pattern`].
        ///
        /// This type is created by [`AinStr::rsplit`]
        #[derive(Debug, Clone)]
        RSplit;
        /// Splits a str into at most `n` pieces separated by a [`Pattern`].
        ///
        /// This type is created by [`AinStr::rsplitn`]
        #[derive(Debug, Clone)]
        RSplitN;
    }
    where next = rsplit_next_impl,
        next_back = split_next_impl,
}

split_impl! {
    for <'a, &'a mut AinStr> {
        /// Splits a str into pieces separated by a [`Pattern`].
        ///
        /// This type is created by [`AinStr::rsplit_mut`]
        #[derive(Debug)]
        RSplitMut;
        /// Splits a str into at most `n` pieces separated by a [`Pattern`].
        ///
        /// This type is created by [`AinStr::rsplitn_mut`]
        #[derive(Debug)]
        RSplitNMut;
    }
    where next = rsplit_next_impl,
        next_back = split_next_impl,
}

/// An iterator over the lines of the internal character array.
#[derive(Clone, Debug)]
#[must_use]
pub struct Lines<'a> {
    searcher: SearchHolder<LineEndSearcher, &'a AinStr>,
}
impl<'a> Lines<'a> {
    #[inline]
    pub(crate) fn new(s: &'a AinStr) -> Self {
        // To match std, an empty str is considered to have no lines, so we create the searcher in
        // the already-done state.
        if s.is_empty() {
            return Self {
                searcher: LineEnd::create_done_searcher(),
            };
        }
        // Since a trailing newline doesn't create an extra blank line (to match std), strip a
        // trailing newline now.
        let s = s.trim_suffix(LineEnd);
        Self {
            searcher: LineEnd::search(s),
        }
    }
}
impl<'a> Iterator for Lines<'a> {
    type Item = &'a AinStr;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.searcher.split_next_impl()
    }
}

impl<'a> DoubleEndedIterator for Lines<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.searcher.rsplit_next_impl()
    }
}

impl<'a> FusedIterator for Lines<'a> {}

/// An iterator over mutable lines of the internal character array.
#[derive(Debug)]
#[must_use]
pub struct LinesMut<'a> {
    searcher: SearchHolder<LineEndSearcher, &'a mut AinStr>,
}
impl<'a> LinesMut<'a> {
    #[inline]
    pub(crate) fn new(s: &'a mut AinStr) -> Self {
        // To match std, an empty str is considered to have no lines, so we create the searcher in
        // the already-done state.
        if s.is_empty() {
            return Self {
                searcher: LineEnd::create_done_searcher(),
            };
        }
        // Since a trailing newline doesn't create an extra blank line (to match std), strip a
        // trailing newline now.
        let s = s.trim_suffix_mut(LineEnd);
        Self {
            searcher: LineEnd::search(s),
        }
    }
}
impl<'a> Iterator for LinesMut<'a> {
    type Item = &'a mut AinStr;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.searcher.split_next_impl()
    }
}

impl<'a> DoubleEndedIterator for LinesMut<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.searcher.rsplit_next_impl()
    }
}

impl<'a> FusedIterator for LinesMut<'a> {}
