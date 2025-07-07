pub trait Builder {
    type Error;
    type Output: HasBuilder<Builder = Self>;
    fn new() -> Self;
    fn build(&self) -> Result<Self::Output, Self::Error>;
}
pub trait HasBuilder {
    type Builder: Builder<Output = Self>;
    fn builder() -> Self::Builder { Self::Builder::new() }
}
