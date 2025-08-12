use std::{
  borrow::Cow,
  ffi::OsStr,
  path::{Path, PathBuf},
  sync::Arc,
};

use algosul_core::{args, cows};
use tokio::process::Command;

use crate::app::{apps::rust::utils, AppInfo, AppLicense, AppPath};

#[derive(Default, Debug, Clone, Eq, PartialEq, Hash)]
pub struct Cargo
{
  home_path: Arc<PathBuf>,
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum CargoCommandArgs
{
  #[default]
  Version,
  Metadata
  {
    format_version: Option<Cow<'static, str>>,
    no_deps:        bool,
    features:       Vec<Cow<'static, str>>,
  },
}
impl CargoCommandArgs
{
  pub fn into_args(self) -> Vec<Cow<'static, OsStr>>
  {
    let mut args = Vec::<Cow<'static, OsStr>>::new();
    match self
    {
      CargoCommandArgs::Version =>
      {
        args!(args:
          cows!["version"]
        );
      }
      CargoCommandArgs::Metadata { format_version, no_deps, features } =>
      {
        args!(args:
          cows!["metadata"];
          if no_deps => cows!["--no-deps"];
          features
            .into_iter()
            .flat_map(|x| cows!["-F", x]);
          if let Some(format_version) = format_version =>
            cows!["--format-version", format_version];
        );
      }
    }
    args
  }

  pub fn into_command(self, cargo_path: Cow<'static, OsStr>) -> Command
  {
    let mut command = Command::new(cargo_path);
    command.args(self.into_args());
    command
  }
}

impl Cargo
{
  pub fn as_home_path(&self) -> Arc<PathBuf>
  {
    self.home_path.clone()
  }
}
impl AppInfo for Cargo
{
  type Error = super::Error;

  async fn name(&self) -> Cow<'_, str>
  {
    Cow::Borrowed("cargo")
  }

  async fn license(&self) -> Result<Cow<'_, AppLicense>, Self::Error>
  {
    Ok(Cow::Owned(utils::rust_license()))
  }

  async fn readme(&self) -> Result<String, Self::Error>
  {
    todo!()
  }

  async fn readme_md(&self) -> Result<String, Self::Error>
  {
    todo!()
  }

  async fn documentation(&self) -> Result<Cow<'_, str>, Self::Error>
  {
    Ok(Cow::Borrowed("https://doc.rust-lang.org/cargo"))
  }

  async fn homepage(&self) -> Result<Cow<'_, str>, Self::Error>
  {
    self.repository().await
  }

  async fn repository(&self) -> Result<Cow<'_, str>, Self::Error>
  {
    Ok(Cow::Borrowed("https://github.com/rust-lang/cargo"))
  }

  async fn version(&self) -> Result<Cow<'_, str>, Self::Error>
  {
    todo!()
  }
}
impl AppPath for Cargo
{
  async fn home_path(&self) -> Result<Cow<'_, Path>, Self::Error>
  {
    Ok(Cow::Borrowed(self.home_path.as_ref()))
  }

  async fn bin_path(&self) -> Result<Cow<'_, Path>, Self::Error>
  {
    Ok(Cow::Owned(self.home_path.join("cargo")))
  }
}
impl utils::RustAppExt for Cargo
{
  fn new(home_path: Arc<PathBuf>) -> crate::app::apps::rust::Result<Self>
  {
    Ok(Self { home_path })
  }
}
