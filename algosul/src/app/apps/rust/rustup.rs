use std::{
    borrow::Cow,
    env,
    fmt::{Debug, Formatter},
    path::{Path, PathBuf},
    process::Stdio,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
        Mutex,
    },
};

use bytes::BytesMut;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use tokio::{io::AsyncRead, process::Command, sync::RwLock};

use crate::{
    app::{
        apps::{
            rust,
            rust::{
                utils,
                utils::{
                    dwld_rsinit_sh,
                    dwld_rsinit_sh_and_save_plus_x,
                    ToRustVersion,
                },
                Error,
                HostTriple,
                Profile,
                Toolchain,
            },
        },
        AppLicense,
        AppPath,
    },
    process::{Process, Status},
    utils::tokio::AsyncBufReadExt,
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
    type Error = Error;

    async fn name(&self) -> Cow<'_, str> { Cow::Borrowed("rustup") }

    async fn license(&self) -> rust::Result<Cow<'_, AppLicense>> {
        Ok(Cow::Owned(AppLicense::Or(
            Box::new(AppLicense::Text("Apache".to_string())),
            Box::new(AppLicense::Text("MIT".to_string())),
        )))
    }

    async fn description(&self) -> rust::Result<Cow<'_, str>> { todo!() }

    async fn documentation(&self) -> rust::Result<Cow<'_, str>> {
        Ok(Cow::Borrowed("https://rust-lang.github.io/rustup/"))
    }

    async fn homepage(&self) -> rust::Result<Cow<'_, str>> {
        Ok(Cow::Borrowed("https://rustup.rs"))
    }

    async fn repository(&self) -> rust::Result<Cow<'_, str>> {
        Ok(Cow::Borrowed("https://github.com/rust-lang/rustup/"))
    }

    async fn version(&self) -> rust::Result<Cow<'_, str>> {
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
    type Error = Error;

    async fn home_path(&self) -> rust::Result<Cow<'_, Path>> {
        Ok(Cow::Borrowed(self.home_path.as_path()))
    }

    async fn bin_path(&self) -> rust::Result<Cow<'_, Path>> {
        Ok(Cow::Owned(self.home_path.join("bin/rustup")))
    }
}
type OnOutputFn = Box<dyn Fn(&[u8], &[u8]) + Send + Sync>;
#[derive(Default)]
pub struct RustupInstaller {
    install_info: InstallInfo,
    status:       RwLock<Status>,
    on_stdout:    Mutex<Option<OnOutputFn>>,
    on_stderr:    Mutex<Option<OnOutputFn>>,
}
#[derive(Default, Debug)]
pub struct RustupReinstaller {
    reinstall_info: ReinstallInfo,
}
#[derive(Default, Debug)]
pub struct RustupRemover {
    remove_info: RemoveInfo,
}
#[derive(Default, Debug)]
pub struct RustupUpdater {
    update_info: UpdateInfo,
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
    type Count = usize;
    type Error = Error;
    type Output = Rustup;

    async fn run(&self) -> rust::Result<Self::Output> {
        info!("Installing Rustup with info: {self:?}");
        let path = env::temp_dir().join("rustup-init.sh");
        dwld_rsinit_sh_and_save_plus_x(&path).await?;
        info!("Downloaded {path:?} successfully");
        let mut child = Command::new(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .args(self.install_info.to_args())
            .spawn()?;
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();
        debug!("Spawned command successfully, now reading stdout and stderr");
        let on_stdout = self.on_stdout.try_lock().unwrap().take();
        let on_stderr = self.on_stderr.try_lock().unwrap().take();
        fn reader(
            f: Option<OnOutputFn>,
            mut read: impl AsyncRead + Unpin + Send + 'static,
        ) -> tokio::task::JoinHandle<rust::Result<BytesMut>> {
            if let Some(f) = f {
                tokio::task::spawn(async move {
                    read.read_to_end_with(|buf, new| f(buf, new)).await
                })
            } else {
                tokio::task::spawn(async move {
                    read.read_to_end_with(|_, _| {}).await
                })
            }
        }
        let stdout_task = reader(on_stdout, stdout);
        let stderr_task = reader(on_stderr, stderr);
        let (stdout_buf, stderr_buf) =
            tokio::try_join!(stdout_task, stderr_task)?;
        debug!("Read stdout and stderr successfully");
        let exit_status = child.wait().await?;
        info!("Command finished with exit status: {exit_status}");
        let (stdout_buf, stderr_buf) = (
            String::from_utf8_lossy(&stdout_buf?).into_owned(),
            String::from_utf8_lossy(&stderr_buf?).into_owned(),
        );
        info!("Command finished with stdout: \n{stdout_buf}");
        info!("Command finished with stderr: \n{stderr_buf}");
        if exit_status.success() {
            Ok(Rustup { home_path: utils::get_home_dir()?.join(".cargo/") })
        } else {
            Err(Error::Failed {
                exit_status,
                stdin: "".into(),
                stdout: stdout_buf.into(),
                stderr: stderr_buf.into(),
            })
        }
    }

    async fn overall_progress(&self) -> rust::Result<Self::Count> { todo!() }

    async fn progress(&self) -> rust::Result<Self::Count> { todo!() }

    async fn status(&self) -> rust::Result<Status> {
        Ok(*self.status.read().await)
    }

    fn try_status(&self) -> rust::Result<Status> {
        Ok(*self.status.try_read()?)
    }
}
impl Process for RustupReinstaller {
    type Count = usize;
    type Error = Error;
    type Output = Rustup;

    async fn run(&self) -> rust::Result<Self::Output> { todo!() }

    async fn overall_progress(&self) -> rust::Result<Self::Count> { todo!() }

    async fn progress(&self) -> rust::Result<Self::Count> { todo!() }

    async fn status(&self) -> rust::Result<Status> { todo!() }

    fn try_status(&self) -> rust::Result<Status> { todo!() }
}
impl Process for RustupRemover {
    type Count = usize;
    type Error = Error;
    type Output = ();

    async fn run(&self) -> rust::Result<Self::Output> { todo!() }

    async fn overall_progress(&self) -> rust::Result<Self::Count> { todo!() }

    async fn progress(&self) -> rust::Result<Self::Count> { todo!() }

    async fn status(&self) -> rust::Result<Status> { todo!() }

    fn try_status(&self) -> rust::Result<Status> { todo!() }
}
impl Process for RustupUpdater {
    type Count = usize;
    type Error = Error;
    type Output = ();

    async fn run(&self) -> rust::Result<Self::Output> { todo!() }

    async fn overall_progress(&self) -> rust::Result<Self::Count> { todo!() }

    async fn progress(&self) -> rust::Result<Self::Count> { todo!() }

    async fn status(&self) -> rust::Result<Status> { todo!() }

    fn try_status(&self) -> rust::Result<Status> { todo!() }
}
impl crate::app::AppOper for Rustup {
    type Error = Error;
    type InstallInfo = InstallInfo;
    type Installer = RustupInstaller;
    type ReinstallInfo = ();
    type Reinstaller = RustupReinstaller;
    type RemoveInfo = ();
    type Remover = RustupRemover;
    type UpdateInfo = ();
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

    #[cfg(unix)]
    async fn install(info: Self::InstallInfo) -> rust::Result<Self> {
        use std::{ffi::OsString, os::unix::ffi::OsStringExt, process::Stdio};

        use log::{debug, info, warn};
        use tokio::{
            io::{AsyncBufReadExt, AsyncRead, AsyncWriteExt, BufReader},
            process::Command,
        };

        use crate::app::apps::rust::utils;
        debug!("Installing Rustup with info: {info:?}");
        let shell = dwld_rsinit_sh().await?;
        info!("Downloaded rustup-init.sh successfully");
        let args = info.to_args();
        debug!("Shell args: {args:?}");
        let mut command = Command::new("/bin/sh")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .arg("-s")
            .arg("--")
            .args(args)
            .spawn()?;
        debug!("Command spawned successfully");
        command.stdin.as_mut().unwrap().write_all(shell.as_bytes()).await?;
        debug!("write shell to stdin successfully");
        let stdout = command.stdout.take().unwrap();
        let stderr = command.stderr.take().unwrap();
        async fn read_and_print(
            prefix: &str, reader: impl AsyncRead + Unpin + Send + 'static,
        ) -> Vec<u8> {
            let mut lines = BufReader::new(reader).lines();
            let mut out = Vec::new();
            // TODO: 不会结束
            while let Ok(Some(line)) = lines.next_line().await {
                out.write_all(line.as_ref()).await.unwrap();
                info!("[{prefix}] {line}");
            }
            out
        }
        let stdout_task = tokio::spawn(read_and_print("stdout", stdout));
        let stderr_task = tokio::spawn(read_and_print("stderr", stderr));
        let (stdout_buf, stderr_buf) =
            tokio::try_join!(stdout_task, stderr_task,)?;
        let (stdout_buf, stderr_buf) = (
            OsString::from_vec(stdout_buf).to_string_lossy().into_owned(),
            OsString::from_vec(stderr_buf).to_string_lossy().into_owned(),
        );
        // TODO: 不会执行
        let exit_status = command.wait().await?;
        info!("Command finished with exit status: {exit_status}");
        info!("Command finished with stdout: \n{stdout_buf}");
        warn!("Command finished with stderr: \n{stderr_buf}");
        if exit_status.success() {
            Ok(Self {
                // ~/.cargo
                home_path: utils::get_home_dir()?.join(".cargo"),
            })
        } else {
            Err(Error::Failed {
                exit_status,
                stdin: shell.into(),
                stdout: stdout_buf.into(),
                stderr: stderr_buf.into(),
            })
        }
    }

    #[cfg(windows)]
    async fn install(info: Self::InstallInfo) -> rust::Result<Self> { todo!() }

    async fn reinstall(self, _info: Self::ReinstallInfo) -> rust::Result<Self> {
        todo!()
    }

    async fn remove(self, _info: Self::RemoveInfo) -> rust::Result<()> {
        todo!()
    }

    async fn update(self, _info: Self::UpdateInfo) -> rust::Result<Self> {
        todo!()
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
    pub async fn get_by_current_user() -> rust::Result<Self> {
        Ok(Self { home_path: utils::get_home_dir()?.join(".cargo/") })
    }

    pub async fn to_command(&self) -> rust::Result<Command> {
        Ok(Command::new(self.bin_path().await?.as_ref()))
    }

    pub async fn full_version_str(&self) -> rust::Result<String> {
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
            .field("status", &self.status)
            .finish()
    }
}
