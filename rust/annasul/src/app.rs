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

//!
//! application operation
//!

use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

#[cfg(any(doc, feature = "app-apps"))]
pub mod apps;
/// application license
#[derive(Default, Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub enum AppLicense {
    #[default]
    Unknown,
    /// e.g. GPL-3.0-only
    Text(String),
    File(PathBuf),
    Or(Box<AppLicense>, Box<AppLicense>),
}
///
/// application information
///
pub trait AppInfo {
    type Error: std::error::Error + Send + Sync + 'static;
    async fn name(&self) -> Cow<'_, str>;
    async fn license(&self) -> Result<Cow<'_, AppLicense>, Self::Error>;
    async fn description(&self) -> Result<Cow<'_, str>, Self::Error>;
    async fn documentation(&self) -> Result<Cow<'_, str>, Self::Error>;
    async fn homepage(&self) -> Result<Cow<'_, str>, Self::Error>;
    async fn repository(&self) -> Result<Cow<'_, str>, Self::Error>;
    async fn version(&self) -> Result<Cow<'_, str>, Self::Error>;
}
///
/// about the application paths
///
pub trait AppPath {
    type Error: std::error::Error + Send + Sync + 'static;
    /// e.g. '~/.cargo/'
    async fn home_path(&self) -> Result<Cow<'_, Path>, Self::Error>;
    /// e.g. '~/.cargo/bin/rustup'
    async fn bin_path(&self) -> Result<Cow<'_, Path>, Self::Error>;
}
///
/// application operators
///
pub trait AppOper: Sized {
    type Error: std::error::Error + Send + Sync + 'static;
    type InstallInfo;
    type ReinstallInfo;
    type RemoveInfo;
    type UpdateInfo;
    async fn install(info: Self::InstallInfo) -> Result<Self, Self::Error>;
    async fn reinstall(self, info: Self::ReinstallInfo) -> Result<Self, Self::Error>;
    async fn remove(self, info: Self::RemoveInfo) -> Result<(), Self::Error>;
    async fn update(self, info: Self::UpdateInfo) -> Result<Self, Self::Error>;
}

impl Display for AppLicense {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AppLicense::Unknown => write!(f, "Unknown"),
            AppLicense::Text(s) => write!(f, "{s}"),
            AppLicense::Or(a, b) => write!(f, "{a} or {b}"),
            AppLicense::File(path) => write!(f, "{path:?}"),
        }
    }
}
