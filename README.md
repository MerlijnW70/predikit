# predikit

**Refinement types for Rust: a value carried with a compile-time proof that it satisfies a
predicate — *parse, don't validate*, as a type. `no_std`, zero-dependency, with predicate
combinators and optional serde validation.**

A `Refined<T, P>` is a `T` that is *known* to satisfy predicate `P`. The only ways to build
one are `try_new` (checked once, at runtime) or a predicate's `const` constructor (checked by
the compiler). The wrapped value is private and never mutated, so once you hold a
`Refined<T, P>`, every downstream function can rely on `P` without re-checking.

```rust
use predikit::{Refined, Positive};

let ok = Refined::<i64, Positive>::try_new(42).unwrap();
assert_eq!(*ok, 42); // Deref to the inner value

// a value that fails the predicate is handed back untouched
assert_eq!(Refined::<i64, Positive>::try_new(-1), Err(-1));
```

## Compile-time refinement

The `const` constructors `positive`, `negative`, `non_negative`, `nonzero`, and `in_range`
infer their type — no turbofish on `Refined` needed. Invalid constants simply **do not
compile**:

```rust
use predikit::{in_range, positive, Refined, InRange, Positive};

const PORT: Refined<i64, InRange<1, 65535>> = in_range::<1, 65535>(8080);
const RETRIES: Refined<i64, Positive> = positive(3);
assert_eq!(*PORT.get(), 8080);
```

```rust,compile_fail
use predikit::positive;
// rejected at compile time — this does not build
const BAD: predikit::Refined<i64, predikit::Positive> = positive(-5);
```

## Composing predicates

Combine predicates with `And`, `Or`, and `Not`:

```rust
use predikit::{Refined, And, Positive, InRange};

// positive AND within 1..=100
type Percent = And<Positive, InRange<1, 100>>;
assert!(Refined::<i64, Percent>::try_new(50).is_ok());
assert!(Refined::<i64, Percent>::try_new(0).is_err());   // fails Positive
assert!(Refined::<i64, Percent>::try_new(101).is_err()); // fails InRange
```

## Serde (optional)

Enable the `serde` feature and `Refined<T, P>` serializes as its inner value and
**re-validates the predicate on deserialization** — data that violates the predicate cannot
be deserialized into a refinement type:

```toml
predikit = { version = "0.3", features = ["serde"] }
```

```rust,ignore
let r: Refined<i64, Positive> = serde_json::from_str("42")?;   // Ok
let bad = serde_json::from_str::<Refined<i64, Positive>>("-1"); // Err — refused at the boundary
```

## Why

- **Parse, don't validate.** Push validation to the boundary once; carry the proof in the
  type. No defensive re-checking downstream.
- **Opaque and immutable.** The inner value is private with no `&mut` access, so a refinement
  can never be invalidated after it is made.
- **Zero-cost.** `Refined<T, P>` has exactly the size of `T` — even nested:
  `Refined<Refined<T, P1>, P2>` is still just a `T` in memory. `into_inner()`, `Deref`,
  and `AsRef` are free.
- **Fully interoperable.** Forwards `Clone`, `Copy`, `Debug`, `Display`, `PartialEq`, `Eq`,
  `PartialOrd`, `Ord`, and `Hash` to the inner value, so a refinement drops into ordered
  and hashed collections.
- **`no_std`, zero default dependencies, `#![forbid(unsafe_code)]`, `#![deny(missing_docs)]`.**

## Built-in predicates

`Positive` (`> 0`), `Negative` (`< 0`), `NonNegative` (`>= 0`), `NonZero` (`!= 0`), and
`InRange<MIN, MAX>` (inclusive) — plus the combinators `And<A, B>`, `Or<A, B>`, and `Not<P>`.
Write your own by implementing the `Predicate<T>` trait:

```rust
use predikit::{Predicate, Refined};

struct Even;
impl Predicate<i64> for Even {
    fn test(value: &i64) -> bool {
        value % 2 == 0
    }
}

let e = Refined::<i64, Even>::try_new(8).unwrap();
assert_eq!(*e, 8);
```

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at your option.
