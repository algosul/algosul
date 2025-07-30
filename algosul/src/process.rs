#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    #[default]
    Initial,
    Running,
    Success,
    Failure,
    Cancelled,
}
impl Status {
    pub fn is_initial(&self) -> bool { matches!(self, Status::Initial) }

    pub fn is_running(&self) -> bool { matches!(self, Status::Running) }

    pub fn is_success(&self) -> bool { matches!(self, Status::Success) }

    pub fn is_failure(&self) -> bool { matches!(self, Status::Failure) }

    pub fn is_cancelled(&self) -> bool { matches!(self, Status::Cancelled) }

    pub fn is_done(&self) -> bool {
        matches!(self, Status::Success | Status::Cancelled | Status::Failure)
    }
}
pub trait Process {
    type Count;
    type Error: std::error::Error + Send + Sync + 'static;
    type Output;
    async fn run(&self) -> Result<Self::Output, Self::Error>;
    async fn overall_progress(&self) -> Result<Self::Count, Self::Error>;
    async fn progress(&self) -> Result<Self::Count, Self::Error>;
    async fn status(&self) -> Result<Status, Self::Error>;
    fn try_status(&self) -> Result<Status, Self::Error>;
}
