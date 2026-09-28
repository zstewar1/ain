use core::mem;

use crate::AinStr;

/// Utility for splitting &AinStr or &mut AinStr.
pub(super) trait PartitionHaystack: Sized + AsRef<AinStr> {
    /// Splits this haystack at the given index.
    fn split_at(self, idx: usize) -> (Self, Self);

    /// Splits this haystack. Returns the portion from 0..idx and sets self to idx..len()
    fn take_before(&mut self, idx: usize) -> Self;

    /// Splits this haystack. Returns the portion from idx..len() and sets self to 0..idx()
    fn take_after(&mut self, idx: usize) -> Self;

    /// Returns this haystack, replacing it with empty.
    fn take(&mut self) -> Self;
}

impl PartitionHaystack for &AinStr {
    #[inline(always)]
    fn split_at(self, idx: usize) -> (Self, Self) {
        // Spell out that this calls the intrinsic method of AinStr rather than the trait method of
        // &AinStr.
        AinStr::split_at(self, idx)
    }

    #[inline(always)]
    fn take_before(&mut self, idx: usize) -> Self {
        let before;
        (before, *self) = self.split_at(idx);
        before
    }

    #[inline(always)]
    fn take_after(&mut self, idx: usize) -> Self {
        let after;
        (*self, after) = self.split_at(idx);
        after
    }

    #[inline(always)]
    fn take(&mut self) -> Self {
        mem::take(self)
    }
}

impl PartitionHaystack for &mut AinStr {
    #[inline(always)]
    fn split_at(self, idx: usize) -> (Self, Self) {
        self.split_at_mut(idx)
    }

    #[inline(always)]
    fn take_before(&mut self, idx: usize) -> Self {
        let (before, this) = self.take().split_at_mut(idx);
        *self = this;
        before
    }

    #[inline(always)]
    fn take_after(&mut self, idx: usize) -> Self {
        let (this, after) = self.take().split_at_mut(idx);
        *self = this;
        after
    }

    #[inline(always)]
    fn take(&mut self) -> Self {
        mem::take(self)
    }
}
