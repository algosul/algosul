pub trait Process {
    type Error: std::error::Error;
    type Output;
    type Status;
    async fn run(&self) -> Result<Self::Output, Self::Error>;
    fn on_status_changed(
        &self,
        f: impl Fn(&Self::Status) -> Result<(), Self::Error> + Send + 'static,
    ) -> Result<(), Self::Error>;
}
