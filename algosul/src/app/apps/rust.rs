use std::{
  borrow::Cow,
  fmt::{Display, Formatter},
  io,
  str::FromStr,
  string::FromUtf8Error,
  time::SystemTimeError,
};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{sync::TryLockError, task::JoinError};
pub mod cargo;
pub mod rustc;
pub mod rustup;
pub mod utils;

pub use cargo::Cargo;
pub use rustc::Rustc;
pub use rustup::Rustup;
#[derive(Debug, Error)]
pub enum Error
{
  #[error("Unsupported: {0}")]
  Unsupported(Cow<'static, str>),
  #[error("IO error: {0}")]
  IOError(#[from] io::Error),
  #[error("Task join error: {0}")]
  TaskJoinError(#[from] JoinError),
  #[error("Inner error: {0}")]
  InnerError(Cow<'static, str>),
  #[error("Failed: {0}")]
  Failed(Cow<'static, str>),
  #[error("Failed to get HOME dir")]
  FailedToGetHomeDir,
  #[error("Request error: {0}")]
  RequestError(#[from] reqwest::Error),
  #[error("From utf8 error: {0}")]
  FromUtf8Error(#[from] FromUtf8Error),
  #[error("Regex error: {0}")]
  RegexError(#[from] regex::Error),
  #[error("Version string no match")]
  VersionStringNoMatch,
  #[error("Try lock error: {0}")]
  TryLockError(#[from] TryLockError),
  #[error("System time error: {0}")]
  SystemTimeError(#[from] SystemTimeError),
  #[error("Invalid rustup-init.sh content")]
  InvalidRsinitShContent,
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
pub enum Toolchain
{
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
pub enum HostTriple
{
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
pub enum Profile
{
  Minimal,
  #[default]
  Default,
  Complete,
}
impl HostTriple
{
  pub fn to_string(self) -> Option<String>
  {
    match self
    {
      HostTriple::Host => None,
      HostTriple::Target(target) => Some(target),
    }
  }
}
impl Display for Toolchain
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
  {
    match self
    {
      Toolchain::Stable => f.write_str("stable"),
      Toolchain::Beta => f.write_str("beta"),
      Toolchain::Nightly => f.write_str("nightly"),
      Toolchain::None => f.write_str("none"),
    }
  }
}
impl FromStr for Toolchain
{
  type Err = ();

  fn from_str(s: &str) -> std::result::Result<Self, Self::Err>
  {
    match s
    {
      "stable" => Ok(Toolchain::Stable),
      "beta" => Ok(Toolchain::Beta),
      "nightly" => Ok(Toolchain::Nightly),
      "none" => Ok(Toolchain::None),
      _ => Err(()),
    }
  }
}
impl Display for HostTriple
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
  {
    match self
    {
      HostTriple::Host => f.write_str("host"),
      HostTriple::Target(target) => f.write_str(target),
    }
  }
}
impl FromStr for HostTriple
{
  type Err = ();

  fn from_str(s: &str) -> std::result::Result<Self, Self::Err>
  {
    match s
    {
      "host" => Ok(HostTriple::Host),
      s => Ok(HostTriple::Target(s.to_string())),
    }
  }
}
impl Display for Profile
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
  {
    match self
    {
      Profile::Minimal => f.write_str("minimal"),
      Profile::Default => f.write_str("default"),
      Profile::Complete => f.write_str("complete"),
    }
  }
}
impl FromStr for Profile
{
  type Err = ();

  fn from_str(s: &str) -> std::result::Result<Self, Self::Err>
  {
    match s
    {
      "minimal" => Ok(Profile::Minimal),
      "default" => Ok(Profile::Default),
      "complete" => Ok(Profile::Complete),
      _ => Err(()),
    }
  }
}
#[cfg(test)]
mod tests
{
  use log::info;
  use utils::ToRustVersion;

  use super::*;
  use crate::{app::AppOper, process::Process};
  #[tokio::test]
  #[ignore]
  async fn install_rustup()
  -> std::result::Result<(), Box<dyn std::error::Error>>
  {
    env_logger::init();
    let mut installer = Rustup::installer().await?;
    installer.on_status_changed(|status| {
      info!("status: {status:?}");
      Ok(())
    })?;
    let rustup = installer.run().await?;
    info!("rustup installed: {rustup:?}");
    let version = rustup.full_version_str().await?;
    info!("version: {version:?}");
    let version = version.to_rust_version()?;
    info!("parsed version: {version:?}");
    Ok::<_, Box<dyn std::error::Error>>(())
  }
  #[tokio::test]
  async fn rustup_version()
  -> std::result::Result<(), Box<dyn std::error::Error>>
  {
    env_logger::init();
    let rustup = Rustup::get_by_current_user().await?;
    let version = rustup.full_version_str().await?;
    let version = version.to_rust_version()?;
    info!("version: {version:?}");
    Ok::<_, Box<dyn std::error::Error>>(())
  }
}
