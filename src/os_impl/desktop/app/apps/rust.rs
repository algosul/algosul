use std::{
    borrow::Cow,
    env,
    ffi::{OsStr, OsString},
    fmt::{Display, Formatter},
    fs::{create_dir, exists, File},
    io::{stderr, stdout, Write},
    os::windows::ffi::OsStringExt,
    path::{Path, PathBuf},
    process::{ExitStatus, Stdio},
    str::FromStr,
};

use log::{info, trace, warn};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncRead, AsyncReadExt},
    process::Command,
};

use crate::app::AppLicense;
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
pub struct Rustup {
    home_path: PathBuf,
}
#[derive(Debug)]
pub enum Error {
    Unsupported(Cow<'static, str>),
    IOError(std::io::Error),
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
        }
    }
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
#[derive(
    Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize,
)]
pub struct InstallCustomInfo {
    pub default_host_triple:  HostTriple,
    pub default_toolchain:    Toolchain,
    pub profile:              Profile,
    pub modify_path_variable: bool,
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
pub enum InstallInfo {
    #[default]
    Default,
    Custom(InstallCustomInfo),
}
#[cfg(unix)]
async fn download_rustup_init_sh() -> Result<()> {
    let url = "https://sh.rustup.rs";
    let content = reqwest::get(url)
        .await
        .map_err(Error::RequestError)?
        .text()
        .await
        .map_err(Error::RequestError)?;
    if !exists("./cache").map_err(Error::IOError)? {
        create_dir("cache").map_err(Error::IOError)?;
    }
    let mut file =
        File::create("./cache/rustup-init.sh").map_err(Error::IOError)?;
    file.write_all(content.as_bytes()).map_err(Error::IOError)?;
    Ok(())
}
#[cfg(windows)]
async fn download_rustup_init_exe() -> Result<()> {
    let url = format!("https://win.rustup.rs/{}", env::consts::ARCH);
    info!("url: {url}");
    if !exists("./cache").map_err(Error::IOError)? {
        create_dir("cache").map_err(Error::IOError)?;
    }
    let client = Client::new();
    let response = client.get(url).send().await.map_err(Error::RequestError)?;
    if response.status().is_success() {
        let mut file =
            File::create("./cache/rustup-init.exe").map_err(Error::IOError)?;
        let bytes = response.bytes().await.map_err(Error::RequestError)?;
        file.write_all(bytes.as_ref()).map_err(Error::IOError)?;
        info!("Download OK");
    } else {
        panic!("Download failed, Code: {}", response.status());
    }
    Ok(())
}
#[cfg(unix)]
async fn chmod_rustup_init_sh() -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let path = Path::new("./cache/rustup-init.sh");
    let metadata = metadata(path).map_err(Error::IOError)?;
    let mut permissions = metadata.permissions();
    permissions.set_mode(permissions.mode() | 0o100);
    std::fs::set_permissions(path, permissions).map_err(Error::IOError)?;
    Ok(())
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
impl crate::app::AppInfo for Rustup {
    type Error = Error;

    async fn name(&self) -> Cow<'_, str> { Cow::Borrowed("rustup") }

    async fn license(&self) -> Result<Cow<'_, AppLicense>> {
        Ok(Cow::Owned(AppLicense::Or(
            Box::new(AppLicense::Text("Apache".to_string())),
            Box::new(AppLicense::Text("MIT".to_string())),
        )))
    }

    async fn description(&self) -> Result<Cow<'_, str>> { todo!() }

    async fn documentation(&self) -> Result<Cow<'_, str>> {
        Ok(Cow::Borrowed("https://rust-lang.github.io/rustup/"))
    }

    async fn homepage(&self) -> Result<Cow<'_, str>> {
        Ok(Cow::Borrowed("https://rustup.rs"))
    }

    async fn repository(&self) -> Result<Cow<'_, str>> {
        Ok(Cow::Borrowed("https://github.com/rust-lang/rustup/"))
    }

    async fn version(&self) -> Result<Cow<'_, str>> { todo!() }
}
impl crate::app::AppPath for Rustup {
    type Error = Error;

    async fn home_path(&self) -> Result<Cow<'_, Path>> {
        Ok(Cow::Borrowed(self.home_path.as_path()))
    }

    async fn bin_path(&self) -> Result<Cow<'_, Path>> {
        Ok(Cow::Owned(self.home_path.join("bin")))
    }
}
impl crate::app::AppOper for Rustup {
    type Error = Error;
    type InstallInfo = InstallInfo;
    type ReinstallInfo = ();
    type RemoveInfo = ();
    type UpdateInfo = ();

    async fn install(info: Self::InstallInfo) -> Result<Self> {
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStringExt;
            trace!("Installing Rustup with info: {info:?}");
            download_rustup_init_sh().await?;
            trace!("Downloaded rustup-init.sh successfully");
            chmod_rustup_init_sh().await?;
            trace!("Chmod rustup-init.sh successfully");
            let shell: Cow<'static, str> = match info {
                InstallInfo::Default => "./cache/rustup-init.sh -y".into(),
                InstallInfo::Custom(InstallCustomInfo {
                    default_host_triple,
                    default_toolchain,
                    profile,
                    modify_path_variable,
                }) => format!(
                    "./cache/rustup-init.sh -y --default-host-triple='{}' \
                     --default-toolchain='{}' --profile='{}'{}",
                    default_host_triple,
                    default_toolchain,
                    profile,
                    if modify_path_variable { " --modify-path" } else { "" }
                )
                .into(),
            };
            trace!("Shell command to execute: {shell}");
            let mut command = Command::new("/usr/bin/sh")
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .arg("-c")
                .arg(shell.as_ref())
                .spawn()
                .map_err(Error::IOError)?;
            trace!("Command spawned successfully");
            let (mut stdout, mut stderr) = (
                command.stdout.take().ok_or(Error::InnerError(
                    "Command 'sh': stdout is not available".into(),
                ))?,
                command.stderr.take().ok_or(Error::InnerError(
                    "Command 'sh': stderr is not available".into(),
                ))?,
            );
            let exit_status = command.wait().await.map_err(Error::IOError)?;
            trace!("Command finished with exit status: {exit_status}");
            let (mut stdout_buf, mut stderr_buf) = (Vec::new(), Vec::new());
            stdout
                .read_to_end(&mut stdout_buf)
                .await
                .map_err(Error::IOError)?;
            stderr
                .read_to_end(&mut stderr_buf)
                .await
                .map_err(Error::IOError)?;
            let (stdout_buf, stderr_buf) = (
                OsString::from_vec(stdout_buf).to_string_lossy().into_owned(),
                OsString::from_vec(stderr_buf).to_string_lossy().into_owned(),
            );
            info!("Command finished with stdout: \n{stdout_buf}");
            warn!("Command finished with stderr: \n{stderr_buf}");
            if exit_status.success() {
                Ok(Self {
                    // ~/.cargo
                    home_path: std::env::home_dir()
                        .ok_or(Error::FailedToGetHomeDir)?
                        .join(".cargo"),
                })
            } else {
                Err(Error::Failed {
                    exit_status,
                    stdin: "".into(),
                    stdout: stdout_buf.into(),
                    stderr: stderr_buf.into(),
                })
            }
        }
        #[cfg(windows)]
        {
            // https://win.rustup.rs/x86_64
            trace!("Installing Rustup with info: {info:?}");
            // download_rustup_init_exe().await?;
            // trace!("Downloaded rustup-init.exe successfully");
            let mut command = Command::new("./cache/rustup-init.exe");
            let command =
                command.stdin(Stdio::null()).stdout(stdout()).stderr(stderr());
            let mut command = match info {
                InstallInfo::Default => command.arg("-y"),
                InstallInfo::Custom(InstallCustomInfo {
                    default_host_triple,
                    default_toolchain,
                    profile,
                    modify_path_variable,
                }) => {
                    let command = command
                        .arg("-y")
                        // .arg(format!("--default-host={default_host_triple}"))
                        .arg(format!("--default-toolchain={default_toolchain}"))
                        .arg(format!("--profile={profile}"));
                    if modify_path_variable {
                        command
                    } else {
                        command.arg("--no-modify-path")
                    }
                }
            }
            .spawn()
            .map_err(Error::IOError)?;
            trace!("Command spawned successfully");
            // let (mut stdout, mut stderr) = (
            //     command
            //         .stdout
            //         .take()
            //         .ok_or(Error::InnerError("Command 'rustup-init': stdout
            // is not available".into()))?,     command
            //         .stderr
            //         .take()
            //         .ok_or(Error::InnerError("Command 'rustup-init': stderr
            // is not available".into()))?, );
            let exit_status = command.wait().await.map_err(Error::IOError)?;
            trace!("Command finished with exit status: {exit_status}");
            let (mut stdout_buf, mut stderr_buf) = (Vec::new(), Vec::new());
            // stdout.read_to_end(&mut
            // stdout_buf).await.map_err(Error::IOError)?;
            // stderr.read_to_end(&mut
            // stderr_buf).await.map_err(Error::IOError)?;
            let (stdout_buf, stderr_buf) = (
                unsafe { OsString::from_encoded_bytes_unchecked(stdout_buf) }
                    .to_string_lossy()
                    .into_owned(),
                unsafe { OsString::from_encoded_bytes_unchecked(stderr_buf) }
                    .to_string_lossy()
                    .into_owned(),
            );
            info!("Command finished with stdout: \n{stdout_buf}");
            warn!("Command finished with stderr: \n{stderr_buf}");
            if exit_status.success() {
                Ok(Self {
                    // ~/.cargo
                    home_path: std::env::home_dir()
                        .ok_or(Error::FailedToGetHomeDir)?
                        .join(".cargo"),
                })
            } else {
                Err(Error::Failed {
                    exit_status,
                    stdin: "".into(),
                    stdout: stdout_buf.into(),
                    stderr: stderr_buf.into(),
                })
            }
        }
        #[cfg(all(not(windows), not(unix)))]
        {
            Err(Error::Unsupported(
                format!("unsupported platform '{}'", std::env::consts::OS)
                    .into(),
            ))
        }
    }

    async fn reinstall(self, _info: Self::ReinstallInfo) -> Result<Self> {
        todo!()
    }

    async fn remove(self, _info: Self::RemoveInfo) -> Result<()> { todo!() }

    async fn update(self, _info: Self::UpdateInfo) -> Result<Self> { todo!() }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::AppOper;
    #[tokio::test]
    #[ignore]
    async fn install_rustup()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        env_logger::init();
        Rustup::install(InstallInfo::Default).await?;
        Ok::<_, Box<dyn std::error::Error>>(())
    }
}
