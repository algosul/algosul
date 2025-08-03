use std::{
  borrow::Cow,
  env,
  fmt::{Debug, Formatter},
  path::{Path, PathBuf},
  process::{ExitStatus, Stdio},
};

use log::{debug, info};
use serde::{Deserialize, Serialize};
use tokio::process::Command;

use super::{
  utils,
  utils::{dwld_rsinit_sh_and_save_plus_x, ToRustVersion},
  HostTriple,
  Profile,
  Toolchain,
};
use crate::{
  app::{AppLicense, AppPath},
  process::Process,
  utils::tokio::TokioChildExt,
};
#[derive(
  Default, Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct Rustup
{
  home_path: PathBuf,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct InstallCustomInfo
{
  pub default_host_triple:  HostTriple,
  pub default_toolchain:    Toolchain,
  pub profile:              Profile,
  pub modify_path_variable: bool,
}
#[derive(
  Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Default,
)]
pub enum InstallInfo
{
  #[default]
  Default,
  Custom(InstallCustomInfo),
}
#[derive(
  Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Default,
)]
pub enum ReinstallInfo
{
  #[default]
  Default,
  Custom(InstallCustomInfo),
}
#[derive(
  Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Default,
)]
pub enum RemoveInfo
{
  #[default]
  Default,
}
#[derive(
  Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Default,
)]
pub enum UpdateInfo
{
  #[default]
  Default,
}
impl crate::app::AppInfo for Rustup
{
  type Error = super::Error;

  async fn name(&self) -> Cow<'_, str>
  {
    Cow::Borrowed("rustup")
  }

  async fn license(&self) -> super::Result<Cow<'_, AppLicense>>
  {
    Ok(Cow::Owned(AppLicense::Or(
      Box::new(AppLicense::Text("Apache".to_string())),
      Box::new(AppLicense::Text("MIT".to_string())),
    )))
  }

  async fn description(&self) -> super::Result<Cow<'_, str>>
  {
    todo!()
  }

  async fn documentation(&self) -> super::Result<Cow<'_, str>>
  {
    Ok(Cow::Borrowed("https://rust-lang.github.io/rustup/"))
  }

  async fn homepage(&self) -> super::Result<Cow<'_, str>>
  {
    Ok(Cow::Borrowed("https://rustup.rs"))
  }

  async fn repository(&self) -> super::Result<Cow<'_, str>>
  {
    Ok(Cow::Borrowed("https://github.com/rust-lang/rustup/"))
  }

  async fn version(&self) -> super::Result<Cow<'_, str>>
  {
    Ok(Cow::Owned(
      self.full_version_str().await?.to_rust_version()?.version.to_owned(),
    ))
  }
}
impl AppPath for Rustup
{
  type Error = super::Error;

  async fn home_path(&self) -> super::Result<Cow<'_, Path>>
  {
    Ok(Cow::Borrowed(self.home_path.as_path()))
  }

  async fn bin_path(&self) -> super::Result<Cow<'_, Path>>
  {
    Ok(Cow::Owned(self.home_path.join("bin/rustup")))
  }
}
type OnOutputFn = Box<dyn Fn(&[u8], &[u8]) + Send + Sync>;
type StatusObserver<T> = crate::process::StatusObserver<T, super::Error>;
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub enum RustupInstallStatus
{
  #[default]
  Initial,
  Downloading,
  Downloaded,
  Running,
  OnExitStatus(ExitStatus),
  Success,
  Failed
  {
    exit_status: ExitStatus,
    stdout:      String,
    stderr:      String,
  },
}
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub enum RustupReinstallStatus
{
  #[default]
  Initial,
  RemoveStatus(RustupInstallStatus),
  InstallStatus(RustupInstallStatus),
  Success,
  Failed
  {
    exit_status: ExitStatus,
    stdout:      String,
    stderr:      String,
  },
}
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub enum RustupRemoveStatus
{
  #[default]
  Initial,
  Running,
  OnExitStatus(ExitStatus),
  Success,
  Failed
  {
    exit_status: ExitStatus,
    stdout:      String,
    stderr:      String,
  },
}
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub enum RustupUpdateStatus
{
  #[default]
  Initial,
  Running,
  OnExitStatus(ExitStatus),
  Success,
  Failed
  {
    exit_status: ExitStatus,
    stdout:      String,
    stderr:      String,
  },
}
#[derive(Default)]
pub struct RustupInstaller
{
  install_info: InstallInfo,
  status:       StatusObserver<RustupInstallStatus>,
  on_stdout:    Option<OnOutputFn>,
  on_stderr:    Option<OnOutputFn>,
}
pub struct RustupReinstaller
{
  rustup:         Rustup,
  reinstall_info: ReinstallInfo,
  status:         StatusObserver<RustupReinstallStatus>,
  on_stdout:      Option<OnOutputFn>,
  on_stderr:      Option<OnOutputFn>,
}
pub struct RustupRemover
{
  rustup:      Rustup,
  remove_info: RemoveInfo,
  status:      StatusObserver<RustupRemoveStatus>,
  on_stdout:   Option<OnOutputFn>,
  on_stderr:   Option<OnOutputFn>,
}
pub struct RustupUpdater
{
  rustup:      Rustup,
  update_info: UpdateInfo,
  status:      StatusObserver<RustupUpdateStatus>,
  on_stdout:   Option<OnOutputFn>,
  on_stderr:   Option<OnOutputFn>,
}
impl RustupInstaller
{
  pub fn set_install_info(&mut self, info: InstallInfo) -> &mut Self
  {
    self.install_info = info;
    self
  }

  pub fn install_info(&self) -> &InstallInfo
  {
    &self.install_info
  }

  pub fn on_stdout<F>(&mut self, f: F) -> &mut Self
  where F: Fn(&[u8], &[u8]) + Send + Sync + 'static
  {
    self.on_stdout = Some(Box::new(f));
    self
  }

  pub fn on_stderr<F>(&mut self, f: F) -> &mut Self
  where F: Fn(&[u8], &[u8]) + Send + Sync + 'static
  {
    self.on_stderr = Some(Box::new(f));
    self
  }
}
impl RustupReinstaller
{
  pub fn new(rustup: Rustup) -> Self
  {
    Self {
      rustup,
      reinstall_info: ReinstallInfo::default(),
      status: Default::default(),
      on_stdout: None,
      on_stderr: None,
    }
  }

  pub fn set_reinstall_info(&mut self, info: ReinstallInfo) -> &mut Self
  {
    self.reinstall_info = info;
    self
  }

  pub fn reinstall_info(&self) -> &ReinstallInfo
  {
    &self.reinstall_info
  }
}
impl RustupRemover
{
  pub fn new(rustup: Rustup) -> Self
  {
    Self {
      rustup,
      remove_info: RemoveInfo::default(),
      status: Default::default(),
      on_stdout: None,
      on_stderr: None,
    }
  }

  pub fn set_remove_info(&mut self, info: RemoveInfo) -> &mut Self
  {
    self.remove_info = info;
    self
  }

  pub fn remove_info(&self) -> &RemoveInfo
  {
    &self.remove_info
  }
}
impl RustupUpdater
{
  pub fn new(rustup: Rustup) -> Self
  {
    Self {
      rustup,
      update_info: UpdateInfo::default(),
      status: Default::default(),
      on_stdout: None,
      on_stderr: None,
    }
  }

  pub fn set_update_info(&mut self, info: UpdateInfo) -> &mut Self
  {
    self.update_info = info;
    self
  }

  pub fn update_info(&self) -> &UpdateInfo
  {
    &self.update_info
  }
}
impl Process for RustupInstaller
{
  type Error = super::Error;
  type Output = Rustup;
  type Status = RustupInstallStatus;

  async fn run(mut self) -> super::Result<Self::Output>
  {
    self.status.change(RustupInstallStatus::Initial)?;
    debug!("Installing Rustup with info: {self:?}");

    self.status.change(RustupInstallStatus::Downloading)?;
    let path = env::temp_dir().join("rustup-init.sh");
    dwld_rsinit_sh_and_save_plus_x(&path).await?;

    self.status.change(RustupInstallStatus::Downloaded)?;
    info!("{path:?} has been successfully downloaded or cached");

    self.status.change(RustupInstallStatus::Running)?;
    let mut child = Command::new(path)
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .args(self.install_info.to_args())
      .spawn()?;
    debug!("Spawned command successfully, now reading stdout and stderr");

    let (stdout_buf, stderr_buf) =
      child.read_out(self.on_stdout.take(), self.on_stderr.take()).await?;

    let exit_status = child.wait().await?;
    self.status.change(RustupInstallStatus::OnExitStatus(exit_status))?;
    info!("Command finished with exit status: {exit_status}");

    let (stdout_buf, stderr_buf) = (stdout_buf?, stderr_buf?);

    if exit_status.success()
    {
      self.status.change(RustupInstallStatus::Success)?;
      Ok(Rustup { home_path: utils::get_home_dir()?.join(".cargo/") })
    }
    else
    {
      self.status.change(RustupInstallStatus::Failed {
        exit_status,
        stdout: String::from_utf8_lossy_owned(stdout_buf.to_vec()),
        stderr: String::from_utf8_lossy_owned(stderr_buf.to_vec()),
      })?;
      Err(super::Error::Failed(
        "Rustup installation failed. For more, please use on_status_changed."
          .into(),
      ))
    }
  }

  fn on_status_changed(
    &mut self,
    f: impl Fn(&Self::Status) -> Result<(), Self::Error> + Send + 'static,
  ) -> Result<(), Self::Error>
  {
    self.status.on_changed(f);
    Ok(())
  }
}
impl Process for RustupReinstaller
{
  type Error = super::Error;
  type Output = Rustup;
  type Status = RustupReinstallStatus;

  async fn run(self) -> super::Result<Self::Output>
  {
    todo!()
  }

  fn on_status_changed(
    &mut self,
    f: impl Fn(&Self::Status) -> Result<(), Self::Error> + Send + 'static,
  ) -> Result<(), Self::Error>
  {
    self.status.on_changed(f);
    Ok(())
  }
}
impl Process for RustupRemover
{
  type Error = super::Error;
  type Output = ();
  type Status = RustupRemoveStatus;

  async fn run(mut self) -> super::Result<Self::Output>
  {
    self.status.change(RustupRemoveStatus::Initial)?;

    self.status.change(RustupRemoveStatus::Running)?;
    let mut child = self
      .rustup
      .to_command()
      .await?
      .arg("self")
      .arg("uninstall")
      .stdin(Stdio::null())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()?;
    debug!("Spawned command successfully, now reading stdout and stderr");

    let (stdout_buf, stderr_buf) =
      child.read_out(self.on_stdout.take(), self.on_stderr.take()).await?;

    let exit_status = child.wait().await?;
    self.status.change(RustupRemoveStatus::OnExitStatus(exit_status))?;
    info!("Command finished with exit status: {exit_status}");

    let (stdout_buf, stderr_buf) = (stdout_buf?, stderr_buf?);

    if exit_status.success()
    {
      self.status.change(RustupRemoveStatus::Success)?;
      Ok(())
    }
    else
    {
      self.status.change(RustupRemoveStatus::Failed {
        exit_status,
        stdout: String::from_utf8_lossy_owned(stdout_buf.to_vec()),
        stderr: String::from_utf8_lossy_owned(stderr_buf.to_vec()),
      })?;
      Err(super::Error::Failed(
        "Rustup installation failed. For more, please use on_status_changed."
          .into(),
      ))
    }
  }

  fn on_status_changed(
    &mut self,
    f: impl Fn(&Self::Status) -> Result<(), Self::Error> + Send + 'static,
  ) -> Result<(), Self::Error>
  {
    self.status.on_changed(f);
    Ok(())
  }
}
impl Process for RustupUpdater
{
  type Error = super::Error;
  type Output = ();
  type Status = RustupUpdateStatus;

  async fn run(self) -> super::Result<Self::Output>
  {
    todo!()
  }

  fn on_status_changed(
    &mut self,
    f: impl Fn(&Self::Status) -> Result<(), Self::Error> + Send + 'static,
  ) -> Result<(), Self::Error>
  {
    self.status.on_changed(f);
    Ok(())
  }
}
impl crate::app::AppOper for Rustup
{
  type Error = super::Error;
  type Installer = RustupInstaller;
  type Reinstaller = RustupReinstaller;
  type Remover = RustupRemover;
  type Updater = RustupUpdater;

  async fn installer() -> Result<Self::Installer, Self::Error>
  {
    Ok(Self::Installer::default())
  }

  async fn reinstaller(self) -> Result<Self::Reinstaller, Self::Error>
  {
    Ok(Self::Reinstaller::new(self))
  }

  async fn remover(self) -> Result<Self::Remover, Self::Error>
  {
    Ok(Self::Remover::new(self))
  }

  async fn updater(self) -> Result<Self::Updater, Self::Error>
  {
    Ok(Self::Updater::new(self))
  }
}
impl InstallInfo
{
  #[cfg(unix)]
  pub fn to_args(&self) -> Vec<String>
  {
    let mut args = vec!["-y".to_string()];
    match self
    {
      InstallInfo::Default =>
      {}
      InstallInfo::Custom(InstallCustomInfo {
        default_host_triple,
        default_toolchain,
        profile,
        modify_path_variable,
      }) =>
      {
        args.push(format!("--default-host-triple='{default_host_triple}'"));
        args.push(format!("--default-toolchain='{default_toolchain}'"));
        args.push(format!("--profile='{profile}'"));
        if *modify_path_variable
        {
          args.push(" --modify-path".to_string());
        }
      }
    };
    args
  }

  #[cfg(windows)]
  pub fn to_args(&self) -> Vec<String>
  {
    todo!()
  }
}
impl Default for InstallCustomInfo
{
  fn default() -> Self
  {
    Self {
      default_host_triple:  Default::default(),
      default_toolchain:    Default::default(),
      profile:              Default::default(),
      modify_path_variable: true,
    }
  }
}
impl Rustup
{
  pub async fn get_by_current_user() -> super::Result<Self>
  {
    Ok(Self { home_path: utils::get_home_dir()?.join(".cargo/") })
  }

  pub async fn to_command(&self) -> super::Result<Command>
  {
    Ok(Command::new(self.bin_path().await?.as_ref()))
  }

  pub async fn full_version_str(&self) -> super::Result<String>
  {
    let out = self
      .to_command()
      .await?
      .stdin(Stdio::null())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .arg("--version")
      .output()
      .await?
      .stdout;
    let version = String::from_utf8(out)?;
    Ok(version)
  }
}
impl Debug for RustupInstaller
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
  {
    f.debug_struct("RustupInstaller")
      .field("install_info", &self.install_info)
      .field("status", &self.status)
      .field("on_stdout", &self.on_stdout.is_some())
      .field("on_stderr", &self.on_stderr.is_some())
      .finish()
  }
}
impl Debug for RustupReinstaller
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
  {
    f.debug_struct("RustupReinstaller")
      .field("reinstall_info", &self.reinstall_info)
      .field("status", &self.status)
      .field("on_stdout", &self.on_stdout.is_some())
      .field("on_stderr", &self.on_stderr.is_some())
      .finish()
  }
}
impl Debug for RustupRemover
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
  {
    f.debug_struct("RustupRemover")
      .field("remove_info", &self.remove_info)
      .field("status", &self.status)
      .field("on_stdout", &self.on_stdout.is_some())
      .field("on_stderr", &self.on_stderr.is_some())
      .finish()
  }
}
impl Debug for RustupUpdater
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
  {
    f.debug_struct("RustupUpdater")
      .field("update_info", &self.update_info)
      .field("status", &self.status)
      .field("on_stdout", &self.on_stdout.is_some())
      .field("on_stderr", &self.on_stderr.is_some())
      .finish()
  }
}
