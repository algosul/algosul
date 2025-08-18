use std::{
  borrow::Cow,
  collections::BTreeSet,
  ffi::OsStr,
  fmt::{Display, Formatter},
  hash::{DefaultHasher, Hash, Hasher},
  path::{Path, PathBuf},
  process::Command,
  sync::Arc,
};

use algosul_core::{args, cows, utils::std::MapToArgs};
use log::info;

use crate::app::{
  apps::{rust, rust::utils},
  AppInfo,
  AppLicense,
  AppPath,
};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Rustc
{
  home_path: Arc<PathBuf>,
}
impl Rustc
{
  pub fn as_home_path(&self) -> Arc<PathBuf>
  {
    self.home_path.clone()
  }
}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct RustcCommandBuilder
{
  pub compile_flag: RustcCompileFlagForHash,
  pub output:       Option<Cow<'static, OsStr>>,
  pub output_dir:   Option<Cow<'static, OsStr>>,
}
#[derive(Default, Debug, Clone, Eq, PartialEq, Hash)]
pub struct RustcCompileFlagForHash
{
  pub source_path: Cow<'static, OsStr>,
  pub rustc_path:  Cow<'static, OsStr>,
  pub crate_type:  CrateType,
  pub crate_name:  Option<Cow<'static, str>>,
  pub edition:     Option<Cow<'static, str>>,
  pub target:      rust::HostTriple,
  pub features:    BTreeSet<Cow<'static, str>>,
  pub check_cfg:   Option<Cow<'static, str>>,
  pub codegen:     Vec<Cow<'static, str>>,
  pub link:        Vec<Cow<'static, str>>,
}
#[derive(Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum CrateType
{
  #[default]
  Binary,
  Library,
  RustLibrary,
  DynamicLibrary,
  CDynamicLibrary,
  StaticLibrary,
  ProcMacroLibrary,
}

pub const EDITION_2015: &str = "2015";
pub const EDITION_2018: &str = "2018";
pub const EDITION_2021: &str = "2021";
pub const EDITION_2024: &str = "2024";

impl AppInfo for Rustc
{
  type Result<T> = super::Result<T>;

  async fn name(&self) -> Cow<'_, str>
  {
    Cow::Borrowed("rustc")
  }

  async fn license(&self) -> Self::Result<Cow<'_, AppLicense>>
  {
    Ok(Cow::Owned(utils::rust_license()))
  }

  async fn readme(&self) -> Self::Result<String>
  {
    todo!()
  }

  async fn readme_md(&self) -> Self::Result<String>
  {
    todo!()
  }

  async fn documentation(&self) -> Self::Result<Cow<'_, str>>
  {
    Ok(Cow::Borrowed("https://doc.rust-lang.org/rustc"))
  }

  async fn homepage(&self) -> Self::Result<Cow<'_, str>>
  {
    self.repository().await
  }

  async fn repository(&self) -> Self::Result<Cow<'_, str>>
  {
    Ok(Cow::Borrowed("https://github.com/rust-lang/rustc"))
  }

  async fn version(&self) -> Self::Result<Cow<'_, str>>
  {
    todo!()
  }
}
impl AppPath for Rustc
{
  async fn home_path(&self) -> Self::Result<Cow<'_, Path>>
  {
    Ok(Cow::Borrowed(self.home_path.as_ref()))
  }

  async fn bin_path(&self) -> Self::Result<Cow<'_, Path>>
  {
    Ok(Cow::Owned(self.home_path.join("rustc")))
  }

  async fn to_command(&self) -> Self::Result<tokio::process::Command>
  {
    Ok(tokio::process::Command::new(self.bin_path().await?.as_ref()))
  }
}
impl utils::RustAppExt for Rustc
{
  fn new(home_path: Arc<PathBuf>) -> rust::Result<Self>
  {
    Ok(Self { home_path })
  }
}
impl RustcCompileFlagForHash
{
  pub fn rustc_path(&self) -> &OsStr
  {
    self.rustc_path.as_ref()
  }

  pub fn rustc_version(&self) -> String
  {
    let buffer = String::from_utf8(
      Command::new(self.rustc_path())
        .args(["--version", "--verbose"])
        .output()
        .unwrap()
        .stdout,
    )
    .unwrap();
    info!("{buffer}");
    buffer
  }

  pub fn metadata_hash(&self, hasher: &mut impl Hasher)
  {
    self.source_path.hash(hasher);
    self.crate_type.hash(hasher);
    self.crate_name.hash(hasher);
    self.edition.hash(hasher);
    self.target.hash(hasher);
    self.features.hash(hasher);
    self.codegen.hash(hasher);
    self.link.hash(hasher);
    self.rustc_version().hash(hasher);
  }

  pub fn default_metadata_hash(&self) -> u64
  {
    let mut default_hasher = DefaultHasher::new();
    self.metadata_hash(&mut default_hasher);
    default_hasher.finish()
  }

  pub fn into_args(self) -> (Vec<Cow<'static, OsStr>>, Cow<'static, OsStr>)
  {
    let mut args = Vec::<Cow<'static, OsStr>>::new();
    args!(args:
      if let Some(name) = self.crate_name => cows!["--crate-name", name];
      if let Some(edition) = self.edition => cows!["--edition", edition];
      cows!["--crate-type", self.crate_type.to_string()];
      if let Some(target) = self.target.to_string() =>
        cows!["--target", target];
      if let Some(check_cfg) = self.check_cfg =>
        cows!["--check-cfg", check_cfg];
      self.features.into_iter().flat_map(|x| {
        cows!["--cfg", format!("feature=\"{x}\"")]
      });
      self.codegen.into_iter().flat_map(|x| cows!["-C", x]);
      self.link.into_iter().flat_map(|x| cows!["-L", x]);
    );
    (args, self.source_path)
  }

  pub fn to_cargo_toml(&self) -> toml::Table
  {
    let crate_name = self.crate_name.clone();
    let edition = self.edition.clone();
    let mut package = toml::Table::new();
    if let Some(name) = crate_name
    {
      package.insert("name".to_string(), name.into_owned().into());
    }
    if let Some(edition) = edition
    {
      package.insert("edition".to_string(), edition.into_owned().into());
    }
    package.insert("version".to_string(), "0.1.0".into());
    package.insert("build".to_string(), "build.rs".into());
    let package = package.into();
    let mut table = toml::Table::new();
    table.insert("package".to_string(), package);
    table
  }
}

impl RustcCommandBuilder
{
  pub fn into_args(self) -> Vec<Cow<'static, OsStr>>
  {
    let mut args = Vec::<Cow<'static, OsStr>>::new();
    let metadata_hash =
      format!("{:016x}", self.compile_flag.default_metadata_hash());
    let (flags, source_path) = self.compile_flag.into_args();
    args!(args:
      flags;
      cows![
        "-C", format!("metadata={metadata_hash}"),
        "-C", format!("extra-filename={metadata_hash}")
      ];
      if let Some(output) = self.output =>
        cows![OsStr::new("-o"), output].map_to_args();
      if let Some(output_dir) = self.output_dir =>
        cows![OsStr::new("--out-dir"), output_dir].map_to_args();
      @push source_path;
    );
    args
  }

  pub fn into_command(self) -> Command
  {
    let mut command = Command::new(self.compile_flag.rustc_path());
    command.args(self.into_args());
    command
  }
}
impl Display for CrateType
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
  {
    match self
    {
      CrateType::Binary => f.write_str("bin"),
      CrateType::Library => f.write_str("lib"),
      CrateType::RustLibrary => f.write_str("rlib"),
      CrateType::DynamicLibrary => f.write_str("dylib"),
      CrateType::CDynamicLibrary => f.write_str("cdylib"),
      CrateType::StaticLibrary => f.write_str("staticlib"),
      CrateType::ProcMacroLibrary => f.write_str("proc-macro"),
    }
  }
}
