pub mod macros;

pub mod access;
pub mod chain;
pub mod prepared;
pub mod token;

pub use access::{target, Take, TakeOwned};
pub use chain::Chain;
pub use prepared::Prepared;
pub use token::{ConsumedToken, Token};

/// Controls mutation access into `T`, allowing at most one
/// scoped mut access by using a `Token`.
///
/// The mutation is allowed in a particular scope, which gets
/// lazily executed in a copy of `T`. If no errors occurred,
/// the copy then gets placed into the original `T`.
///
/// The mutation's scope exists on `prepare()`.  
/// The lazy appliance exists on a `Prepared::apply()`.
///
/// - If the appliance is successful, the `Token` get's consumed
/// and doesn't allow further mut accesses. The `ConsumedToken`
/// can prove that `T` got linearly accessed, ie. that it
/// got changed exactly once, even if such "change" was an
/// intentional `skip()`.  
/// - If the appliance fails, `T` never get's changed and only the
/// `Token` is returned. This means that further accesses are
/// disallowed and the `Token` can prove that `T` stayed unchanged.
pub struct OneMut<'t, T> {
    inner: &'t mut T,
    token: Token<'t, T>,
}

/// Extends `Result` relating `Ok` with `ConsumedToken`s and `Err`
/// with `Token`s.
///
/// - The `Ok` case enforces that _all_ `Token`s were consumed exactly
/// once (either by a mut access or by an intentional skipping
/// of such),  
/// - And the `Err` case enforces that _none_ of `Tokens` were consumed,
/// ie. no mut access occurred.
pub type AllOrNone<'tokens, T, E, Tokens> =
    std::result::Result<(T, ConsumedToken<'tokens, Tokens>), (E, Token<'tokens, Tokens>)>;

/// Trait types:
///
/// - `T` is the protected type.
/// - `F` is the scoped closure that will mut access `T`.
/// - `O` is `F`'s `Ok` return type.
/// - `E` is `F`'s `Err` return type.
pub trait PartialApply<T, F, O, E> {
    /// Creates a copy of `T`.
    fn get_next(&self) -> T;
    /// Applies a modification into a `T` (presumably the copy of `T`).
    fn modify_next(next: T, f: F) -> Result<(O, T), E>;
    /// Replaces the original `T` with the modified copy of `T`.
    fn replace(&mut self, next: T);
}

/// # Safety
///
/// `apply()` has complete mut access into the original `T`, and
/// it must guarantee that either:
///
/// - The original `T` is completely untouched and an `Err` is returned;
/// - The original `T` is completely updated, and an `Ok` is returned.
///
/// That is to say that once any change start to happen in the
/// original `T`, no failure going forward, into the end of `apply()`,
/// is allowed.
///
/// For example, `Chain` represents it's `T` by `(T1, T2)` and it guarantees
/// the safety by first executing all mutations into the copies of `T1`
/// and `T2`, any of which can fail, and then only after that, where there is
/// no more failure possibilities, it starts updating the original values of
/// `T1` and `T2`; which then `apply()` which is finalized with an `Ok`.
///
///
/// Trait types:
///
/// - `T` is the protected type.
/// - `F` is the scoped closure that will mut access `T`.
/// - `O` is `F`'s `Ok` return type.
/// - `E` is `F`'s `Err` return type.
pub unsafe trait Apply<'t, T, F, O, E> {
    /// Copies `T`, modifies it, and then replaces it into the
    /// original `T`.
    ///
    /// - `Ok` implies the original `T` got completely modified
    /// (ie. no incomplete modifications occurred),
    /// - `Err` implies the original `T` is untouched.
    fn apply(self) -> AllOrNone<'t, O, E, T>;
}
