pub mod macros;
pub mod split;

pub mod access;
pub mod chain;
pub mod prepared;
pub mod token;

pub use access::{target, Take, TakeOwned};
pub use chain::Chain;
pub use prepared::Prepared;
pub use token::{ConsumedToken, Token, UpgraderToken};

pub mod from_apply {
    pub use crate::split::{
        FromApply1, FromApply10, FromApply11, FromApply12, FromApply2, FromApply3, FromApply4,
        FromApply5, FromApply6, FromApply7, FromApply8, FromApply9,
    };
}

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

/// Allows shared access into the Token.
impl<'t, T> AsRef<T> for OneMut<'t, T> {
    fn as_ref(&self) -> &T {
        self.inner
    }
}

impl<'t, T> OneMut<'t, T> {
    pub fn new(inner: &'t mut T) -> Self {
        let (token, inner) = Token::new(inner);
        Self { inner, token }
    }

    /// Defines how `T` should be mutated, given an `Ok` response.
    ///
    /// The definition is stored to be lazily applied, for when the
    /// `Prepared` values get's an `apply()`.  
    ///
    /// _A priori_, all mutations are applied into a copy of `T`.  
    /// - `Err` signals for the (potentially changed) copy of `T`
    /// to be discarded, and for the original `T` to be kept intact.  
    /// - `Ok` signals for the (potentially changed) copy of `T`
    /// to be replaced into the original `T`, while the old value of the
    /// original `T` to be discarded.
    ///
    /// # Safety
    ///
    /// You must pay attention to the `Result` signaling.  
    ///
    /// - `Err` means that the original `T` _won't_ be changed.  
    /// - `Ok` means that the original `T` _will_ be changed.
    ///
    /// Also, you must guarantee that your early-return logic is correct.  
    /// That means ensuring that no `Ok` returns happened where `Err` should
    /// have been returned, and ensuring that you don't early-return an `Ok`
    /// before finalizing all of your necessary mutations.  
    /// Otherwise you'll have inconsistent internal state.
    pub unsafe fn prepare<F, E>(self, f: F) -> Prepared<OneMut<'t, T>, T, F, E> {
        Prepared::new(self, f)
    }

    pub fn unchecked_prepare<F, E>(self, f: F) -> Prepared<OneMut<'t, T>, T, F, E> {
        Prepared::new(self, f)
    }

    /// Skips changing `T` by using an `|_| Ok(())` on `prepare()`.  
    ///
    /// This may be useful for easily chaining `Prepared` values.
    #[allow(clippy::type_complexity)]
    pub fn unchecked_skip<E>(self) -> Prepared<OneMut<'t, T>, T, fn(&mut T) -> Result<(), E>, E> {
        Prepared::new(self, |_t| Ok(()))
    }

    /// Skips changing `T` by using an `|_| Ok(())` on `prepare()`.  
    ///
    /// This may be useful for easily chaining `Prepared` values.
    ///
    /// # Safety
    /// (entirely logical)
    ///
    /// You must guarantee that this value being skipped of mutation is
    /// logically correct.
    #[allow(clippy::type_complexity)]
    pub unsafe fn skip<E>(self) -> Prepared<OneMut<'t, T>, T, fn(&mut T) -> Result<(), E>, E> {
        self.unchecked_skip()
    }

    /// Consumes the token without changing `T`.
    pub fn unchecked_consume(self) -> ConsumedToken<'t, T> {
        self.token.consume()
    }

    /// Consumes the token without changing `T`.
    ///
    /// # Safety
    /// (entirely logical)
    ///
    /// You must guarantee that this value being skipped of mutation is
    /// logically correct.
    pub unsafe fn consume(self) -> ConsumedToken<'t, T> {
        self.token.consume()
    }

    /// Consumes the container `&'t mut T` to create an item `TP<L>`,
    /// while storing the container's token into `UpgraderToken`.
    /// The item `L` may later be modified, and the item's consumed
    /// token may be upgraded into a container's consumed token.
    ///
    /// # Safety
    ///
    /// `f1` must ensure that the container is not modified,
    /// such as pushing or removing items into it.
    pub unsafe fn downgrade<'l, F1, L>(self, f1: F1) -> (UpgraderToken<'t, 'l, T, L>, OneMut<'l, L>)
    where
        F1: FnOnce(&'t mut T) -> &'l mut L,
    {
        let l = OneMut::new(f1(self.inner));
        let u = UpgraderToken::new(self.token, &l);
        (u, l)
    }

    pub fn unchecked_token(self) -> Token<'t, T> {
        self.token
    }

    /// # Safety
    /// (entirely logical)
    ///
    /// You must guarantee that this value being skipped of mutation is
    /// logically correct.
    pub unsafe fn token(self) -> Token<'t, T> {
        self.token
    }
}

/// Extends `Result` relating `Ok` with `ConsumedToken`s and `Err`
/// with `Token`s.
///
/// - The `Ok` case enforces that _all_ `Token`s were consumed exactly
/// once (either by a mut access or by an intentional skipping
/// of such),  
/// - And the `Err` case enforces that _none_ of the `Tokens` were consumed,
/// ie. no mut access occurred.
pub type AllOrNone<'tokens, T, E, Tokens> =
    std::result::Result<(T, ConsumedToken<'tokens, Tokens>), (E, Token<'tokens, Tokens>)>;

pub trait ResultLike {
    type Ok;
    type Err;
}

impl<T, E> ResultLike for Result<T, E> {
    type Ok = T;
    type Err = E;
}

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
