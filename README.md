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
| `load(ordering)` | atomic load | current flags |
| `store(flags, ordering)` | atomic store | nothing |
| `swap(flags, ordering)` | atomic swap | previous flags |
| `set(flags, ordering)` | atomic OR | previous flags |
| `unset(flags, ordering)` | atomic AND NOT | previous flags |
| `clear(ordering)` | store empty | nothing |
| `take(flags, ordering)` | atomic AND NOT | requested flags that were set |
| `peek(flags, ordering)` | atomic load | requested flags that are set |
| `fetch_and(flags, ordering)` | atomic AND | previous flags |
| `fetch_or(flags, ordering)` | atomic OR | previous flags |
| `fetch_xor(flags, ordering)` | atomic XOR | previous flags |
| `fetch_nand(flags, ordering)` | atomic NAND | previous flags |
| `into_inner()` | consume | final flags |

`set` and `unset` are convenience wrappers around `fetch_or` and `fetch_and`.
All `fetch_*` methods return the full previous value.

`take` and `peek` return only the requested flags. `take` claims them with a
single atomic operation; `peek` is a plain load, so another thread may change
the value immediately after.

`fetch_nand` keeps the standard atomic semantics: the new value is
`!(current & flags)`, so bits outside `flags` are set as well.

## Limits

- `u128` and `i128` are not supported because Rust has no 128-bit atomic
  integer.
- 64-bit flag sets need a target with 64-bit atomics, like any `AtomicU64`.
- Unknown bits are preserved on load (`from_bits_retain`), not truncated.
- No ordering is chosen for you; pick one with the same rules as the standard
  library atomics.

## License

MIT OR Apache-2.0.
