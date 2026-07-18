#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
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
//! The `const` constructors ([`positive`], [`nonzero`], [`in_range`], …) infer their type —
//! no turbofish on `Refined` needed. Invalid constants do not compile:
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
//! ```compile_fail
//! use predikit::positive;
//! const BAD: predikit::Refined<i64, predikit::Positive> = positive(-5); // rejected at compile time
//! ```
//!
//! # Composing predicates
//!
//! Combine predicates with [`And`], [`Or`], and [`Not`]:
//!
//! ```
//! use predikit::{Refined, And, Positive, InRange};
//!
//! // positive AND within 1..=100
//! type Percent = And<Positive, InRange<1, 100>>;
//! assert!(Refined::<i64, Percent>::try_new(50).is_ok());
//! assert!(Refined::<i64, Percent>::try_new(0).is_err());   // fails Positive
//! assert!(Refined::<i64, Percent>::try_new(101).is_err()); // fails InRange
//! ```
//!
//! # Serde
//!
//! With the `serde` feature, `Refined<T, P>` serializes as its inner value and
//! **re-validates the predicate on deserialization** — so a refinement type cannot be
//! constructed from data that violates it. See the crate's `serde` feature.

use core::cmp::Ordering;
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use core::ops::Deref;

/// A predicate that a value of type `T` either satisfies or does not.
///
/// Implementors are usually zero-sized marker types ([`Positive`], [`NonZero`], …). A
/// predicate's `test` and any `const` constructor for it must agree on the same condition.
pub trait Predicate<T> {
    /// Whether `value` satisfies this predicate.
    fn test(value: &T) -> bool;
}

/// A `T` proven to satisfy predicate `P`.
///
/// The wrapped value is private and never mutated, so a `Refined<T, P>` you hold is a
/// standing guarantee that `P` holds for it. Build one with [`Refined::try_new`] or a
/// `const` constructor function ([`positive`], [`nonzero`], [`in_range`], …).
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

impl<T, P> AsRef<T> for Refined<T, P> {
    fn as_ref(&self) -> &T {
        &self.value
    }
}

// The marker `P` carries no data, so every trait below bounds only `T` — a `Refined`
// forwards the trait exactly when its inner value implements it, regardless of `P`.

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

impl<T: core::fmt::Display, P> core::fmt::Display for Refined<T, P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.value.fmt(f)
    }
}

impl<T: PartialEq, P> PartialEq for Refined<T, P> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T: Eq, P> Eq for Refined<T, P> {}

impl<T: PartialOrd, P> PartialOrd for Refined<T, P> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl<T: Ord, P> Ord for Refined<T, P> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}

impl<T: Hash, P> Hash for Refined<T, P> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

#[cfg(feature = "serde")]
impl<T: serde::Serialize, P> serde::Serialize for Refined<T, P> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.value.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T, P> serde::Deserialize<'de> for Refined<T, P>
where
    T: serde::Deserialize<'de>,
    P: Predicate<T>,
{
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = T::deserialize(deserializer)?;
        Refined::try_new(value).map_err(|_| {
            <D::Error as serde::de::Error>::custom("value did not satisfy the refinement predicate")
        })
    }
}

/// The predicate `value > 0`.
pub struct Positive;

impl Predicate<i64> for Positive {
    fn test(value: &i64) -> bool {
        *value > 0
    }
}

/// The predicate `value < 0`.
pub struct Negative;

impl Predicate<i64> for Negative {
    fn test(value: &i64) -> bool {
        *value < 0
    }
}

/// The predicate `value >= 0`.
pub struct NonNegative;

impl Predicate<i64> for NonNegative {
    fn test(value: &i64) -> bool {
        *value >= 0
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

/// The predicate that holds exactly when `P` does **not** — the negation of `P`.
pub struct Not<P>(PhantomData<P>);

impl<T, P: Predicate<T>> Predicate<T> for Not<P> {
    fn test(value: &T) -> bool {
        !P::test(value)
    }
}

/// The predicate that holds when **both** `A` and `B` do.
pub struct And<A, B>(PhantomData<(A, B)>);

impl<T, A: Predicate<T>, B: Predicate<T>> Predicate<T> for And<A, B> {
    fn test(value: &T) -> bool {
        A::test(value) && B::test(value)
    }
}

/// The predicate that holds when **either** `A` or `B` does.
pub struct Or<A, B>(PhantomData<(A, B)>);

impl<T, A: Predicate<T>, B: Predicate<T>> Predicate<T> for Or<A, B> {
    fn test(value: &T) -> bool {
        A::test(value) || B::test(value)
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

/// Refine a constant as [`Negative`] at compile time, rejecting any value that is not less
/// than zero.
pub const fn negative(value: i64) -> Refined<i64, Negative> {
    assert!(
        value < 0,
        "predikit::negative: the value must be less than zero"
    );
    Refined {
        value,
        _p: PhantomData,
    }
}

/// Refine a constant as [`NonNegative`] at compile time, rejecting any negative value.
pub const fn non_negative(value: i64) -> Refined<i64, NonNegative> {
    assert!(
        value >= 0,
        "predikit::non_negative: the value must not be negative"
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
    fn negative_accepts_below_zero_and_rejects_the_rest() {
        assert!(Negative::test(&-1));
        assert!(!Negative::test(&0), "zero is not negative");
        assert!(!Negative::test(&1));
    }

    #[test]
    fn non_negative_accepts_zero_and_up() {
        assert!(NonNegative::test(&0));
        assert!(NonNegative::test(&1));
        assert!(!NonNegative::test(&-1));
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
    fn not_negates_the_inner_predicate() {
        assert!(<Not<Positive>>::test(&0), "0 is not positive, so Not<Positive> holds");
        assert!(<Not<Positive>>::test(&-1));
        assert!(!<Not<Positive>>::test(&1));
    }

    #[test]
    fn and_requires_both_predicates() {
        type P = And<Positive, InRange<1, 10>>;
        assert!(<P>::test(&5));
        assert!(!<P>::test(&0), "fails Positive");
        assert!(!<P>::test(&11), "fails InRange");
    }

    #[test]
    fn or_requires_either_predicate() {
        type P = Or<Negative, InRange<100, 200>>;
        assert!(<P>::test(&-5), "satisfies Negative");
        assert!(<P>::test(&150), "satisfies InRange");
        assert!(!<P>::test(&5), "satisfies neither");
    }

    #[test]
    fn try_new_accepts_valid_and_returns_the_value_back_on_failure() {
        let ok = Refined::<i64, Positive>::try_new(5).expect("5 is positive");
        assert_eq!(*ok.get(), 5);
        assert_eq!(Refined::<i64, Positive>::try_new(0), Err(0));
        assert_eq!(Refined::<i64, Positive>::try_new(-3), Err(-3));
    }

    #[test]
    fn get_deref_asref_and_into_inner_all_yield_the_value() {
        let r = Refined::<i64, NonZero>::try_new(7).unwrap();
        assert_eq!(*r.get(), 7);
        assert_eq!(*r, 7); // Deref
        assert_eq!(*r.as_ref(), 7); // AsRef
        assert_eq!(r.into_inner(), 7);
    }

    #[test]
    fn const_constructors_build_valid_values_and_infer_their_type() {
        const P: Refined<i64, Positive> = positive(3);
        const N: Refined<i64, Negative> = negative(-4);
        const NN: Refined<i64, NonNegative> = non_negative(0);
        const Z: Refined<i64, NonZero> = nonzero(-9);
        const R: Refined<i64, InRange<1, 100>> = in_range::<1, 100>(100);
        assert_eq!(*P.get(), 3);
        assert_eq!(*N.get(), -4);
        assert_eq!(*NN.get(), 0);
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
    fn negative_rejects_zero() {
        let _ = negative(0);
    }

    #[test]
    #[should_panic]
    fn non_negative_rejects_below_zero() {
        let _ = non_negative(-1);
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
        assert_eq!(size_of::<Refined<i64, Positive>>(), size_of::<i64>());
        assert_eq!(
            size_of::<Refined<Refined<i64, Positive>, NonZero>>(),
            size_of::<i64>(),
            "nested refinement collapses to the inner value's size"
        );
    }

    #[test]
    fn ordering_hashing_and_display_follow_the_inner_value() {
        let small = Refined::<i64, Positive>::try_new(3).unwrap();
        let large = Refined::<i64, Positive>::try_new(9).unwrap();
        // Ord / PartialOrd
        assert!(small < large);
        assert_eq!(small.cmp(&large), Ordering::Less);
        assert_eq!(large.partial_cmp(&small), Some(Ordering::Greater));
        // Hash agrees with the inner value's hash
        assert_eq!(hash_of(&small), hash_of(&3i64));
        assert_ne!(hash_of(&small), hash_of(&large));
    }

    #[test]
    #[allow(clippy::clone_on_copy)] // deliberately exercising the manual Clone impl
    fn clone_copy_debug_and_display_follow_the_inner_value() {
        let a = Refined::<i64, Positive>::try_new(4).unwrap();
        let b = a; // Copy
        let c = a.clone();
        assert_eq!(a, b);
        assert_eq!(a, c);
        assert_ne!(a, Refined::<i64, Positive>::try_new(5).unwrap());

        use core::fmt::Write;
        let mut sink = Sink::new();
        let _ = write!(sink, "{:?}|{}", a, a);
        assert_eq!(sink.as_str(), "Refined(4)|4", "Debug then Display");
    }

    /// A hasher-agnostic 64-bit hash of any `Hash` value, for equality comparison.
    fn hash_of<H: Hash>(value: &H) -> u64 {
        let mut hasher = Fnv(0xcbf29ce484222325);
        value.hash(&mut hasher);
        hasher.0
    }

    /// A tiny deterministic hasher so the test needs no external crate.
    struct Fnv(u64);
    impl Hasher for Fnv {
        fn finish(&self) -> u64 {
            self.0
        }
        fn write(&mut self, bytes: &[u8]) {
            for &byte in bytes {
                self.0 ^= u64::from(byte);
                self.0 = self.0.wrapping_mul(0x100000001b3);
            }
        }
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

    #[cfg(feature = "serde")]
    #[test]
    fn serde_round_trips_and_revalidates_on_deserialize() {
        // serialize as the bare inner value
        let r = Refined::<i64, Positive>::try_new(42).unwrap();
        let json = serde_json::to_string(&r).unwrap();
        assert_eq!(json, "42");

        // deserialize a valid value succeeds
        let back: Refined<i64, Positive> = serde_json::from_str("42").unwrap();
        assert_eq!(*back, 42);

        // deserialize a value that violates the predicate is REFUSED
        let bad: Result<Refined<i64, Positive>, _> = serde_json::from_str("-1");
        assert!(bad.is_err(), "invalid data must not deserialize into a refinement");
    }
}
