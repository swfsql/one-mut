use super::{target, Apply, ConsumedToken, PartialApply, Take, TakeOwned, Token};

/// Container of `Prepared` items.
///
/// During `apply`, copies of `A1` and `A2` are lazily modified,
/// and only after both modifications successfully were executed,
/// the original `A1` and `A2` are replaced with the modified ones.
pub struct Chain<A1, A2> {
    a1: A1,
    a2: A2,
}

impl<A1, A2> Chain<A1, A2> {
    pub fn new(a1: A1, a2: A2) -> Self {
        Self { a1, a2 }
    }

    // TODO: need to test
    pub fn chain<A3, F3>(self, a3: A3) -> Chain<(A1, A2), A3> {
        Chain::new((self.a1, self.a2), a3)
    }
}

impl<'t, A1, A2, F1, F2> TakeOwned<(F1, F2), target::Function> for Chain<A1, A2>
where
    A1: TakeOwned<F1, target::Function>,
    A2: TakeOwned<F2, target::Function>,
{
    fn take_owned(self) -> (F1, F2) {
        let f1 = self.a1.take_owned();
        let f2 = self.a2.take_owned();
        (f1, f2)
    }
}

impl<'t, A1, A2, T1, T2, F1, F2, O1, O2, E> PartialApply<(T1, T2), (F1, F2), (O1, O2), E>
    for Chain<A1, A2>
where
    A1: PartialApply<T1, F1, O1, E>,
    A2: PartialApply<T2, F2, O2, E>,
{
    fn get_next(&self) -> (T1, T2) {
        let t1 = A1::get_next(&self.a1);
        let t2 = A2::get_next(&self.a2);
        (t1, t2)
    }

    #[allow(clippy::type_complexity)]
    fn modify_next(
        (next1, next2): (T1, T2),
        (f1, f2): (F1, F2),
    ) -> Result<((O1, O2), (T1, T2)), E> {
        let (o1, next1) = A1::modify_next(next1, f1)?;
        let (o2, next2) = A2::modify_next(next2, f2)?;
        Ok(((o1, o2), (next1, next2)))
    }

    fn replace(&mut self, (next1, next2): (T1, T2)) {
        A1::replace(&mut self.a1, next1);
        A2::replace(&mut self.a2, next2);
    }
}

unsafe impl<'t1, 't2, 'tboth, A1, A2, T1, T2, F1, F2, O, E> Apply<'tboth, (T1, T2), (F1, F2), O, E>
    for Chain<A1, A2>
where
    Self: PartialApply<(T1, T2), (F1, F2), O, E>,
    A1: Take<F1, target::Function> + TakeOwned<Token<'t1, T1>, target::Token>,
    A2: Take<F2, target::Function> + TakeOwned<Token<'t2, T2>, target::Token>,
    T1: 't1,
    T2: 't2,
    F1: 't1 + Clone,
    F2: 't2 + Clone,
{
    fn apply(mut self) -> crate::AllOrNone<'tboth, O, E, (T1, T2)> {
        let next = Self::get_next(&self);
        let f1: &mut F1 = self.a1.take_mut();
        let f2: &mut F2 = self.a2.take_mut();

        // modify both copies
        let (o, (next1, next2)) = match Self::modify_next(next, (f1.clone(), f2.clone())) {
            Ok((v1, v2)) => (v1, v2),
            Err(e) => {
                let t1: Token<T1> = self.a1.take_owned();
                let t2: Token<T2> = self.a2.take_owned();
                return Err((e, t1.then(t2)));
            }
        };

        // only replace after both modifications were successfull
        Self::replace(&mut self, (next1, next2));

        let t1: Token<T1> = self.a1.take_owned();
        let t2: Token<T2> = self.a2.take_owned();
        let consumed1 = ConsumedToken::from(t1);
        let consumed2 = ConsumedToken::from(t2);

        // merge the consumed tokens
        let tokens = consumed1.then(consumed2);
        Ok((o, tokens))
    }
}
