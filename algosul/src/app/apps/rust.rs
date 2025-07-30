use std::{
    borrow::Cow,
    fmt::{Display, Formatter},
    io,
    process::ExitStatus,
    str::FromStr,
    string::FromUtf8Error,
    time::SystemTimeError,
};

use serde::{Deserialize, Serialize};
use tokio::{sync::TryLockError, task::JoinError};
pub mod rustup;
pub mod utils;
pub use rustup::Rustup;
#[derive(Debug)]
pub enum Error {
    Unsupported(Cow<'static, str>),
    IOError(io::Error),
    TaskJoinError(tokio::task::JoinError),
    InnerError(Cow<'static, str>),
    Failed {
        exit_status: ExitStatus,
        stdin:       Cow<'static, str>,
        stdout:      Cow<'static, str>,
        stderr:      Cow<'static, str>,
    },
    FailedToGetHomeDir,
    RequestError(reqwest::Error),
    FromUtf8Error(FromUtf8Error),
    RegexError(regex::Error),
    VersionStringNoMatch,
    TryLockError(TryLockError),
    SystemTimeError(SystemTimeError),
}
pub type Result<T> = std::result::Result<T, Error>;
#[derive(
    Default,
    Debug,
    Copy,
    Clone,
    Eq,
    PartialEq,
    Hash,
    Ord,
    PartialOrd,
    Serialize,
    Deserialize,
)]
pub enum Toolchain {
    #[default]
    Stable,
    Beta,
    Nightly,
    None,
}
#[derive(
    Default,
    Debug,
    Clone,
    Eq,
    PartialEq,
    Hash,
    Ord,
    PartialOrd,
    Serialize,
    Deserialize,
)]
pub enum HostTriple {
    #[default]
    Host,
    /// e.g. x86_64-unknown-linux-gnu
    Target(String),
}
#[derive(
    Default,
    Debug,
    Copy,
    Clone,
    Eq,
    PartialEq,
    Hash,
    Ord,
    PartialOrd,
    Serialize,
    Deserialize,
)]
pub enum Profile {
    Minimal,
    #[default]
    Default,
    Complete,
}
impl Display for Toolchain {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Toolchain::Stable => f.write_str("stable"),
            Toolchain::Beta => f.write_str("beta"),
            Toolchain::Nightly => f.write_str("nightly"),
            Toolchain::None => f.write_str("none"),
        }
    }
}
impl FromStr for Toolchain {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "stable" => Ok(Toolchain::Stable),
            "beta" => Ok(Toolchain::Beta),
            "nightly" => Ok(Toolchain::Nightly),
            "none" => Ok(Toolchain::None),
            _ => Err(()),
        }
    }
}
impl Display for HostTriple {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            HostTriple::Host => f.write_str("host"),
            HostTriple::Target(target) => f.write_str(target),
        }
    }
}
impl FromStr for HostTriple {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "host" => Ok(HostTriple::Host),
            s => Ok(HostTriple::Target(s.to_string())),
        }
    }
}
impl Display for Profile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Profile::Minimal => f.write_str("minimal"),
            Profile::Default => f.write_str("default"),
            Profile::Complete => f.write_str("complete"),
        }
    }
}
impl FromStr for Profile {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "minimal" => Ok(Profile::Minimal),
            "default" => Ok(Profile::Default),
            "complete" => Ok(Profile::Complete),
            _ => Err(()),
        }
    }
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Unsupported(info) => {
                f.write_fmt(format_args!("Unsupported: {info}"))
            }
            Error::IOError(e) => f.write_fmt(format_args!("IO error: {e}")),
            Error::TaskJoinError(e) => {
                f.write_fmt(format_args!("Task join error: {e}"))
            }
            Error::InnerError(info) => {
                f.write_fmt(format_args!("Inner error: {info}"))
            }
            Error::Failed { exit_status, stdin, stdout, stderr } => f
                .write_fmt(format_args!(
                    "Failed:\n - exit status: {exit_status}\n - \
                     stdin:\n{stdin}\n\n - stdout:\n{stdout}\n\n - \
                     stderr:\n{stderr}"
                )),
            Error::FailedToGetHomeDir => {
                f.write_fmt(format_args!("failed to get HOME dir"))
            }
            Error::RequestError(e) => {
                f.write_fmt(format_args!("request error: {e}"))
            }
            Error::FromUtf8Error(e) => {
                f.write_fmt(format_args!("from utf8 error: {e}"))
            }
            Error::RegexError(e) => {
                f.write_fmt(format_args!("regex error: {e}"))
            }
            Error::VersionStringNoMatch => {
                f.write_str("version string no match")
            }
            Error::TryLockError(e) => {
                f.write_fmt(format_args!("try lock error: {e}"))
            }
            Error::SystemTimeError(e) => {
                f.write_fmt(format_args!("system time error: {e}"))
            }
        }
    }
}
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Unsupported(_) => None,
            Error::IOError(e) => Some(e),
            Error::TaskJoinError(e) => Some(e),
            Error::InnerError(_) => None,
            Error::Failed { .. } => None,
            Error::FailedToGetHomeDir => None,
            Error::RequestError(e) => Some(e),
            Error::FromUtf8Error(e) => Some(e),
            Error::RegexError(e) => Some(e),
            Error::VersionStringNoMatch => None,
            Error::TryLockError(e) => Some(e),
            Error::SystemTimeError(e) => Some(e),
        }
    }
}
impl From<JoinError> for Error {
    fn from(value: JoinError) -> Self { Self::TaskJoinError(value) }
}
impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self { Self::IOError(value) }
}
impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self { Self::RequestError(value) }
}
impl From<FromUtf8Error> for Error {
    fn from(value: FromUtf8Error) -> Self { Self::FromUtf8Error(value) }
}
impl From<regex::Error> for Error {
    fn from(value: regex::Error) -> Self { Self::RegexError(value) }
}
impl From<TryLockError> for Error {
    fn from(value: TryLockError) -> Self { Self::TryLockError(value) }
}
impl From<SystemTimeError> for Error {
    fn from(value: SystemTimeError) -> Self { Self::SystemTimeError(value) }
}
#[cfg(test)]
mod tests {
    use log::info;
    use utils::ToRustVersion;

    use super::*;
    use crate::{app::AppOper, process::Process};
    #[tokio::test]
    #[ignore]
    async fn install_rustup()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        env_logger::init();
        Rustup::installer().await?.run().await?;
        Ok::<_, Box<dyn std::error::Error>>(())
    }
    #[tokio::test]
    async fn rustup_version()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        env_logger::init();
        let rustup = Rustup::get_by_current_user().await?;
        let version = rustup.full_version_str().await?;
        let version = version.to_rust_version()?;
        info!("version: {version:?}");
        Ok::<_, Box<dyn std::error::Error>>(())
    }
}
