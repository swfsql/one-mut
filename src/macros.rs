#[macro_export]
macro_rules! tokens {
    ($last_token:expr) => {
        $crate::Token::from($last_token)
    };

    ($first_token:expr, $( $tail_tokens:expr),+ ) => {
        $crate::Token::from($first_token).then( crate::tokens!( $($tail_tokens),+ ) )
    };
}

#[macro_export]
macro_rules! try_on {
    ( $id:ident, [ $( $token:expr ),+ ] ) => {
        match $id {
            Ok(id) => id,
            Err(e) => return Err((e, crate::tokens!( $($token),+ ) )),
        }
    };
}

#[macro_export]
macro_rules! ok {
    ( $ok:expr, [ $( $token:expr ),+ ] ) => {
        Ok(($ok, crate::tokens!( $($token),+ ) ))
    };
}

#[macro_export]
macro_rules! err {
    ( $err:expr, [ $( $token:expr ),+ ] ) => {
        Err(($err, crate::tokens!( $($token),+ ) ))
    };
}
