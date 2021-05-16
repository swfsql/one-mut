#[derive(Clone, Debug)]
struct A(pub u8);

#[derive(Clone, Debug)]
struct B(pub u8);

// normal test

#[test]
fn example_1() {
    let mut a = A(0);
    let mut b = B(0);

    let ok = example_1_(&mut a, &mut b, false).unwrap();

    assert_eq!(ok, 2);
    assert_eq!(a.0, b.0);
    // GOOD!
}

fn example_1_(a: &mut A, b: &mut B, _cond: bool) -> Result<u8, ()> {
    a.0 += 1;
    b.0 += 1;
    Ok(a.0 + b.0)
}

// mistakenly Err early-return

#[test]
fn example_2() {
    let mut a = A(0);
    let mut b = B(0);

    let _err = example_2_(&mut a, &mut b, true).unwrap_err();

    assert!(a.0 != b.0);
    // BAD!
}

fn example_2_(a: &mut A, b: &mut B, cond: bool) -> Result<u8, ()> {
    a.0 += 1;

    // mistakenly Err early-return
    if cond {
        return Err(());
    }

    b.0 += 1;
    Ok(a.0 + b.0)
}

// mistakenly Ok early-return

#[test]
fn example_3() {
    let mut a = A(0);
    let mut b = B(0);

    let ok = example_3_(&mut a, &mut b, true).unwrap();

    assert_eq!(ok, 1);
    assert!(a.0 != b.0);
    // BAD!
}

fn example_3_(a: &mut A, b: &mut B, cond: bool) -> Result<u8, ()> {
    a.0 += 1;

    // mistakenly Ok early-return
    if cond {
        return Ok(a.0);
    };

    b.0 += 1;
    Ok(a.0 + b.0)
}

#[test]
fn example_4() {
    let mut a = A(0);
    let mut b = B(0);

    let ok = example_4_(&mut a, &mut b, false).unwrap();

    assert_eq!(ok, 1);
    assert!(a.0 != b.0);
    // BAD!
}

fn example_4_(a: &mut A, b: &mut B, cond: bool) -> Result<u8, ()> {
    if cond {
        a.0 += 1;
    };
    // mistakenly forgets to mutate `a` on !cond

    b.0 += 1;

    Ok(a.0 + b.0)
}

#[test]
fn example_5() {
    let mut a = A(0);
    let mut b = B(0);

    let ok = example_5_(&mut a, &mut b, true).unwrap();

    assert_eq!(ok, 3);
    assert!(a.0 != b.0);
    // BAD!
}

fn example_5_(a: &mut A, b: &mut B, cond: bool) -> Result<u8, ()> {
    a.0 += 1;

    b.0 += 1;

    // mistakenly mutates `a` again
    if cond {
        a.0 += 1;
    };

    Ok(a.0 + b.0)
}
