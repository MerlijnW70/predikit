#![no_std]
#![forbid(unsafe_code)]
//! Refinement types for Rust: a value carried alongside a compile-time proof that it
//! satisfies a predicate.
//!
//! A [`Refined<T, P>`] is a `T` that is *known* to satisfy predicate `P` — because the only
//! ways to build one are [`Refined::try_new`] (checked once, at runtime) or a predicate's
//! `const` constructor (checked by the compiler). After construction the value is immutable,
//! so the proof can never be invalidated. [`Refined::into_inner`] and `Deref` hand the value
//! back at zero cost.
//!
//! This is *parse, don't validate* as a type: once you hold a `Refined<T, P>`, every
//! downstream function can rely on `P` without re-checking.
//!
//! # Runtime refinement
//!
//! ```
//! use predikit::{Refined, Positive};
//!
//! let ok = Refined::<i64, Positive>::try_new(42).unwrap();
//! assert_eq!(*ok, 42); // Deref to the inner value
//!
//! // a value that fails the predicate is handed back untouched
//! assert_eq!(Refined::<i64, Positive>::try_new(-1), Err(-1));
//! ```
//!
//! # Compile-time refinement
//!
//! The constructor functions [`positive`], [`nonzero`], and [`in_range`] are `const` and
//! infer their type — no turbofish on `Refined` needed:
//!
//! ```
//! use predikit::{in_range, positive, Refined, InRange, Positive};
//!
//! const PORT: Refined<i64, InRange<1, 65535>> = in_range::<1, 65535>(8080);
//! const RETRIES: Refined<i64, Positive> = positive(3);
//! assert_eq!(*PORT.get(), 8080);
//! assert_eq!(*RETRIES.get(), 3);
//! ```
//!
//! An invalid constant does not compile:
//!
//! ```compile_fail
//! use predikit::positive;
//! const BAD: predikit::Refined<i64, predikit::Positive> = positive(-5); // rejected at compile time
//! ```

use core::marker::PhantomData;
use core::ops::Deref;

/// A predicate that a value of type `T` either satisfies or does not.
///
/// Implementors are usually zero-sized marker types (`Positive`, `NonZero`, …). A
/// predicate's `test` and any `const` constructor for it must agree on the same condition.
pub trait Predicate<T> {
    /// Whether `value` satisfies this predicate.
    fn test(value: &T) -> bool;
}

/// A `T` proven to satisfy predicate `P`.
///
/// The wrapped value is private and never mutated, so a `Refined<T, P>` you hold is a
/// standing guarantee that `P` holds for it. Build one with [`Refined::try_new`] or one of
/// the `const` constructor functions ([`positive`], [`nonzero`], [`in_range`]).
pub struct Refined<T, P> {
    value: T,
    _p: PhantomData<P>,
}

impl<T, P: Predicate<T>> Refined<T, P> {
    /// Refine `value` if it satisfies `P`, otherwise hand it back unchanged in `Err`.
    pub fn try_new(value: T) -> Result<Self, T> {
        if P::test(&value) {
            Ok(Refined {
                value,
                _p: PhantomData,
            })
        } else {
            Err(value)
        }
    }
}

impl<T, P> Refined<T, P> {
    /// A shared reference to the wrapped value.
    pub const fn get(&self) -> &T {
        &self.value
    }

    /// Consume the refinement, returning the wrapped value. Compiles to a move — zero cost.
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T, P> Deref for Refined<T, P> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}

// The marker `P` carries no data, so every trait below bounds only `T` — a `Refined` is
// `Clone`/`Copy`/`Debug`/`Eq` exactly when its inner value is, regardless of `P`.

impl<T: Clone, P> Clone for Refined<T, P> {
    fn clone(&self) -> Self {
        Refined {
            value: self.value.clone(),
            _p: PhantomData,
        }
    }
}

impl<T: Copy, P> Copy for Refined<T, P> {}

impl<T: core::fmt::Debug, P> core::fmt::Debug for Refined<T, P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Refined").field(&self.value).finish()
    }
}

impl<T: PartialEq, P> PartialEq for Refined<T, P> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T: Eq, P> Eq for Refined<T, P> {}

/// The predicate `value > 0`.
pub struct Positive;

impl Predicate<i64> for Positive {
    fn test(value: &i64) -> bool {
        *value > 0
    }
}

/// The predicate `value != 0`.
pub struct NonZero;

impl Predicate<i64> for NonZero {
    fn test(value: &i64) -> bool {
        *value != 0
    }
}

/// The predicate `MIN <= value <= MAX`, inclusive on both ends.
pub struct InRange<const MIN: i64, const MAX: i64>;

impl<const MIN: i64, const MAX: i64> Predicate<i64> for InRange<MIN, MAX> {
    fn test(value: &i64) -> bool {
        *value >= MIN && *value <= MAX
    }
}

/// Refine a constant as [`Positive`] at compile time, rejecting any value that is not
/// greater than zero.
pub const fn positive(value: i64) -> Refined<i64, Positive> {
    assert!(
        value > 0,
        "predikit::positive: the value must be greater than zero"
    );
    Refined {
        value,
        _p: PhantomData,
    }
}

/// Refine a constant as [`NonZero`] at compile time, rejecting a value of zero.
pub const fn nonzero(value: i64) -> Refined<i64, NonZero> {
    assert!(value != 0, "predikit::nonzero: the value must not be zero");
    Refined {
        value,
        _p: PhantomData,
    }
}

/// Refine a constant as [`InRange`] at compile time, rejecting any value outside `MIN..=MAX`.
pub const fn in_range<const MIN: i64, const MAX: i64>(
    value: i64,
) -> Refined<i64, InRange<MIN, MAX>> {
    assert!(
        value >= MIN && value <= MAX,
        "predikit::in_range: the value is out of range"
    );
    Refined {
        value,
        _p: PhantomData,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::size_of;

    #[test]
    fn positive_accepts_above_zero_and_rejects_the_rest() {
        assert!(Positive::test(&1));
        assert!(!Positive::test(&0), "zero is not positive");
        assert!(!Positive::test(&-1));
    }

    #[test]
    fn nonzero_rejects_only_zero() {
        assert!(!NonZero::test(&0));
        assert!(NonZero::test(&1));
        assert!(NonZero::test(&-1));
    }

    #[test]
    fn inrange_is_inclusive_on_both_ends() {
        assert!(<InRange<1, 10>>::test(&1), "MIN is included");
        assert!(<InRange<1, 10>>::test(&10), "MAX is included");
        assert!(!<InRange<1, 10>>::test(&0), "below MIN is excluded");
        assert!(!<InRange<1, 10>>::test(&11), "above MAX is excluded");
    }

    #[test]
    fn try_new_accepts_valid_and_returns_the_value_back_on_failure() {
        let ok = Refined::<i64, Positive>::try_new(5).expect("5 is positive");
        assert_eq!(*ok.get(), 5);
        assert_eq!(Refined::<i64, Positive>::try_new(0), Err(0));
        assert_eq!(Refined::<i64, Positive>::try_new(-3), Err(-3));
    }

    #[test]
    fn get_deref_and_into_inner_all_yield_the_value() {
        let r = Refined::<i64, NonZero>::try_new(7).unwrap();
        assert_eq!(*r.get(), 7);
        assert_eq!(*r, 7); // Deref
        assert_eq!(r.into_inner(), 7);
    }

    #[test]
    fn const_constructors_build_valid_values_and_infer_their_type() {
        const P: Refined<i64, Positive> = positive(3);
        const Z: Refined<i64, NonZero> = nonzero(-9);
        const R: Refined<i64, InRange<1, 100>> = in_range::<1, 100>(100);
        assert_eq!(*P.get(), 3);
        assert_eq!(*Z.get(), -9);
        assert_eq!(*R.get(), 100);
    }

    #[test]
    #[should_panic]
    fn positive_rejects_zero() {
        let _ = positive(0);
    }

    #[test]
    #[should_panic]
    fn nonzero_rejects_zero() {
        let _ = nonzero(0);
    }

    #[test]
    #[should_panic]
    fn in_range_rejects_below_min() {
        let _ = in_range::<1, 10>(0);
    }

    #[test]
    #[should_panic]
    fn in_range_rejects_above_max() {
        let _ = in_range::<1, 10>(11);
    }

    #[test]
    fn in_range_accepts_both_boundaries() {
        assert_eq!(*in_range::<1, 10>(1).get(), 1);
        assert_eq!(*in_range::<1, 10>(10).get(), 10);
    }

    #[test]
    fn refinement_is_zero_cost_even_when_nested() {
        // a Refined is exactly its inner value in memory; nesting adds nothing
        assert_eq!(size_of::<Refined<i64, Positive>>(), size_of::<i64>());
        assert_eq!(
            size_of::<Refined<Refined<i64, Positive>, NonZero>>(),
            size_of::<i64>(),
            "nested refinement collapses to the inner value's size"
        );
    }

    #[test]
    #[allow(clippy::clone_on_copy)] // deliberately exercising the manual Clone impl
    fn clone_copy_debug_and_eq_follow_the_inner_value() {
        let a = Refined::<i64, Positive>::try_new(4).unwrap();
        let b = a; // Copy
        let c = a.clone();
        assert_eq!(a, b);
        assert_eq!(a, c);
        assert_ne!(a, Refined::<i64, Positive>::try_new(5).unwrap());
        // Debug shows the inner value under the Refined name
        use core::fmt::Write;
        let mut sink = Sink::new();
        let _ = write!(sink, "{:?}", a);
        assert_eq!(sink.as_str(), "Refined(4)");
    }

    struct Sink {
        bytes: [u8; 64],
        len: usize,
    }
    impl Sink {
        fn new() -> Self {
            Sink {
                bytes: [0u8; 64],
                len: 0,
            }
        }
        fn as_str(&self) -> &str {
            core::str::from_utf8(&self.bytes[..self.len]).unwrap_or("")
        }
    }
    impl core::fmt::Write for Sink {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            for &byte in s.as_bytes() {
                if self.len < self.bytes.len() {
                    self.bytes[self.len] = byte;
                    self.len += 1;
                }
            }
            Ok(())
        }
    }
}
