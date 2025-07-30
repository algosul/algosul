use std::{
    borrow::Cow,
    env,
    fmt::{Debug, Formatter},
    path::{Path, PathBuf},
    process::{ExitStatus, Stdio},
    sync::Mutex,
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
    utils::tokio::TokioReadTaskExt,
};
#[derive(
    Default, Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct Rustup {
    home_path: PathBuf,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct InstallCustomInfo {
    pub default_host_triple:  HostTriple,
    pub default_toolchain:    Toolchain,
    pub profile:              Profile,
    pub modify_path_variable: bool,
}
#[derive(
    Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Default,
)]
pub enum InstallInfo {
    #[default]
    Default,
    Custom(InstallCustomInfo),
}
#[derive(
    Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Default,
)]
pub enum ReinstallInfo {
    #[default]
    Default,
    Custom(InstallCustomInfo),
}
#[derive(
    Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Default,
)]
pub enum RemoveInfo {
    #[default]
    Default,
}
#[derive(
    Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Default,
)]
pub enum UpdateInfo {
    #[default]
    Default,
}
impl crate::app::AppInfo for Rustup {
    type Error = super::Error;

    async fn name(&self) -> Cow<'_, str> { Cow::Borrowed("rustup") }

    async fn license(&self) -> super::Result<Cow<'_, AppLicense>> {
        Ok(Cow::Owned(AppLicense::Or(
            Box::new(AppLicense::Text("Apache".to_string())),
            Box::new(AppLicense::Text("MIT".to_string())),
        )))
    }

    async fn description(&self) -> super::Result<Cow<'_, str>> { todo!() }

    async fn documentation(&self) -> super::Result<Cow<'_, str>> {
        Ok(Cow::Borrowed("https://rust-lang.github.io/rustup/"))
    }

    async fn homepage(&self) -> super::Result<Cow<'_, str>> {
        Ok(Cow::Borrowed("https://rustup.rs"))
    }

    async fn repository(&self) -> super::Result<Cow<'_, str>> {
        Ok(Cow::Borrowed("https://github.com/rust-lang/rustup/"))
    }

    async fn version(&self) -> super::Result<Cow<'_, str>> {
        Ok(Cow::Owned(
            self.full_version_str()
                .await?
                .to_rust_version()?
                .version
                .to_owned(),
        ))
    }
}
impl AppPath for Rustup {
    type Error = super::Error;

    async fn home_path(&self) -> super::Result<Cow<'_, Path>> {
        Ok(Cow::Borrowed(self.home_path.as_path()))
    }

    async fn bin_path(&self) -> super::Result<Cow<'_, Path>> {
        Ok(Cow::Owned(self.home_path.join("bin/rustup")))
    }
}
type OnOutputFn = Box<dyn Fn(&[u8], &[u8]) + Send + Sync>;
type OnStatusChangedFn<T> = Box<dyn Fn(&T) -> Result<(), super::Error> + Send>;
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RustupInstallStatus {
    Initial,
    Downloading,
    Downloaded,
    Running,
    OnExitStatus(ExitStatus),
    Success,
    Failed {
        exit_status: ExitStatus,
        stdout:      String,
        stderr:      String,
    },
}
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RustupReinstallStatus {
    Initial,
    RemoveStatus(RustupInstallStatus),
    InstallStatus(RustupInstallStatus),
    Success,
    Failed {
        exit_status: ExitStatus,
        stdout:      String,
        stderr:      String,
    },
}
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RustupRemoveStatus {
    Initial,
    Running,
    OnExitStatus(ExitStatus),
    Success,
    Failed {
        exit_status: ExitStatus,
        stdout:      String,
        stderr:      String,
    },
}
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RustupUpdateStatus {
    Initial,
    Running,
    OnExitStatus(ExitStatus),
    Success,
    Failed {
        exit_status: ExitStatus,
        stdout:      String,
        stderr:      String,
    },
}
#[derive(Default)]
pub struct RustupInstaller {
    install_info:      InstallInfo,
    on_status_changed: Mutex<Option<OnStatusChangedFn<RustupInstallStatus>>>,
    on_stdout:         Mutex<Option<OnOutputFn>>,
    on_stderr:         Mutex<Option<OnOutputFn>>,
}
#[derive(Default)]
pub struct RustupReinstaller {
    reinstall_info:    ReinstallInfo,
    on_status_changed: Mutex<Option<OnStatusChangedFn<RustupReinstallStatus>>>,
    on_stdout:         Mutex<Option<OnOutputFn>>,
    on_stderr:         Mutex<Option<OnOutputFn>>,
}
#[derive(Default)]
pub struct RustupRemover {
    remove_info:       RemoveInfo,
    on_status_changed: Mutex<Option<OnStatusChangedFn<RustupRemoveStatus>>>,
    on_stdout:         Mutex<Option<OnOutputFn>>,
    on_stderr:         Mutex<Option<OnOutputFn>>,
}
#[derive(Default)]
pub struct RustupUpdater {
    update_info:       UpdateInfo,
    on_status_changed: Mutex<Option<OnStatusChangedFn<RustupUpdateStatus>>>,
    on_stdout:         Mutex<Option<OnOutputFn>>,
    on_stderr:         Mutex<Option<OnOutputFn>>,
}
impl RustupInstaller {
    pub fn set_install_info(&mut self, info: InstallInfo) -> &mut Self {
        self.install_info = info;
        self
    }

    pub fn install_info(&self) -> &InstallInfo { &self.install_info }

    pub fn on_stdout<F>(&mut self, f: F) -> &mut Self
    where F: Fn(&[u8], &[u8]) + Send + Sync + 'static {
        *self.on_stdout.try_lock().unwrap() = Some(Box::new(f));
        self
    }

    pub fn on_stderr<F>(&mut self, f: F) -> &mut Self
    where F: Fn(&[u8], &[u8]) + Send + Sync + 'static {
        *self.on_stderr.try_lock().unwrap() = Some(Box::new(f));
        self
    }

    pub fn change_status(
        &self, status: RustupInstallStatus,
    ) -> Result<(), super::Error> {
        if let Some(f) = self.on_status_changed.try_lock().unwrap().as_ref() {
            f(&status)?;
        }
        Ok(())
    }
}
impl RustupReinstaller {
    pub fn set_reinstall_info(&mut self, info: ReinstallInfo) -> &mut Self {
        self.reinstall_info = info;
        self
    }

    pub fn reinstall_info(&self) -> &ReinstallInfo { &self.reinstall_info }
}
impl RustupRemover {
    pub fn set_remove_info(&mut self, info: RemoveInfo) -> &mut Self {
        self.remove_info = info;
        self
    }

    pub fn remove_info(&self) -> &RemoveInfo { &self.remove_info }
}
impl RustupUpdater {
    pub fn set_update_info(&mut self, info: UpdateInfo) -> &mut Self {
        self.update_info = info;
        self
    }

    pub fn update_info(&self) -> &UpdateInfo { &self.update_info }
}
impl Process for RustupInstaller {
    type Error = super::Error;
    type Output = Rustup;
    type Status = RustupInstallStatus;

    async fn run(&self) -> super::Result<Self::Output> {
        self.change_status(RustupInstallStatus::Initial)?;
        debug!("Installing Rustup with info: {self:?}");
        let path = env::temp_dir().join("rustup-init.sh");
        self.change_status(RustupInstallStatus::Downloading)?;
        dwld_rsinit_sh_and_save_plus_x(&path).await?;
        self.change_status(RustupInstallStatus::Downloaded)?;
        info!("{path:?} has been successfully downloaded or cached");
        let mut child = Command::new(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .args(self.install_info.to_args())
            .spawn()?;
        self.change_status(RustupInstallStatus::Running)?;
        debug!("Spawned command successfully, now reading stdout and stderr");
        let stdout_task = child
            .stdout
            .take()
            .unwrap()
            .spawn_read_opt(self.on_stdout.try_lock().unwrap().take());
        let stderr_task = child
            .stderr
            .take()
            .unwrap()
            .spawn_read_opt(self.on_stderr.try_lock().unwrap().take());
        let (stdout_buf, stderr_buf) =
            tokio::try_join!(stdout_task, stderr_task)?;
        debug!("Read stdout and stderr successfully");
        let exit_status = child.wait().await?;
        self.change_status(RustupInstallStatus::OnExitStatus(exit_status))?;
        info!("Command finished with exit status: {exit_status}");
        let (stdout_buf, stderr_buf) = (stdout_buf?, stderr_buf?);
        if exit_status.success() {
            self.change_status(RustupInstallStatus::Success)?;
            Ok(Rustup { home_path: utils::get_home_dir()?.join(".cargo/") })
        } else {
            self.change_status(RustupInstallStatus::Failed {
                exit_status,
                stdout: String::from_utf8_lossy_owned(stdout_buf.to_vec()),
                stderr: String::from_utf8_lossy_owned(stderr_buf.to_vec()),
            })?;
            Err(super::Error::Failed(
                "Rustup installation failed. For more, please use \
                 on_status_changed."
                    .into(),
            ))
        }
    }

    fn on_status_changed(
        &self,
        f: impl Fn(&Self::Status) -> Result<(), Self::Error> + Send + 'static,
    ) -> Result<(), Self::Error> {
        *self.on_status_changed.try_lock().unwrap() = Some(Box::new(f));
        Ok(())
    }
}
impl Process for RustupReinstaller {
    type Error = super::Error;
    type Output = Rustup;
    type Status = RustupReinstallStatus;

    async fn run(&self) -> super::Result<Self::Output> { todo!() }

    fn on_status_changed(
        &self,
        f: impl Fn(&Self::Status) -> Result<(), Self::Error> + Send + 'static,
    ) -> Result<(), Self::Error> {
        *self.on_status_changed.try_lock().unwrap() = Some(Box::new(f));
        Ok(())
    }
}
impl Process for RustupRemover {
    type Error = super::Error;
    type Output = ();
    type Status = RustupRemoveStatus;

    async fn run(&self) -> super::Result<Self::Output> { todo!() }

    fn on_status_changed(
        &self,
        f: impl Fn(&Self::Status) -> Result<(), Self::Error> + Send + 'static,
    ) -> Result<(), Self::Error> {
        *self.on_status_changed.try_lock().unwrap() = Some(Box::new(f));
        Ok(())
    }
}
impl Process for RustupUpdater {
    type Error = super::Error;
    type Output = ();
    type Status = RustupUpdateStatus;

    async fn run(&self) -> super::Result<Self::Output> { todo!() }

    fn on_status_changed(
        &self,
        f: impl Fn(&Self::Status) -> Result<(), Self::Error> + Send + 'static,
    ) -> Result<(), Self::Error> {
        *self.on_status_changed.try_lock().unwrap() = Some(Box::new(f));
        Ok(())
    }
}
impl crate::app::AppOper for Rustup {
    type Error = super::Error;
    type Installer = RustupInstaller;
    type Reinstaller = RustupReinstaller;
    type Remover = RustupRemover;
    type Updater = RustupUpdater;

    async fn installer() -> Result<Self::Installer, Self::Error> {
        Ok(Self::Installer::default())
    }

    async fn reinstaller(self) -> Result<Self::Reinstaller, Self::Error> {
        Ok(Self::Reinstaller::default())
    }

    async fn remover(self) -> Result<Self::Remover, Self::Error> {
        Ok(Self::Remover::default())
    }

    async fn updater(self) -> Result<Self::Updater, Self::Error> {
        Ok(Self::Updater::default())
    }
}
impl InstallInfo {
    #[cfg(unix)]
    pub fn to_args(&self) -> Vec<String> {
        let mut args = vec!["-y".to_string()];
        match self {
            InstallInfo::Default => {}
            InstallInfo::Custom(InstallCustomInfo {
                default_host_triple,
                default_toolchain,
                profile,
                modify_path_variable,
            }) => {
                args.push(format!(
                    "--default-host-triple='{default_host_triple}'"
                ));
                args.push(format!("--default-toolchain='{default_toolchain}'"));
                args.push(format!("--profile='{profile}'"));
                if *modify_path_variable {
                    args.push(" --modify-path".to_string());
                }
            }
        };
        args
    }

    #[cfg(windows)]
    pub fn to_args(&self) -> Vec<String> { todo!() }
}
impl Default for InstallCustomInfo {
    fn default() -> Self {
        Self {
            default_host_triple:  Default::default(),
            default_toolchain:    Default::default(),
            profile:              Default::default(),
            modify_path_variable: true,
        }
    }
}
impl Rustup {
    pub async fn get_by_current_user() -> super::Result<Self> {
        Ok(Self { home_path: utils::get_home_dir()?.join(".cargo/") })
    }

    pub async fn to_command(&self) -> super::Result<Command> {
        Ok(Command::new(self.bin_path().await?.as_ref()))
    }

    pub async fn full_version_str(&self) -> super::Result<String> {
        let version = String::from_utf8(
            self.to_command()
                .await?
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .arg("--version")
                .output()
                .await?
                .stdout,
        )?;
        Ok(version)
    }
}
impl Debug for RustupInstaller {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RustupInstaller")
            .field("install_info", &self.install_info)
            .field(
                "on_status_changed",
                &self.on_status_changed.lock().unwrap().is_some(),
            )
            .field("on_stdout", &self.on_stdout.lock().unwrap().is_some())
            .field("on_stderr", &self.on_stderr.lock().unwrap().is_some())
            .finish()
    }
}
impl Debug for RustupReinstaller {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RustupReinstaller")
            .field("reinstall_info", &self.reinstall_info)
            .field(
                "on_status_changed",
                &self.on_status_changed.lock().unwrap().is_some(),
            )
            .field("on_stdout", &self.on_stdout.lock().unwrap().is_some())
            .field("on_stderr", &self.on_stderr.lock().unwrap().is_some())
            .finish()
    }
}
impl Debug for RustupRemover {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RustupRemover")
            .field("remove_info", &self.remove_info)
            .field(
                "on_status_changed",
                &self.on_status_changed.lock().unwrap().is_some(),
            )
            .field("on_stdout", &self.on_stdout.lock().unwrap().is_some())
            .field("on_stderr", &self.on_stderr.lock().unwrap().is_some())
            .finish()
    }
}
impl Debug for RustupUpdater {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RustupUpdater")
            .field("update_info", &self.update_info)
            .field(
                "on_status_changed",
                &self.on_status_changed.lock().unwrap().is_some(),
            )
            .field("on_stdout", &self.on_stdout.lock().unwrap().is_some())
            .field("on_stderr", &self.on_stderr.lock().unwrap().is_some())
            .finish()
    }
}
