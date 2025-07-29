use std::{
    borrow::Cow,
    ffi::OsString,
    fmt::{Display, Formatter},
    io,
    path::{Path, PathBuf},
    process::{ExitStatus, Stdio},
    str::FromStr,
    string::FromUtf8Error,
};

use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use tokio::{
    io::{
        AsyncBufReadExt,
        AsyncRead,
        AsyncReadExt,
        AsyncWrite,
        AsyncWriteExt,
        BufReader,
        BufWriter,
    },
    process::{ChildStdout, Command},
};
use utils::ToRustVersion;

use crate::app::{AppLicense, AppPath};
pub mod utils;
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
}
pub type Result<T> = std::result::Result<T, Error>;
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
impl Rustup {
    pub async fn get_by_current_user() -> Result<Self> {
        Ok(Self { home_path: utils::get_home_dir()?.join(".cargo/") })
    }

    pub async fn to_command(&self) -> Result<Command> {
        Ok(Command::new(self.bin_path().await?.join("rustup")))
    }

    pub async fn full_version_str(&self) -> Result<String> {
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

    async fn version(&self) -> Result<Cow<'_, str>> {
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

    #[cfg(unix)]
    async fn install(info: Self::InstallInfo) -> Result<Self> {
        use std::os::unix::ffi::OsStringExt;
        debug!("Installing Rustup with info: {info:?}");
        let shell = utils::download_rustup_init_sh().await?;
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
            while let Ok(Some(line)) = lines.next_line().await {
                out.write_all(line.as_ref()).await.unwrap();
                info!("[{prefix}] {line}");
            }
            out
        }
        let stdout_task = tokio::spawn(read_and_print("stdout", stdout));
        let stderr_task = tokio::spawn(read_and_print("stderr", stderr));
        let (stdout_buf, stderr_buf) =
            tokio::try_join!(stdout_task, stderr_task,).unwrap();
        let (stdout_buf, stderr_buf) = (
            OsString::from_vec(stdout_buf).to_string_lossy().into_owned(),
            OsString::from_vec(stderr_buf).to_string_lossy().into_owned(),
        );
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
    async fn install(info: Self::InstallInfo) -> Result<Self> { todo!() }

    async fn reinstall(self, _info: Self::ReinstallInfo) -> Result<Self> {
        todo!()
    }

    async fn remove(self, _info: Self::RemoveInfo) -> Result<()> { todo!() }

    async fn update(self, _info: Self::UpdateInfo) -> Result<Self> { todo!() }
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
        }
    }
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
#[cfg(test)]
mod tests {
    use utils::ToRustVersion;

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
