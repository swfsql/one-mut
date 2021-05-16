use crate::{target, OneMut, Take, TakeOwned};
use std::marker::PhantomData;

/// A tag value that can be moved or consumed, and indicates
/// that modifications into `T` can happen only in a single scope.
///
/// See also `OneMut`.
#[derive(Debug)]
pub struct Token<'t, T>(pub(crate) PhantomData<&'t T>);

/// A tag value that can be moved, and indicates that `T` will
/// no longer be able to be modified.
#[derive(Debug)]
pub struct ConsumedToken<'t, T>(PhantomData<&'t T>);

/// A tag value related to containers.
///
/// This indicates that the container had some accessed item, and
/// depending on whether this item was modified or not, this tag can be
/// moved, consumed or be restored back into a `Token`.
#[derive(Debug)]
pub struct UpgraderToken<'u, 'l, U, L> {
    upper: Token<'u, U>,
    lower: PhantomData<&'l L>,
}

impl<'u, 'l, U, L> UpgraderToken<'u, 'l, U, L> {
    pub fn new(upper: Token<'u, U>, _lower: &OneMut<'l, L>) -> Self {
        Self {
            upper,
            lower: PhantomData,
        }
    }

    /// Consumes the token without changing `T` (as it's innaccesible).
    pub fn consume(self, _lower: ConsumedToken<'l, L>) -> ConsumedToken<'u, U> {
        self.upper.into()
    }

    /// Consumes the token without changing `T` (as it's innaccesible).
    pub fn returned(self, _lower: impl Into<Token<'l, L>>) -> Token<'u, U> {
        self.upper
    }

    /// Discards an unconsummed lower Token, and extracets the upper one.
    pub fn discard_lower(self, _lower: Token<'l, L>) -> Token<'u, U> {
        self.upper
    }
}

impl<'t, T> Token<'t, T> {
    pub(crate) fn new(t: &'t mut T) -> (Self, &'t mut T) {
        (Self(PhantomData), t)
    }

    /// Concatenate this token with another one.
    pub fn then<'t2, 'tboth, T2>(self, _token2: Token<'t2, T2>) -> Token<'tboth, (T, T2)> {
        Token(PhantomData)
    }

    /// Consumes the token.
    pub fn consume(self) -> ConsumedToken<'t, T> {
        self.into()
    }
}

impl<'t, T> ConsumedToken<'t, T> {
    /// Concatenates with another Consumed Token.
    pub fn then<'t2, 'tboth, T2>(
        self,
        _token2: ConsumedToken<'t2, T2>,
    ) -> ConsumedToken<'tboth, (T, T2)> {
        ConsumedToken(PhantomData)
    }
}

impl<'t1, 't2, 'tboth, T1, T2> ConsumedToken<'tboth, (T1, T2)>
where
    'tboth: 't1 + 't2,
{
    pub fn split2(self) -> (ConsumedToken<'t1, T1>, ConsumedToken<'t2, T2>) {
        (ConsumedToken(PhantomData), ConsumedToken(PhantomData))
    }
}

/// Consumes a Token.
impl<'t, T> From<Token<'t, T>> for ConsumedToken<'t, T> {
    fn from(token: Token<'t, T>) -> Self {
        ConsumedToken(token.0)
    }
}

pub unsafe trait UncheckedFrom<T>: Sized {
    /// Performs the conversion.
    fn unchecked_from(_: T) -> Self;
}

unsafe impl<'t, T> UncheckedFrom<OneMut<'t, T>> for Token<'t, T> {
    fn unchecked_from(t: OneMut<'t, T>) -> Self {
        t.unchecked_token()
    }
}

unsafe impl<'t, T> UncheckedFrom<Token<'t, T>> for Token<'t, T> {
    fn unchecked_from(t: Token<'t, T>) -> Self {
        t
    }
}

unsafe impl<'t, T> UncheckedFrom<OneMut<'t, T>> for ConsumedToken<'t, T> {
    fn unchecked_from(t: OneMut<'t, T>) -> Self {
        t.unchecked_consume()
    }
}

unsafe impl<'t, T> UncheckedFrom<ConsumedToken<'t, T>> for ConsumedToken<'t, T> {
    fn unchecked_from(t: ConsumedToken<'t, T>) -> Self {
        t
    }
}

impl<'t, T> Take<T, target::Type> for OneMut<'t, T> {
    fn take_ref(&self) -> &T {
        self.inner
    }

    fn take_mut(&mut self) -> &mut T {
        self.inner
    }
}

impl<'t, T> Take<Token<'t, T>, target::Token> for OneMut<'t, T> {
    fn take_ref(&self) -> &Token<'t, T> {
        &self.token
    }

    fn take_mut(&mut self) -> &mut Token<'t, T> {
        &mut self.token
    }
}

impl<'t, T> TakeOwned<Token<'t, T>, target::Token> for OneMut<'t, T> {
    unsafe fn take_owned(self) -> Token<'t, T> {
        self.token
    }
}
