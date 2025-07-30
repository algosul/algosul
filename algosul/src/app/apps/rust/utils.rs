use std::{
    env,
    fmt::{Display, Formatter},
    fs::Permissions,
    io::Write,
    path::{Path, PathBuf},
    time::SystemTime,
};
pub fn get_home_dir() -> super::Result<PathBuf> {
    env::home_dir().ok_or(super::Error::FailedToGetHomeDir)
}
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct RustVersion<'a> {
    pub tool_name: &'a str,
    pub version:   &'a str,
    pub hash:      &'a str,
    pub date:      &'a str,
}
pub mod regexs {
    use std::sync::LazyLock;

    use regex::Regex;
    pub const TOOL_NAME: &str = r"(.+)";
    pub const VERSION: &str = r"([\w.\-]+)";
    pub const HASH: &str = r"([a-f0-9]+)";
    pub const DATE: &str = r"(\d{4}-\d{2}-\d{2})";
    pub static RUST_VERSION: LazyLock<Result<Regex, regex::Error>> =
        LazyLock::new(|| {
            Regex::new(&format!(
                r"{TOOL_NAME}\s+{VERSION}\s+\({HASH}\s+{DATE}\)"
            ))
        });
}
pub trait ToRustVersion {
    fn to_rust_version(&self) -> super::Result<RustVersion>;
}
impl ToRustVersion for String {
    fn to_rust_version(&'_ self) -> super::Result<RustVersion<'_>> {
        let captures = regexs::RUST_VERSION
            .clone()?
            .captures(self)
            .ok_or(super::Error::VersionStringNoMatch)?;
        Ok(RustVersion {
            tool_name: captures.get(1).unwrap().as_str(),
            version:   captures.get(2).unwrap().as_str(),
            hash:      captures.get(3).unwrap().as_str(),
            date:      captures.get(4).unwrap().as_str(),
        })
    }
}
impl Display for RustVersion<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let RustVersion { tool_name, version, hash, date } = self;
        f.write_fmt(format_args!("{tool_name} {version} ({hash} {date})"))
    }
}
pub async fn dwld_rsinit_sh() -> super::Result<String> {
    let url = "https://sh.rustup.rs";
    let content = reqwest::get(url).await?.text().await?;
    Ok(content)
}
pub async fn dwld_rsinit_sh_and_save_plus_x(path: &Path) -> super::Result<()> {
    if !path.exists()
        || !path.is_file()
        || (SystemTime::now()
            .duration_since(path.metadata()?.modified()?)?
            .as_secs()
            >= 60 * 24)
    {
        std::fs::write(path, dwld_rsinit_sh().await?)?;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, Permissions::from_mode(0o755))?;
    }
    Ok(())
}
pub async fn dwld_rsinit_exe(arch: &str) -> super::Result<Vec<u8>> {
    let url = format!("https://win.rustup.rs/{arch}");
    let response = reqwest::get(url).await?;
    response
        .bytes()
        .await
        .map(|bytes| bytes.to_vec())
        .map_err(super::Error::RequestError)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_to_rust_version() {
        let version = "rustup 1.28.2 (e4f3ad6f8 2025-04-28)".to_string();
        let version = version.to_rust_version().unwrap();
        assert_eq!(version, RustVersion {
            tool_name: "rustup",
            version:   "1.28.2",
            hash:      "e4f3ad6f8",
            date:      "2025-04-28",
        });
    }
}
