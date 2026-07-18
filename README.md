# predikit

**Refinement types for Rust: a value carried with a compile-time proof that it satisfies a
predicate — *parse, don't validate*, as a type. `no_std`, zero-dependency.**

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

Constants are checked by the compiler — invalid values simply **do not compile**:

```rust
use predikit::{Refined, InRange};

const PORT: Refined<i64, InRange<1, 65535>> = Refined::<i64, InRange<1, 65535>>::new(8080);
assert_eq!(*PORT.get(), 8080);
```

```rust,compile_fail
use predikit::{Refined, Positive};
// rejected at compile time — this does not build
const BAD: Refined<i64, Positive> = Refined::<i64, Positive>::new(-5);
```

## Why

- **Parse, don't validate.** Push validation to the boundary once; carry the proof in the
  type. No defensive re-checking downstream.
- **Opaque and immutable.** The inner value is private with no `&mut` access, so a refinement
  can never be invalidated after it is made.
- **Zero-cost.** `Refined<T, P>` has exactly the size of `T` — even nested:
  `Refined<Refined<T, P1>, P2>` is still just a `T` in memory. `into_inner()` and `Deref`
  are free.
- **Compile-time or runtime.** `const` constructors reject bad constants at compile time;
  `try_new` refines at runtime and hands the value back on failure.
- **`no_std`, zero dependencies, `#![forbid(unsafe_code)]`.**

## Built-in predicates

`Positive` (`> 0`), `NonZero` (`!= 0`), and `InRange<MIN, MAX>` (inclusive). Write your own
by implementing the `Predicate<T>` trait:

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

## Tested exhaustively

Every predicate boundary, the runtime and compile-time constructors (including the
compile-time rejection of invalid constants), the zero-overhead layout — nested included —
and the `Deref`/`Clone`/`Debug`/`Eq` behavior are all exercised by the test suite. A green
build means the behavior is pinned, not merely line-covered.

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at your option.
