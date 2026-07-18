# Changelog

All notable changes to this crate are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/), and the project adheres to
[Semantic Versioning](https://semver.org/).

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
