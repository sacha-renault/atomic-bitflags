# atomic-bitflags

Atomic storage for [bitflags](https://crates.io/crates/bitflags) flag sets.

`AtomicFlags<F>` stores the bits of a `bitflags` type in a standard library
atomic integer. Reads and writes are lock-free, and every operation takes an
`Ordering` exactly like `AtomicU32` or `AtomicU8`.

## Install

```toml
[dependencies]
atomic-bitflags = "0.1"
bitflags = "2"
```

## Use case

Runtime configuration that several threads read and a few threads update:
feature switches, log levels, or service modes. It is also a replacement for a
mutex around a flag set when you only need atomic reads, writes, and
set/clear/toggle operations.

```rust
use atomic_bitflags::AtomicFlags;
use bitflags::bitflags;
use std::sync::Arc;
use std::sync::atomic::Ordering;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Features: u8 {
        const CACHE = 1 << 0;
        const TRACE = 1 << 1;
        const STRICT = 1 << 2;
    }
}

fn main() {
    let features = Arc::new(AtomicFlags::new(Features::CACHE));

    // Enable tracing from another thread.
    let remote = Arc::clone(&features);
    std::thread::spawn(move || {
        remote.set(Features::TRACE, Ordering::Release);
    })
    .join()
    .unwrap();

    let current = features.load(Ordering::Acquire);
    assert!(current.contains(Features::CACHE | Features::TRACE));

    // `take` claims flags atomically: only one caller can claim STRICT.
    features.set(Features::STRICT, Ordering::SeqCst);
    assert_eq!(features.take(Features::STRICT, Ordering::SeqCst), Features::STRICT);
    assert!(features.take(Features::STRICT, Ordering::SeqCst).is_empty());
}
```

Operations on a shared instance take `&self`, so wrap the value in `Arc` to
share it. `AtomicFlags` is `Send + Sync`.

## API

| Method | Operation | Returns |
| --- | --- | --- |
| `new(flags)` | initialize | the new `AtomicFlags` |
| `from_storage(atomic)` | const initialize from a pre-built atomic | the new `AtomicFlags` |
| `load(ordering)` | atomic load | current flags |
| `store(flags, ordering)` | atomic store | nothing |
| `swap(flags, ordering)` | atomic swap | previous flags |
| `compare_exchange(current, new, success, failure)` | compare and swap | `Ok(previous)` or `Err(actual)` |
| `compare_exchange_weak(current, new, success, failure)` | same, may fail spuriously | `Ok(previous)` or `Err(actual)` |
| `set(flags, ordering)` | atomic OR | previous flags |
| `unset(flags, ordering)` | atomic AND NOT | previous flags |
| `clear(ordering)` | store empty | nothing |
| `take(flags, ordering)` | atomic AND NOT | requested flags that were set |
| `peek(flags, ordering)` | atomic load | requested flags that are set |
| `fetch_and(flags, ordering)` | atomic AND | previous flags |
| `fetch_or(flags, ordering)` | atomic OR | previous flags |
| `fetch_xor(flags, ordering)` | atomic XOR | previous flags |
| `toggle(flags, ordering)` | alias for `fetch_xor` | previous flags |
| `fetch_nand(flags, ordering)` | atomic NAND | previous flags |
| `try_update(set_order, fetch_order, f)` | apply `f` until it succeeds | `Ok(previous)` or `Err(current)` |
| `into_inner()` | consume | final flags |

`set` and `unset` are convenience wrappers around `fetch_or` and `fetch_and`.
The bitwise `fetch_*` methods return the full previous value; `try_update`
returns `Ok(previous)` on success, or `Err(current)` if the closure returns
`None`.

`compare_exchange` and `compare_exchange_weak` follow the standard atomics: on
failure they return the actual current value, and the failure ordering may not
be stronger than the success ordering.

`take` and `peek` return only the requested flags. `take` claims them with a
single atomic operation; `peek` is a plain load, so another thread may change
the value immediately after.

`fetch_nand` keeps the standard atomic semantics: the new value is
`!(current & flags)`, so bits outside `flags` are set as well.

### Static initialization

`from_storage` is const, so a flag set can live in a `static`:

```rust
use std::sync::atomic::AtomicU8;

static FEATURES: AtomicFlags<Features> =
    AtomicFlags::from_storage(AtomicU8::new(Features::CACHE.bits()));
```

The atomic type must match the bits type of the flag set (`u8` uses `AtomicU8`,
`u64` uses `AtomicU64`, and so on).

## Limits

- `u128` and `i128` are not supported because Rust has no 128-bit atomic
  integer.
- 64-bit flag sets need a target with 64-bit atomics, like any `AtomicU64`.
- Unknown bits are preserved on load (`from_bits_retain`), not truncated.
- No ordering is chosen for you; pick one with the same rules as the standard
  library atomics.

## License

MIT OR Apache-2.0.
