use onemut::{from_apply::FromApply2, ok_consumed, OneMut};

#[derive(Clone, Debug)]
struct A(pub u8);

#[derive(Clone, Debug)]
struct B(pub u8);

#[test]
fn example_1() {
    let mut a = A(0);
    let mut b = B(0);

    let amut = OneMut::new(&mut a);
    let bmut = OneMut::new(&mut b);
    let (ok, _toks) = Ex1::from_apply((amut, bmut), false).unwrap();

    // the internal state is kept coeherent
    assert_eq!(a.0, b.0);
    assert_eq!(ok, 2);
    // GOOD!
}

struct Ex1;
impl FromApply2<A, B> for Ex1 {
    type Input = bool;
    type Return = Result<u8, ()>;

    fn from_apply<'tokens, 't1, 't2>(
        (a, b): (OneMut<'t1, A>, OneMut<'t2, B>),
        _cond: Self::Input,
    ) -> onemut::AllOrNone<'tokens, u8, (), (A, B)> {
        use onemut::Apply;
        let a = a.unchecked_prepare(|a: &mut A| {
            a.0 += 1;
            Ok(a.0)
        });
        let b = b.unchecked_prepare(|b: &mut B| {
            b.0 += 1;
            Ok(b.0)
        });
        let ((a0, b0), toks) = a.chain(b).apply()?;
        Ok((a0 + b0, toks))
    }
}

#[test]
fn example_2() {
    let mut a = A(0);
    let mut b = B(0);

    let amut = OneMut::new(&mut a);
    let bmut = OneMut::new(&mut b);
    let (_err, _toks) = Ex2::from_apply((amut, bmut), true).unwrap_err();

    // the internal state is kept intact
    assert_eq!(a.0, b.0);
    // GOOD!
}

struct Ex2;
impl FromApply2<A, B> for Ex2 {
    type Input = bool;
    type Return = Result<u8, ()>;

    fn from_apply<'tokens, 't1, 't2>(
        (a, b): (OneMut<'t1, A>, OneMut<'t2, B>),
        cond: Self::Input,
    ) -> onemut::AllOrNone<'tokens, u8, (), (A, B)> {
        use onemut::Apply;
        let a = a.unchecked_prepare(|a: &mut A| {
            a.0 += 1;

            // mistakenly Err early-return
            // (from inside a preparation)
            if cond {
                return Err(());
            }

            Ok(a.0)
        });

        let b = b.unchecked_prepare(|b: &mut B| {
            b.0 += 1;
            Ok(b.0)
        });
        let ((a0, b0), toks) = a.chain(b).apply()?;
        Ok((a0 + b0, toks))
    }
}

#[test]
fn example_2b() {
    let mut a = A(0);
    let mut b = B(0);

    let amut = OneMut::new(&mut a);
    let bmut = OneMut::new(&mut b);
    let (_err, _toks) = Ex2b::from_apply((amut, bmut), true).unwrap_err();

    // the internal state is kept intact
    assert_eq!(a.0, b.0);
    // GOOD!
}

struct Ex2b;
impl FromApply2<A, B> for Ex2b {
    type Input = bool;
    type Return = Result<u8, ()>;

    fn from_apply<'tokens, 't1, 't2>(
        (a, b): (OneMut<'t1, A>, OneMut<'t2, B>),
        cond: Self::Input,
    ) -> onemut::AllOrNone<'tokens, u8, (), (A, B)> {
        use onemut::Apply;
        let a = a.unchecked_prepare(|a: &mut A| {
            a.0 += 1;
            Ok(a.0)
        });

        // mistakenly Err early-return
        // (from outside a preparation)
        if cond {
            // // return Err(());
            //               ^^ expected `((), onemut::Token<'_, (A, B)>)`,
            //               ^^    found `()`
            //
            // cannot trivially early-return after a preparation.
            //
            // first, needs to capture and _cancel_ the preparation `a`,
            // and then also needs to capture the unprepared `b`.
            return onemut::err!((), [a.unchecked_cancel(), b]);
        }

        let b = b.unchecked_prepare(|b: &mut B| {
            b.0 += 1;
            Ok(b.0)
        });
        let ((a0, b0), toks) = a.chain(b).apply()?;
        Ok((a0 + b0, toks))
    }
}

#[test]
fn example_3() {
    let mut a = A(0);
    let mut b = B(0);

    let amut = OneMut::new(&mut a);
    let bmut = OneMut::new(&mut b);
    let (ok, _toks) = Ex3::from_apply((amut, bmut), true).unwrap();

    assert_eq!(ok, 1);
    assert!(a.0 != b.0);
    // BAD!
}

struct Ex3;
impl FromApply2<A, B> for Ex3 {
    type Input = bool;
    type Return = Result<u8, ()>;

    fn from_apply<'tokens, 't1, 't2>(
        (a, b): (OneMut<'t1, A>, OneMut<'t2, B>),
        cond: Self::Input,
    ) -> onemut::AllOrNone<'tokens, u8, (), (A, B)> {
        use onemut::Apply;
        let a = a.unchecked_prepare(|a: &mut A| {
            a.0 += 1;

            Ok(a.0)
        });

        // mistakenly Ok early-return
        if cond {
            // // return Ok(_);
            //
            // cannot early-ok-return without either applying
            // on `a`, or cancelling on it.
            // (cancelling was shown on `example_2b`)

            // early ok with early-applying example:
            let (a_res, a_tok) = a.apply().unwrap();
            return ok_consumed!(a_res, [a_tok, b]);
            // still needs to indicate a forced-consumption of `b`.
        };

        let b = b.unchecked_prepare(|b: &mut B| {
            b.0 += 1;
            Ok(b.0)
        });
        let ((a0, b0), toks) = a.chain(b).apply()?;
        Ok((a0 + b0, toks))
    }
}

#[test]
fn example_4() {
    let mut a = A(0);
    let mut b = B(0);

    let amut = OneMut::new(&mut a);
    let bmut = OneMut::new(&mut b);
    let (ok, _toks) = Ex4::from_apply((amut, bmut), false).unwrap();

    assert_eq!(ok, 1);
    assert!(a.0 != b.0);
    // BAD!
}

struct Ex4;
impl FromApply2<A, B> for Ex4 {
    type Input = bool;
    type Return = Result<u8, ()>;

    fn from_apply<'tokens, 't1, 't2>(
        (a, b): (OneMut<'t1, A>, OneMut<'t2, B>),
        cond: Self::Input,
    ) -> onemut::AllOrNone<'tokens, u8, (), (A, B)> {
        use onemut::Apply;
        let a = a.unchecked_prepare(|a: &mut A| {
            if cond {
                a.0 += 1;
            }

            // if the mistake is inside this scope, there is not
            // much to do..
            Ok(a.0)
        });

        let b = b.unchecked_prepare(|b: &mut B| {
            b.0 += 1;
            Ok(b.0)
        });
        let ((a0, b0), toks) = a.chain(b).apply()?;
        Ok((a0 + b0, toks))
    }
}

#[test]
fn example_4b() {
    let mut a = A(0);
    let mut b = B(0);

    let amut = OneMut::new(&mut a);
    let bmut = OneMut::new(&mut b);
    let (_err, _toks) = Ex4b::from_apply((amut, bmut), false).unwrap_err();

    assert_eq!(a.0, b.0);
    // GOOD!
}

struct Ex4b;
impl FromApply2<A, B> for Ex4b {
    type Input = bool;
    type Return = Result<u8, ()>;

    fn from_apply<'tokens, 't1, 't2>(
        (a, b): (OneMut<'t1, A>, OneMut<'t2, B>),
        cond: Self::Input,
    ) -> onemut::AllOrNone<'tokens, u8, (), (A, B)> {
        use onemut::Apply;

        // if the mistake is outside of the preparation scope,
        // it requires an early-return.
        let a = if cond {
            a.unchecked_prepare(|a: &mut A| {
                a.0 += 1;
                Ok(a.0)
            })
        } else {
            // the else clausule is required,
            // but a different closure cannot be returned.
            //
            // this means that it would only be possible to
            // early return from here
            return onemut::err!((), [a, b]);
        };

        let b = b.unchecked_prepare(|b: &mut B| {
            b.0 += 1;
            Ok(b.0)
        });
        let ((a0, b0), toks) = a.chain(b).apply()?;
        Ok((a0 + b0, toks))
    }
}

#[test]
fn example_5() {
    let mut a = A(0);
    let mut b = B(0);

    let amut = OneMut::new(&mut a);
    let bmut = OneMut::new(&mut b);
    let (ok, _toks) = Ex5::from_apply((amut, bmut), true).unwrap();

    assert_eq!(ok, 2);
    assert_eq!(a.0, b.0);
    // GOOD!
}

struct Ex5;
impl FromApply2<A, B> for Ex5 {
    type Input = bool;
    type Return = Result<u8, ()>;

    fn from_apply<'tokens, 't1, 't2>(
        (a, b): (OneMut<'t1, A>, OneMut<'t2, B>),
        _cond: Self::Input,
    ) -> onemut::AllOrNone<'tokens, u8, (), (A, B)> {
        use onemut::Apply;

        let a = a.unchecked_prepare(|a: &mut A| {
            a.0 += 1;
            Ok(a.0)
        });

        let b = b.unchecked_prepare(|b: &mut B| {
            b.0 += 1;
            Ok(b.0)
        });

        // mistakenly mutates `a` again.
        //
        // (but cannot mut access `a` again, it got already prepared)
        // so this mistake cannot be made.

        let ((a0, b0), toks) = a.chain(b).apply()?;
        Ok((a0 + b0, toks))
    }
}
