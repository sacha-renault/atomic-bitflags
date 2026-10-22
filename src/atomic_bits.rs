use std::sync::atomic::{
    AtomicI8, AtomicI16, AtomicI32, AtomicI64, AtomicIsize, AtomicU8, AtomicU16, AtomicU32,
    AtomicU64, AtomicUsize, Ordering,
};

/// Integer type that can back an [`AtomicFlags`](crate::AtomicFlags).
///
/// Implemented for every integer type `bitflags` supports except `u128` and
/// `i128`, which have no atomic counterpart.
pub trait AtomicBits: Copy {
    /// The atomic type that holds the bits.
    type Storage;

    fn new(bits: Self) -> Self::Storage;
    fn store(storage: &Self::Storage, bits: Self, ordering: Ordering);
    fn load(storage: &Self::Storage, ordering: Ordering) -> Self;
    fn fetch_and(storage: &Self::Storage, bits: Self, ordering: Ordering) -> Self;
    fn fetch_or(storage: &Self::Storage, bits: Self, ordering: Ordering) -> Self;
    fn fetch_xor(storage: &Self::Storage, bits: Self, ordering: Ordering) -> Self;
    fn fetch_nand(storage: &Self::Storage, bits: Self, ordering: Ordering) -> Self;
    fn swap(storage: &Self::Storage, bits: Self, ordering: Ordering) -> Self;
    fn compare_exchange(
        storage: &Self::Storage,
        current: Self,
        new: Self,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Self, Self>;
    fn compare_exchange_weak(
        storage: &Self::Storage,
        current: Self,
        new: Self,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Self, Self>;
    fn into_inner(storage: Self::Storage) -> Self;
}

macro_rules! impl_atomic_bits {
    ($bits:ty, $atomic:ty) => {
        impl AtomicBits for $bits {
            type Storage = $atomic;

            fn new(bits: Self) -> Self::Storage {
                <$atomic>::new(bits)
            }

            fn store(storage: &Self::Storage, bits: Self, ordering: Ordering) {
                <$atomic>::store(storage, bits, ordering);
            }

            fn load(storage: &Self::Storage, ordering: Ordering) -> Self {
                <$atomic>::load(storage, ordering)
            }

            fn fetch_and(storage: &Self::Storage, bits: Self, ordering: Ordering) -> Self {
                <$atomic>::fetch_and(storage, bits, ordering)
            }

            fn fetch_or(storage: &Self::Storage, bits: Self, ordering: Ordering) -> Self {
                <$atomic>::fetch_or(storage, bits, ordering)
            }

            fn fetch_xor(storage: &Self::Storage, bits: Self, ordering: Ordering) -> Self {
                <$atomic>::fetch_xor(storage, bits, ordering)
            }

            fn fetch_nand(storage: &Self::Storage, bits: Self, ordering: Ordering) -> Self {
                <$atomic>::fetch_nand(storage, bits, ordering)
            }

            fn swap(storage: &Self::Storage, bits: Self, ordering: Ordering) -> Self {
                <$atomic>::swap(storage, bits, ordering)
            }

            fn compare_exchange(
                storage: &Self::Storage,
                current: Self,
                new: Self,
                success: Ordering,
                failure: Ordering,
            ) -> Result<Self, Self> {
                <$atomic>::compare_exchange(storage, current, new, success, failure)
            }

            fn compare_exchange_weak(
                storage: &Self::Storage,
                current: Self,
                new: Self,
                success: Ordering,
                failure: Ordering,
            ) -> Result<Self, Self> {
                <$atomic>::compare_exchange_weak(storage, current, new, success, failure)
            }

            fn into_inner(storage: Self::Storage) -> Self {
                storage.into_inner()
            }
        }
    };
}

// bitflags implements Flags for almost all integer types:
// u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize.
// There is no 128-bit atomic integer in Rust.
impl_atomic_bits!(u8, AtomicU8);
impl_atomic_bits!(u16, AtomicU16);
impl_atomic_bits!(u32, AtomicU32);
impl_atomic_bits!(u64, AtomicU64);
impl_atomic_bits!(usize, AtomicUsize);
impl_atomic_bits!(i8, AtomicI8);
impl_atomic_bits!(i16, AtomicI16);
impl_atomic_bits!(i32, AtomicI32);
impl_atomic_bits!(i64, AtomicI64);
impl_atomic_bits!(isize, AtomicIsize);
