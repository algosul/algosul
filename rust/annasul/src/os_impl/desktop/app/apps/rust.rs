// Copyright (c) 2025 air (https://yuanair.github.io).
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, version 3 of the License only.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.
use crate::app::AppLicense;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::ffi::OsString;
use std::fmt::{Display, Formatter};
use std::io::{stderr, stdout};
#[cfg(unix)]
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::process::{ExitStatus, Stdio};
use tokio::process::Command;
use trauma::download::{Download, Status};
use trauma::downloader::DownloaderBuilder;

#[derive(Default, Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
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
        stdin: Cow<'static, str>,
        stdout: Cow<'static, str>,
        stderr: Cow<'static, str>,
    },
    FailedToGetHomeDir,
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Unsupported(info) => f.write_fmt(format_args!("Unsupported: {}", info)),
            Error::IOError(e) => f.write_fmt(format_args!("IO error: {}", e)),
            Error::TaskJoinError(e) => f.write_fmt(format_args!("Task join error: {}", e)),
            Error::InnerError(info) => f.write_fmt(format_args!("Inner error: {}", info)),
            Error::Failed {
                exit_status,
                stdin,
                stdout,
                stderr,
            } => f.write_fmt(format_args!(
                "Failed:\n - exit status: {}\n - stdin:\n{}\n\n - stdout:\n{}\n\n - stderr:\n{}",
                exit_status, stdin, stdout, stderr
            )),
            Error::FailedToGetHomeDir => f.write_fmt(format_args!("failed to get HOME dir")),
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
        }
    }
}
pub type Result<T> = std::result::Result<T, Error>;
#[derive(
    Default, Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize,
)]
pub enum Toolchain {
    #[default]
    Stable,
    Beta,
    Nightly,
    None,
}
#[derive(Default, Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub enum HostTriple {
    #[default]
    Host,
    /// e.g. x86_64-unknown-linux-gnu
    Target(String),
}
#[derive(
    Default, Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize,
)]
pub enum Profile {
    Minimal,
    #[default]
    Default,
    Complete,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct InstallCustomInfo {
    default_host_triple: HostTriple,
    default_toolchain: Toolchain,
    profile: Profile,
    modify_path_variable: bool,
}
#[derive(Default, Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub enum InstallInfo {
    #[default]
    Default,
    Custom(InstallCustomInfo),
}

async fn download_rustup_init_sh() -> Result<()> {
    let url = "https://sh.rustup.rs";
    let downloads = vec![
        Download::try_from(url)
            .map_err(|_| Error::InnerError(format!("url '{}' error", url).into()))?,
    ];
    let downloader = DownloaderBuilder::new()
        .directory(PathBuf::from("cache"))
        .build();
    for summary in downloader.download(&downloads).await {
        match summary.status() {
            Status::Success => {}
            Status::Fail(url) => Err(Error::InnerError(url.clone().into()))?,
            Status::NotStarted => Err(Error::InnerError("not start".into()))?,
            Status::Skipped(reason) => Err(Error::InnerError(reason.clone().into()))?,
        }
    }
    Ok::<_, Error>(())
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
impl Display for HostTriple {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            HostTriple::Host => f.write_str("host"),
            HostTriple::Target(target) => f.write_str(target),
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
impl Default for InstallCustomInfo {
    fn default() -> Self {
        Self {
            default_host_triple: Default::default(),
            default_toolchain: Default::default(),
            profile: Default::default(),
            modify_path_variable: true,
        }
    }
}
impl crate::app::AppInfo for Rustup {
    type Error = Error;
    async fn name(&self) -> Cow<'_, str> {
        Cow::Borrowed("rustup")
    }

    async fn license(&self) -> Result<Cow<'_, AppLicense>> {
        Ok(Cow::Owned(AppLicense::Or(
            Box::new(AppLicense::Text("Apache".to_string())),
            Box::new(AppLicense::Text("MIT".to_string())),
        )))
    }

    async fn description(&self) -> Result<Cow<'_, str>> {
        todo!()
    }

    async fn documentation(&self) -> Result<Cow<'_, str>> {
        Ok(Cow::Borrowed("https://rust-lang.github.io/rustup/"))
    }

    async fn homepage(&self) -> Result<Cow<'_, str>> {
        Ok(Cow::Borrowed("https://rustup.rs"))
    }

    async fn repository(&self) -> Result<Cow<'_, str>> {
        Ok(Cow::Borrowed("https://github.com/rust-lang/rustup/"))
    }

    async fn version(&self) -> Result<Cow<'_, str>> {
        todo!()
    }
}
impl crate::app::AppPath for Rustup {
    type Error = Error;
    async fn home_path(&self) -> Result<Cow<'_, Path>> {
        Ok(Cow::Borrowed(self.home_path.as_path()))
    }
    async fn bin_path(&self) -> Result<Cow<'_, Path>> {
        Ok(Cow::Owned(self.home_path.join("bin").into()))
    }
}
impl crate::app::AppOper for Rustup {
    type Error = Error;
    type InstallInfo = InstallInfo;
    type ReinstallInfo = ();
    type RemoveInfo = ();
    type UpdateInfo = ();
    async fn install(info: Self::InstallInfo) -> Result<Self> {
        if cfg!(unix) {
            download_rustup_init_sh().await?;
            let shell: Cow<'static, str> = match info {
                    InstallInfo::Default => "cat ./cache/rustup-init.sh | sh -s -- -y".into(),
                    InstallInfo::Custom(InstallCustomInfo {
                                            default_host_triple,
                                            default_toolchain,
                                            profile,
                                            modify_path_variable,
                                        }) => format!(
                        "cat ./cache/rustup-init.sh | sh -s -- -y --default-host-triple='{}' --default-toolchain='{}' --profile='{}'{}",
                        default_host_triple,
                        default_toolchain,
                        profile,
                        if modify_path_variable { " --modify-path" } else { "" }
                    )
                        .into(),
                };
            let mut command = Command::new("/usr/bin/sh")
                .stdin(Stdio::null())
                .stdout(stdout())
                .stderr(stderr())
                .arg("-c")
                .arg(shell.as_ref())
                .spawn()
                .map_err(Error::IOError)?;

            // let (mut stdout, mut stderr) = (
            //     command.stdout.take().ok_or(Error::InnerError(
            //         "Command 'sh': stdout is not available".into(),
            //     ))?,
            //     command.stderr.take().ok_or(Error::InnerError(
            //         "Command 'sh': stderr is not available".into(),
            //     ))?,
            // );

            let exit_status = command.wait().await.map_err(Error::IOError)?;

            let mut stdout_buf = Vec::new();
            // stdout
            //     .read_to_end(&mut stdout_buf)
            //     .await
            //     .map_err(Error::IOError)?;

            let mut stderr_buf = Vec::new();
            // stderr
            //     .read_to_end(&mut stderr_buf)
            //     .await
            //     .map_err(Error::IOError)?;
            if exit_status.success() {
                Ok(Self {
                    // ~/.config
                    home_path: std::env::home_dir()
                        .ok_or(Error::FailedToGetHomeDir)?
                        .join(".config"),
                })
            } else {
                Err(Error::Failed {
                    exit_status,
                    stdin: "".into(),
                    stdout: OsString::from_vec(stdout_buf)
                        .to_string_lossy()
                        .into_owned()
                        .into(),
                    stderr: OsString::from_vec(stderr_buf)
                        .to_string_lossy()
                        .into_owned()
                        .into(),
                })
            }
        } else if cfg!(windows) {
            todo!()
        } else {
            Err(Error::Unsupported(
                format!("unsupported platform '{}'", std::env::consts::OS).into(),
            ))
        }
    }

    async fn reinstall(self, _info: Self::ReinstallInfo) -> Result<Self> {
        todo!()
    }

    async fn remove(self, _info: Self::RemoveInfo) -> Result<()> {
        todo!()
    }

    async fn update(self, _info: Self::UpdateInfo) -> Result<Self> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::AppOper;
    #[tokio::test]
    async fn install_rustup() -> std::result::Result<(), Box<dyn std::error::Error>> {
        Rustup::install(InstallInfo::Default).await?;
        Ok::<_, Box<dyn std::error::Error>>(())
    }
}
