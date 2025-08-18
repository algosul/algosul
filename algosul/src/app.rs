use std::{
  borrow::Cow,
  fmt::{Display, Formatter},
  path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use tokio::process::Command;

#[cfg(feature = "app-apps")]
pub mod apps;
/// application license
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
pub enum AppLicense
{
  #[default]
  Unknown,
  /// e.g. GPL-3.0-only
  Text(String),
  File(PathBuf),
  Or(Box<AppLicense>, Box<AppLicense>),
}
/// application information
pub trait AppInfo
{
  type Result<T>;
  fn name(&self) -> impl Future<Output = Cow<'_, str>> + Send;
  fn license(
    &self,
  ) -> impl Future<Output = Self::Result<Cow<'_, AppLicense>>> + Send;
  fn readme(&self) -> impl Future<Output = Self::Result<String>> + Send;
  fn readme_md(&self) -> impl Future<Output = Self::Result<String>> + Send;
  fn documentation(
    &self,
  ) -> impl Future<Output = Self::Result<Cow<'_, str>>> + Send;
  fn homepage(&self)
  -> impl Future<Output = Self::Result<Cow<'_, str>>> + Send;
  fn repository(
    &self,
  ) -> impl Future<Output = Self::Result<Cow<'_, str>>> + Send;
  fn version(&self) -> impl Future<Output = Self::Result<Cow<'_, str>>> + Send;
}
/// about the application paths
pub trait AppPath: AppInfo
{
  /// e.g. '~/.cargo/'
  fn home_path(
    &self,
  ) -> impl Future<Output = Self::Result<Cow<'_, Path>>> + Send;
  /// e.g. '~/.cargo/bin/rustup'
  fn bin_path(
    &self,
  ) -> impl Future<Output = Self::Result<Cow<'_, Path>>> + Send;
  fn to_command(&self) -> impl Future<Output = Self::Result<Command>> + Send;
}
/// application getter
pub trait AppGetter: Sized + AppPath
{
  fn get_by_current_user() -> impl Future<Output = Self::Result<Self>> + Send;
  fn get_by_all_user() -> impl Future<Output = Self::Result<Self>> + Send;
}
/// application operators
pub trait AppOper: Sized + AppInfo
{
  type Installer: crate::process::Process;
  type Reinstaller: crate::process::Process;
  type Remover: crate::process::Process;
  type Updater: crate::process::Process;
  fn installer() -> impl Future<Output = Self::Result<Self::Installer>> + Send;
  fn reinstaller(
    self,
  ) -> impl Future<Output = Self::Result<Self::Reinstaller>> + Send;
  fn remover(self) -> impl Future<Output = Self::Result<Self::Remover>> + Send;
  fn updater(self) -> impl Future<Output = Self::Result<Self::Updater>> + Send;
}
impl Display for AppLicense
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
  {
    match self
    {
      AppLicense::Unknown => write!(f, "Unknown"),
      AppLicense::Text(s) => write!(f, "{s}"),
      AppLicense::Or(a, b) => write!(f, "{a} or {b}"),
      AppLicense::File(path) => write!(f, "{path:?}"),
    }
  }
}
