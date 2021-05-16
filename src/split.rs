use crate::{OneMut, Token};
use paste::paste;
use std::marker::PhantomData;

pub trait Split<Lifetimes> {
    type Return;
    fn split(self) -> Self::Return;
}

// 12 11 10 9 8 7 6 5 4 3 2 1

#[allow(unused_macros)]
macro_rules! split_impls {
    ( $last:tt ) => {
        paste! {
            pub struct [<Lifetimes $last>]<
                [<'t $last>]
            >(
                #[allow(unused_parens)]
                PhantomData<(
                    & [<'t $last>] ()
                )>
            );

            impl<
                'tall,
                [<'t $last>],
                [<T $last>],
            > Split<[<Lifetimes $last>]<
                [<'t $last>],
            >> for Token<
                'tall,
                (
                    [<T $last>],
                )
            >
            where
                'tall: [<'t $last>],
            {
                type Return = (
                    Token<[<'t $last>], [<T $last>]>,
                );
                fn split(self) -> Self::Return {
                    (
                        Token(PhantomData), // last
                    )
                }
            }

            pub trait [<FromApply $last>]<
                [<T $last>]
                > {
                type Input;
                type Return: crate::ResultLike;

                #[allow(clippy::type_complexity)]
                fn from_apply<
                    'tokens,
                    [<'t $last>]
                >(
                    one_muts: (
                        OneMut<[<'t $last>], [<T $last>]>,
                    ),
                    input: Self::Input,
                ) -> crate::AllOrNone<
                    'tokens,
                    <Self::Return as crate::ResultLike>::Ok,
                    <Self::Return as crate::ResultLike>::Err,
                    (
                        [<T $last>],
                    )
                >;
            }



        }
    };
    ( $first:tt, $( $tail:tt),+  ) => {
        paste! {
            pub struct [<Lifetimes $first>]<
                [<'t $first>],
                $( [<'t $tail>], )+
            >(
                PhantomData<(
                    & [<'t $first>] (),
                    $( &[<'t $tail>] (), )+
                )>
            );

            impl<
                'tall,
                [<'t $first>],
                $( [<'t $tail>], )+
                [<T $first>],
                $( [<T $tail>], )+
            > Split<[<Lifetimes $first>]<
                [<'t $first>],
                $( [<'t $tail>], )+
            >> for Token<
                'tall,
                (
                    [<T $first>],
                    $( [<T $tail>], )+
                )
            >
            where
                'tall: $( [<'t $tail>] + )+ [<'t $first>],
            {
                type Return = (
                    Token<[<'t $first>], [<T $first>]>,
                    $( Token<[<'t $tail>], [<T $tail >]>, )+
                );
                fn split(self) -> Self::Return {
                    (
                        Token(PhantomData), // first
                        $(
                            #[allow(unused_doc_comments)]
                            #[doc = "tail `" $tail "` member."]
                            Token(PhantomData),
                        )+
                    )
                }
            }

            pub trait [<FromApply $first>]<
                [<T $first>],
                $( [<T $tail>], )+
                > {
                type Input;
                type Return: crate::ResultLike;

                #[allow(clippy::type_complexity)]
                fn from_apply<
                    'tokens,
                    [<'t $first>],
                    $( [<'t $tail>], )+
                >(
                    one_muts: (
                        OneMut<[<'t $first>], [<T $first>]>,
                        $( OneMut<[<'t $tail>], [<T $tail>]>, )+
                    ),
                    input: Self::Input,
                ) -> crate::AllOrNone<
                    'tokens,
                    <Self::Return as crate::ResultLike>::Ok,
                    <Self::Return as crate::ResultLike>::Err,
                    (
                        [<T $first>],
                        $( [<T $tail>], )+
                    )
                >;
            }

            split_impls! { $( $tail ),+  }
        }
    };
}
split_impls! {12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1}
