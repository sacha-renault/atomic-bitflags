//! Atomic storage for a [`bitflags`] flag set.
//!
//! [`AtomicFlags<F>`] keeps the bits of a `bitflags` type in an atomic
//! integer, so a flag set can be shared between threads without a lock.
//!
//! Every operation on the stored value takes an [`Ordering`] under the same
//! rules as the standard library atomics. Flags wider than 64 bits are not
//! supported, and 64-bit atomics are not available on every target.
//!
//! # Examples
//!
//! ```
//! use atomic_bitflags::AtomicFlags;
//! use bitflags::bitflags;
//! use std::sync::Arc;
//! use std::sync::atomic::Ordering;
//!
//! bitflags! {
//!     #[derive(Debug, Clone, Copy, PartialEq, Eq)]
//!     struct Features: u8 {
//!         const FAST = 0b001;
//!         const SAFE = 0b010;
//!     }
//! }
//!
//! let flags = Arc::new(AtomicFlags::new(Features::FAST));
//! let remote = Arc::clone(&flags);
//!
//! std::thread::spawn(move || {
//!     remote.set(Features::SAFE, Ordering::Relaxed);
//! })
//! .join()
//! .unwrap();
//!
//! let current = flags.load(Ordering::Relaxed);
//! assert!(current.contains(Features::FAST | Features::SAFE));
//! ```

use std::fmt;
use std::sync::atomic::Ordering;

use bitflags::Flags;

mod atomic_bits;
#[cfg(test)]
mod tests;

pub use atomic_bits::AtomicBits;

/// A flag set stored in an atomic integer.
pub struct AtomicFlags<F>
where
    F: Flags,
    F::Bits: AtomicBits,
    F: Copy,
{
    storage: <F::Bits as AtomicBits>::Storage,
}

impl<F> Default for AtomicFlags<F>
where
    F: Flags,
    F::Bits: AtomicBits,
    F: Copy,
    F: Default,
{
    fn default() -> Self {
        Self::new(F::default())
    }
}

impl<F> fmt::Debug for AtomicFlags<F>
where
    F: Flags,
    F::Bits: AtomicBits,
    F: Copy,
    F: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.load(Ordering::SeqCst).fmt(f)
    }
}

impl<F> AtomicFlags<F>
where
    F: Flags,
    F::Bits: AtomicBits,
    F: Copy,
{
    #[inline]
    pub fn new(flags: F) -> Self {
        Self::from_storage(F::Bits::new(flags.bits()))
    }

    /// Const constructor from an already-built atomic.
    ///
    /// Less ergonomic than `new` but allow to build this struct from a const context
    /// (i.e. static flags).
    ///
    /// ```
    /// use std::sync::atomic::AtomicU8;
    /// use atomic_bitflags::AtomicFlags;
    /// use bitflags::bitflags;
    ///
    /// bitflags! {
    ///     #[derive(Clone, Copy)]
    ///     struct Features: u8 {
    ///         const CACHE = 1 << 0;
    ///     }
    /// }
    ///
    /// static FEATURES: AtomicFlags<Features> =
    ///     AtomicFlags::from_storage(AtomicU8::new(Features::CACHE.bits()));
    /// ```
    pub const fn from_storage(storage: <F::Bits as AtomicBits>::Storage) -> Self {
        Self { storage }
    }

    #[inline]
    pub fn store(&self, flags: F, ordering: Ordering) {
        F::Bits::store(&self.storage, flags.bits(), ordering);
    }

    #[inline]
    pub fn load(&self, ordering: Ordering) -> F {
        F::from_bits_retain(F::Bits::load(&self.storage, ordering))
    }

    /// Atomically replaces the value with `current & flags` returning the previous value.
    #[inline]
    pub fn fetch_and(&self, flags: F, ordering: Ordering) -> F {
        F::from_bits_retain(F::Bits::fetch_and(&self.storage, flags.bits(), ordering))
    }

    /// Atomically replaces the value with `current | flags`, returning the
    /// previous value.
    #[inline]
    pub fn fetch_or(&self, flags: F, ordering: Ordering) -> F {
        F::from_bits_retain(F::Bits::fetch_or(&self.storage, flags.bits(), ordering))
    }

    /// Atomically toggles `flags`, returning the previous value.
    #[inline]
    pub fn fetch_xor(&self, flags: F, ordering: Ordering) -> F {
        F::from_bits_retain(F::Bits::fetch_xor(&self.storage, flags.bits(), ordering))
    }

    /// Synonymous with `fetch_xor`.
    #[inline]
    pub fn toggle(&self, flags: F, ordering: Ordering) -> F {
        self.fetch_xor(flags, ordering)
    }

    /// Atomically replaces the value with `!(current & flags)`, returning the
    /// previous value.
    ///
    /// This follows the standard atomic `fetch_nand`: the result is not
    /// restricted to `flags`, so bits outside the mask are set.
    #[inline]
    pub fn fetch_nand(&self, flags: F, ordering: Ordering) -> F {
        F::from_bits_retain(F::Bits::fetch_nand(&self.storage, flags.bits(), ordering))
    }

    /// Atomically replaces the value with `flags`, returning the previous value.
    #[inline]
    pub fn swap(&self, flags: F, ordering: Ordering) -> F {
        F::from_bits_retain(F::Bits::swap(&self.storage, flags.bits(), ordering))
    }

    /// Atomically stores `new` if the current value is `current`.
    ///
    /// Returns `Ok(previous)` on success and `Err(actual)` on failure.
    ///
    /// # Panics
    ///
    /// Panics if `failure` is stronger than `success`.
    #[inline]
    pub fn compare_exchange(
        &self,
        current: F,
        new: F,
        success: Ordering,
        failure: Ordering,
    ) -> Result<F, F> {
        F::Bits::compare_exchange(&self.storage, current.bits(), new.bits(), success, failure)
            .map(F::from_bits_retain)
            .map_err(F::from_bits_retain)
    }

    /// Like [`compare_exchange`](Self::compare_exchange), but may fail
    /// spuriously.
    ///
    /// Use it inside a retry loop where a spurious failure is harmless; prefer
    /// [`compare_exchange`](Self::compare_exchange) otherwise.
    ///
    /// # Panics
    ///
    /// Panics if `failure` is stronger than `success`.
    #[inline]
    pub fn compare_exchange_weak(
        &self,
        current: F,
        new: F,
        success: Ordering,
        failure: Ordering,
    ) -> Result<F, F> {
        F::Bits::compare_exchange_weak(&self.storage, current.bits(), new.bits(), success, failure)
            .map(F::from_bits_retain)
            .map_err(F::from_bits_retain)
    }

    /// Atomically updates the value using `f`, returning the previous value.
    ///
    /// `f` receives the current value and returns the new one, or `None` to
    /// abort. It may run more than once if another thread changes the value.
    ///
    /// Returns `Ok(previous)` on success and `Err(current)` if `f` returns
    /// `None`.
    ///
    /// # Panics
    ///
    /// Panics if `fetch_order` is `Release` or `AcqRel`, or stronger than
    /// `set_order`.
    #[inline]
    pub fn try_update<Op>(
        &self,
        set_order: Ordering,
        fetch_order: Ordering,
        mut f: Op,
    ) -> Result<F, F>
    where
        Op: FnMut(F) -> Option<F>,
    {
        let mut previous = self.load(fetch_order);
        loop {
            let new = match f(previous) {
                Some(new) => new,
                None => return Err(previous),
            };
            match self.compare_exchange_weak(previous, new, set_order, fetch_order) {
                Ok(previous) => return Ok(previous),
                Err(actual) => previous = actual,
            }
        }
    }

    /// Atomically clears `flags` and returns the subset that was set.
    ///
    /// Returns an empty set if none of `flags` were set. Other bits are left
    /// untouched.
    #[inline]
    pub fn take(&self, flags: F, ordering: Ordering) -> F {
        let complement = Self::retained_complement(flags);
        let previous = self.fetch_and(complement, ordering);
        previous.intersection(flags)
    }

    /// Returns the subset of `flags` that is currently set.
    ///
    /// Does not modify the stored value. The result is a snapshot from a single
    /// load; use [`take`](Self::take) to claim flags atomically.
    #[inline]
    pub fn peek(&self, flags: F, ordering: Ordering) -> F {
        self.load(ordering).intersection(flags)
    }

    /// Clears all flags.
    #[inline]
    pub fn clear(&self, ordering: Ordering) {
        self.store(F::empty(), ordering);
    }

    /// Consumes the atomic and returns the flags.
    #[inline]
    pub fn into_inner(self) -> F {
        F::from_bits_retain(F::Bits::into_inner(self.storage))
    }

    /// Atomically sets the bits in `flags`, returning the previous value.
    #[inline]
    pub fn set(&self, flags: F, ordering: Ordering) -> F {
        self.fetch_or(flags, ordering)
    }

    /// Atomically clears the bits in `flags`, returning the previous value.
    #[inline]
    pub fn unset(&self, flags: F, ordering: Ordering) -> F {
        let complement = Self::retained_complement(flags);
        self.fetch_and(complement, ordering)
    }

    /// Return the complement with retained instead of truncated mode.
    ///
    /// This is useful for take & unset that would otherwise clear unknown
    /// bits, contradicting the documentation.
    #[inline]
    fn retained_complement(flags: F) -> F {
        F::from_bits_retain(!flags.bits())
    }
}
