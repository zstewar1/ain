use core::iter::FusedIterator;

use alloc::vec::IntoIter;

use crate::{AinChar, AinStr};

/// A consuming iterator for an [`AinString`][crate::AinString].
#[derive(Clone, Debug)]
#[must_use]
pub struct IntoChars {
    pub(crate) inner: IntoIter<AinChar>,
}

impl IntoChars {
    /// Get an ascii string slice of the remaining characters.
    #[must_use]
    #[inline]
    pub fn as_ain_str(&self) -> &AinStr {
        AinStr::from_slice(self.inner.as_slice())
    }

    /// Get a mutable ascii string slice of the remaining characters.
    #[must_use]
    #[inline]
    pub fn as_mut_ain_str(&mut self) -> &mut AinStr {
        AinStr::from_slice_mut(self.inner.as_mut_slice())
    }
}

impl Iterator for IntoChars {
    type Item = AinChar;

    #[inline]
    fn next(&mut self) -> Option<AinChar> {
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

impl DoubleEndedIterator for IntoChars {
    #[inline]
    fn next_back(&mut self) -> Option<AinChar> {
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

impl ExactSizeIterator for IntoChars {
    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl FusedIterator for IntoChars {}
