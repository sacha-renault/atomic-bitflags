use super::*;
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    struct TestFlags: u8 {
        const A = 0b0001;
        const B = 0b0010;
        const C = 0b0100;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct WideFlags: u64 {
        const LOW = 1 << 0;
        const HIGH = 1 << 63;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct SignedFlags: i32 {
        const A = 1 << 0;
        const B = 1 << 30;
    }
}

#[test]
fn store_and_load() {
    let flags = AtomicFlags::new(TestFlags::A);
    assert_eq!(flags.load(Ordering::SeqCst), TestFlags::A);

    flags.store(TestFlags::B, Ordering::SeqCst);
    assert_eq!(flags.load(Ordering::SeqCst), TestFlags::B);
}

#[test]
fn take_removes_requested_flags_and_returns_taken() {
    let flags = AtomicFlags::new(TestFlags::A | TestFlags::B);

    let taken = flags.take(TestFlags::A, Ordering::SeqCst);
    assert_eq!(taken, TestFlags::A);
    assert_eq!(flags.load(Ordering::SeqCst), TestFlags::B);

    // Taking a flag that isn't set is a no-op.
    let taken = flags.take(TestFlags::A, Ordering::SeqCst);
    assert_eq!(taken, TestFlags::empty());
    assert_eq!(flags.load(Ordering::SeqCst), TestFlags::B);

    let taken = flags.take(TestFlags::A | TestFlags::B, Ordering::SeqCst);
    assert_eq!(taken, TestFlags::B);
    assert_eq!(flags.load(Ordering::SeqCst), TestFlags::empty());
}

#[test]
fn peek_returns_requested_flags_without_clearing() {
    let flags = AtomicFlags::new(TestFlags::A | TestFlags::B);

    assert_eq!(flags.peek(TestFlags::A, Ordering::SeqCst), TestFlags::A);
    assert_eq!(
        flags.peek(TestFlags::A | TestFlags::B | TestFlags::C, Ordering::SeqCst),
        TestFlags::A | TestFlags::B
    );
    assert_eq!(
        flags.peek(TestFlags::C, Ordering::SeqCst),
        TestFlags::empty()
    );

    // Peeking doesn't clear anything.
    assert_eq!(flags.load(Ordering::SeqCst), TestFlags::A | TestFlags::B);
}

#[test]
fn fetch_operations() {
    let flags = AtomicFlags::new(TestFlags::A);

    let previous = flags.fetch_or(TestFlags::B, Ordering::SeqCst);
    assert_eq!(previous, TestFlags::A);
    assert_eq!(flags.load(Ordering::SeqCst), TestFlags::A | TestFlags::B);

    let previous = flags.fetch_and(!TestFlags::A, Ordering::SeqCst);
    assert_eq!(previous, TestFlags::A | TestFlags::B);
    assert_eq!(flags.load(Ordering::SeqCst), TestFlags::B);
}

#[test]
fn fetch_xor_and_swap_return_previous_state() {
    let flags = AtomicFlags::new(TestFlags::A | TestFlags::B);

    let previous = flags.toggle(TestFlags::B | TestFlags::C, Ordering::SeqCst);
    assert_eq!(previous, TestFlags::A | TestFlags::B);
    assert_eq!(flags.load(Ordering::SeqCst), TestFlags::A | TestFlags::C);

    let previous = flags.swap(TestFlags::B, Ordering::SeqCst);
    assert_eq!(previous, TestFlags::A | TestFlags::C);
    assert_eq!(flags.load(Ordering::SeqCst), TestFlags::B);
}

#[test]
fn fetch_nand_operates_on_the_raw_word() {
    let flags = AtomicFlags::new(TestFlags::A | TestFlags::B);

    let previous = flags.fetch_nand(TestFlags::A, Ordering::SeqCst);
    assert_eq!(previous, TestFlags::A | TestFlags::B);

    // NAND affects bits outside the mask too: !(0b011 & 0b001) = 0xFE.
    assert_eq!(flags.load(Ordering::SeqCst).bits(), !TestFlags::A.bits());
}

#[test]
fn set_returns_previous_state_and_ors_requested_flags() {
    let flags = AtomicFlags::new(TestFlags::A);

    let previous = flags.set(TestFlags::B, Ordering::SeqCst);
    assert_eq!(previous, TestFlags::A);
    assert_eq!(flags.load(Ordering::SeqCst), TestFlags::A | TestFlags::B);

    // Setting an already-set flag leaves the state unchanged.
    let previous = flags.set(TestFlags::A, Ordering::SeqCst);
    assert_eq!(previous, TestFlags::A | TestFlags::B);
    assert_eq!(flags.load(Ordering::SeqCst), TestFlags::A | TestFlags::B);
}

#[test]
fn unset_returns_previous_state_and_clears_only_requested_flags() {
    let flags = AtomicFlags::new(TestFlags::A | TestFlags::B);

    let previous = flags.unset(TestFlags::A, Ordering::SeqCst);
    assert_eq!(previous, TestFlags::A | TestFlags::B);
    assert_eq!(flags.load(Ordering::SeqCst), TestFlags::B);

    // Clearing a flag that isn't set is a no-op.
    let previous = flags.unset(TestFlags::C, Ordering::SeqCst);
    assert_eq!(previous, TestFlags::B);
    assert_eq!(flags.load(Ordering::SeqCst), TestFlags::B);
}

#[test]
fn clear_and_into_inner() {
    let flags = AtomicFlags::new(TestFlags::A | TestFlags::B);
    flags.clear(Ordering::SeqCst);
    assert_eq!(flags.load(Ordering::SeqCst), TestFlags::empty());

    let flags = AtomicFlags::new(TestFlags::A | TestFlags::C);
    assert_eq!(flags.into_inner(), TestFlags::A | TestFlags::C);
}

#[test]
fn default_uses_the_flags_default() {
    let flags = AtomicFlags::<TestFlags>::default();
    assert_eq!(flags.load(Ordering::SeqCst), TestFlags::default());
}

#[test]
fn works_with_u64_and_signed_bits() {
    let wide = AtomicFlags::new(WideFlags::HIGH);
    assert_eq!(wide.load(Ordering::SeqCst), WideFlags::HIGH);

    wide.set(WideFlags::LOW, Ordering::SeqCst);
    assert_eq!(
        wide.load(Ordering::SeqCst),
        WideFlags::LOW | WideFlags::HIGH
    );

    let signed = AtomicFlags::new(SignedFlags::B);
    assert_eq!(
        signed.take(SignedFlags::B, Ordering::SeqCst),
        SignedFlags::B
    );
    assert_eq!(signed.load(Ordering::SeqCst), SignedFlags::empty());
}

#[test]
fn take_claims_a_flag_only_once() {
    use std::sync::{Arc, Barrier};

    let flags = Arc::new(AtomicFlags::new(TestFlags::A));
    let barrier = Arc::new(Barrier::new(8));

    let handles: Vec<_> = (0..8)
        .map(|_| {
            let flags = Arc::clone(&flags);
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                flags.take(TestFlags::A, Ordering::SeqCst)
            })
        })
        .collect();

    let claimed = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .filter(|flags| flags.contains(TestFlags::A))
        .count();

    assert_eq!(claimed, 1);
    assert_eq!(flags.load(Ordering::SeqCst), TestFlags::empty());
}

#[test]
fn debug_shows_the_current_flags() {
    let flags = AtomicFlags::new(TestFlags::A | TestFlags::C);

    assert_eq!(
        format!("{:?}", flags),
        format!("{:?}", TestFlags::A | TestFlags::C)
    );
}

#[test]
fn compare_exchange_swaps_on_match() {
    let flags = AtomicFlags::new(TestFlags::A);

    assert_eq!(
        flags.compare_exchange(
            TestFlags::A,
            TestFlags::B,
            Ordering::SeqCst,
            Ordering::SeqCst
        ),
        Ok(TestFlags::A)
    );
    assert_eq!(flags.load(Ordering::SeqCst), TestFlags::B);

    assert_eq!(
        flags.compare_exchange(
            TestFlags::A,
            TestFlags::C,
            Ordering::SeqCst,
            Ordering::SeqCst
        ),
        Err(TestFlags::B)
    );
    assert_eq!(flags.load(Ordering::SeqCst), TestFlags::B);
}

#[test]
fn try_update_applies_or_aborts() {
    let flags = AtomicFlags::new(TestFlags::A);

    let previous = flags.try_update(Ordering::SeqCst, Ordering::SeqCst, |current| {
        Some(current | TestFlags::B)
    });
    assert_eq!(previous, Ok(TestFlags::A));
    assert_eq!(flags.load(Ordering::SeqCst), TestFlags::A | TestFlags::B);

    let previous = flags.try_update(Ordering::SeqCst, Ordering::SeqCst, |_| None);
    assert_eq!(previous, Err(TestFlags::A | TestFlags::B));
    assert_eq!(flags.load(Ordering::SeqCst), TestFlags::A | TestFlags::B);
}

#[test]
fn unset_preserves_unknown_bits() {
    let unknown = TestFlags::from_bits_retain(0b1000_0000);
    let flags = AtomicFlags::new(TestFlags::A | unknown);

    let previous = flags.unset(TestFlags::A, Ordering::SeqCst);
    assert_eq!(previous, TestFlags::A | unknown);
    assert_eq!(flags.load(Ordering::SeqCst), unknown);
}

#[test]
fn unset_clears_requested_unknown_bits() {
    let unknown = TestFlags::from_bits_retain(0b1000_0000);
    let flags = AtomicFlags::new(TestFlags::A | TestFlags::B | unknown);

    let previous = flags.unset(unknown, Ordering::SeqCst);
    assert_eq!(previous, TestFlags::A | TestFlags::B | unknown);
    assert_eq!(flags.load(Ordering::SeqCst), TestFlags::A | TestFlags::B);
}

#[test]
fn take_preserves_unknown_bits_that_were_not_requested() {
    let unknown = TestFlags::from_bits_retain(0b1000_0000);
    let flags = AtomicFlags::new(TestFlags::A | TestFlags::B | unknown);

    let taken = flags.take(TestFlags::A, Ordering::SeqCst);
    assert_eq!(taken, TestFlags::A);
    assert_eq!(flags.load(Ordering::SeqCst), TestFlags::B | unknown);
}

#[test]
fn take_returns_and_clears_requested_unknown_bits() {
    let unknown = TestFlags::from_bits_retain(0b1000_0000);
    let flags = AtomicFlags::new(TestFlags::A | unknown);

    let taken = flags.take(unknown, Ordering::SeqCst);
    assert_eq!(taken.bits(), unknown.bits());
    assert_eq!(flags.load(Ordering::SeqCst), TestFlags::A);
}
