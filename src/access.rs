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
    fn take_owned(self) -> T;
}
