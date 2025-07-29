use std::{
    borrow::Cow,
    ffi::OsString,
    fmt::{Display, Formatter},
    io,
    path::{Path, PathBuf},
    process::{ExitStatus, Stdio},
    str::FromStr,
};

use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
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
impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self { Self::IOError(value) }
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
impl InstallInfo {
    pub fn to_args(&self) -> Vec<String> {
        let mut args = Vec::new();
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
}
#[cfg(unix)]
async fn download_rustup_init_sh() -> Result<String> {
    let url = "https://sh.rustup.rs";
    let content = reqwest::get(url)
        .await
        .map_err(Error::RequestError)?
        .text()
        .await
        .map_err(Error::RequestError)?;
    Ok(content)
}
#[cfg(windows)]
async fn download_rustup_init_exe() -> Result<()> {
    let url = format!("https://win.rustup.rs/{}", env::consts::ARCH);
    info!("url: {url}");
    if !exists("./cache")? {
        create_dir("cache")?;
    }
    let client = Client::new();
    let response = client.get(url).send().await.map_err(Error::RequestError)?;
    if response.status().is_success() {
        let mut file = File::create("./cache/rustup-init.exe")?;
        let bytes = response.bytes().await.map_err(Error::RequestError)?;
        file.write_all(bytes.as_ref())?;
        info!("Download OK");
    } else {
        panic!("Download failed, Code: {}", response.status());
    }
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
            debug!("Installing Rustup with info: {info:?}");
            let shell = download_rustup_init_sh().await?;
            info!("Downloaded rustup-init.sh successfully");
            let mut args = Vec::new();
            match info {
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
                    args.push(format!(
                        "--default-toolchain='{default_toolchain}'"
                    ));
                    args.push(format!("--profile='{profile}'"));
                    if modify_path_variable {
                        args.push(" --modify-path".to_string());
                    }
                }
            };
            debug!("Shell args: {args:?}");
            let mut command = Command::new("/bin/sh")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .arg("-s")
                .arg("--")
                .arg("-y")
                .args(args)
                .spawn()?;
            debug!("Command spawned successfully");
            command.stdin.as_mut().unwrap().write_all(shell.as_bytes()).await?;
            debug!("write shell to stdin successfully");
            let mut stdout = command.stdout.take().unwrap();
            let mut stderr = command.stderr.take().unwrap();
            let exit_status = command.wait().await?;
            info!("Command finished with exit status: {exit_status}");
            let (mut stdout_buf, mut stderr_buf) = (Vec::new(), Vec::new());
            stdout.read_to_end(&mut stdout_buf).await?;
            stderr.read_to_end(&mut stderr_buf).await?;
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
                    stdin: shell.into(),
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
            .spawn()?;
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
            let exit_status = command.wait().await?;
            trace!("Command finished with exit status: {exit_status}");
            let (mut stdout_buf, mut stderr_buf) = (Vec::new(), Vec::new());
            // stdout.read_to_end(&mut
            // stdout_buf).await?;
            // stderr.read_to_end(&mut
            // stderr_buf).await?;
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
