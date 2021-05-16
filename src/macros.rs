#[macro_export]
macro_rules! tokens {
    ($last_token:expr) => {
        {
            use $crate::token::UncheckedFrom;
            $crate::Token::unchecked_from($last_token)
        }
    };

    ($first_token:expr, $( $tail_tokens:expr),+ ) => {
        {
            use $crate::token::UncheckedFrom;
            $crate::Token::unchecked_from($first_token).then( $crate::tokens!( $($tail_tokens),+ ) )
        }
    };
}

#[macro_export]
macro_rules! tokens_consumed {
    ($last_token:expr) => {
        {
            use $crate::token::UncheckedFrom;
            $crate::ConsumedToken::unchecked_from($last_token)
        }
    };

    ($first_token:expr, $( $tail_tokens:expr),+ ) => {
        {
            use $crate::token::UncheckedFrom;
            $crate::ConsumedToken::unchecked_from($first_token).then( $crate::tokens_consumed!( $($tail_tokens),+ ) )
        }
    };
}

#[macro_export]
macro_rules! try_on {
    ( $id:ident, [ $( $token:expr ),+ ] ) => {
        match $id {
            Ok(id) => id,
            Err(e) => return Err((e, $crate::tokens!( $($token),+ ) )),
        }
    };
}

#[macro_export]
macro_rules! ok {
    ( $ok:expr, [ $( $token:expr ),+ ] ) => {
        Ok(($ok, $crate::tokens!( $($token),+ ) ))
    };
}

#[macro_export]
macro_rules! ok_consumed {
    ( $ok:expr, [ $( $token:expr ),+ ] ) => {
        Ok(($ok, $crate::tokens_consumed!( $($token),+ ) ))
    };
}

#[macro_export]
macro_rules! err {
    ( $err:expr, [ $( $token:expr ),+ ] ) => {
        Err(($err, $crate::tokens!( $($token),+ ) ))
    };
}
