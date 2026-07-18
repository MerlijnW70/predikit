# Changelog

All notable changes to this crate are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/), and the project adheres to
[Semantic Versioning](https://semver.org/).

## 0.3.1

### Added

- `#[repr(transparent)]` on `Refined<T, P>` — the "same layout as `T`" property is now a
  guarantee, not merely a tested observation.

### Documentation

- Corrected the immutability claim: predikit never mutates the value, but a `T` with interior
  mutability (`Cell`, atomics, …) can still change through a shared reference. Added an
  *Interior mutability* section stating exactly when the proof is permanent.
- Clarified that the `const` constructors are panicking `const fn`s — a compile error in a
  `const` context, a panic at runtime — and that they exist for the built-in predicates
  (custom predicates use `try_new`).
- Added tests for the `i64` extremes, an inverted (empty) `InRange`, and a custom predicate;
  softened the "tested exhaustively" wording.

## 0.3.0

### Added

- Full standard-trait suite on `Refined<T, P>`, each forwarding to the inner value:
  `Display`, `PartialOrd`, `Ord`, `Hash`, and `AsRef<T>` — so a refinement drops into
  ordered and hashed collections.
- Predicate combinators `And<A, B>`, `Or<A, B>`, and `Not<P>` for composing predicates.
- Predicates `Negative` and `NonNegative`, with `const` constructors `negative` and
  `non_negative`.
- Optional `serde` feature: `Serialize` (as the inner value) and a `Deserialize` that
  **re-validates the predicate**, so data violating a refinement cannot be deserialized.
- `#![deny(missing_docs)]` and a declared minimum supported Rust version.

## 0.2.0

### Changed

- **Breaking:** the ambiguous per-predicate `Refined::<i64, P>::new` inherent constructors
  are replaced by free `const` functions `positive`, `nonzero`, and `in_range`. Type
  inference now works without turbofish on `Refined`.

## 0.1.0

- Initial release: `Refined<T, P>`, `try_new`, `get`, `into_inner`, `Deref`, and the
  `Positive`, `NonZero`, and `InRange<MIN, MAX>` predicates.
