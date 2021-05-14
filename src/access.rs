/// Information to diverge some access trait implementations.
pub mod target {
    /// Access into a protected type.
    pub struct Type;

    /// Access into a token.
    pub struct Token;

    /// Access into a function.
    pub struct Function;
}

/// Indicates access into fields.
pub trait Take<T, Target> {
    fn take_ref(&self) -> &T;
    fn take_mut(&mut self) -> &mut T;
}

/// Indicates access into fields.
pub trait TakeOwned<T, Target> {
    // TODO: possibly refactor for the Token Target, which relates to
    // PartialApply::modify_next.
    //
    /// # Safety
    ///
    /// In the case of `Target` being `Token`, you must ensure that it is
    /// correct to take the `Token`s ownership. Either:
    ///
    /// - That it's correct to "consume" it, if to indicate a successful
    /// mutation of some `T`;
    /// - That it's correct to "return" it, if to indicate an unsuccessful
    /// mutation of some `T`.
    ///
    /// In both cases, this should be using by an `Apply::apply()` method,
    /// and only after an `PartialApply::modify_next` was tried.  
    /// A `Token` consumption should reflect an `Ok` case, and a `Token`
    /// return should reflect an `Err` case.
    unsafe fn take_owned(self) -> T;
}
