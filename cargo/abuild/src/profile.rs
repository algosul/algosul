use std::{fmt::Debug, path::Path};

use crate::{
    lang::ComplierInfo,
    utils::{Builder, HasBuilder},
};
/// optimization level
#[derive(Debug, Clone)]
pub enum OptLevel {
    /// O0 optimization
    /// + Rustc: `-O0`
    O0,
    /// O1 optimization
    /// + Rustc: `-O1`
    O1,
    /// O2 optimization
    /// + Rustc: `-O2`
    O2,
    /// O3 optimization
    /// + Rustc: `-O3`
    O3,
    /// Os optimization
    /// + Rustc: `-Os`
    Os,
    /// Oz optimization
    /// + Rustc: `-Oz`
    Oz,
}
pub trait Profile: Debug {
    fn compile_option(
        &self, file_path: &Path, compiler_info: &ComplierInfo,
    ) -> CompileOptions;
}
#[derive(Default, Debug, Clone)]
pub struct DevProfile;
#[derive(Default, Debug, Clone)]
pub struct ReleaseProfile;
#[derive(Debug, Clone)]
pub enum DebugInfoOption {
    /// none of debug information
    /// + Rustc: `-C debuginfo=0`
    None,
    /// limited debug information
    /// + Rustc: `-C debuginfo=1`
    Limited,
    /// full of debug information
    /// + Rustc: `-C debuginfo=2`
    Full,
    /// custom debug information
    /// + Rustc: `-C debuginfo="$custom"`
    Custom(String),
}
#[derive(Default, Debug, Clone)]
pub enum DebugAssertions {
    /// Determined by the compiler
    #[default]
    Default,
    /// Rustc: `-C debug-assertions=true`
    On,
    /// Rustc: `-C debug-assertions=false`
    Off,
}
#[derive(Default, Debug, Clone)]
pub enum LTO {
    /// "thin local LTO"
    /// Rustc: `-C lto="false"`
    #[default]
    Local,
    /// "fat" LTO
    /// Rustc: `-C lto="true"`
    Fat,
    /// "thin" LTO
    /// Rustc: `-C lto="thin"`
    Thin,
    /// Disables LTO
    /// Rustc: `-C lto="off"`
    Off,
}
/// > [!NOTE]
/// >
/// > Only in Rustc
#[derive(Debug, Clone)]
pub enum OverflowChecks {
    /// + Rustc: `-C overflow-checks="true"`
    True,
    /// + Rustc: `-C overflow-checks="false"`
    False,
}
#[derive(Debug, Clone)]
pub enum CompileProfile {
    /// + Rustc: rustc default profile
    Debug,
    /// + Rustc: `todo!()`
    Release,
    /// + Rustc: `--profile-use="$custom"`
    Custom(String),
}
#[derive(Default, Debug, Clone)]
pub struct CompileOptions {
    /// see [OptLevel]
    pub opt_level:         Option<OptLevel>,
    /// see [LTO]
    pub lto:               Option<LTO>,
    /// see [DebugInfoOption]
    pub debug_info_option: Option<DebugInfoOption>,
    /// see [DebugAssertions]
    pub debug_assertions:  Option<DebugAssertions>,
    /// see [OverflowChecks]
    pub overflow_checks:   Option<OverflowChecks>,
    /// see [CompileProfile]
    pub profile:           Option<CompileProfile>,
    /// compile flags
    pub flags:             Vec<String>,
}
#[derive(Debug, Clone)]
pub struct CompilerOptionsBuilder {}
impl HasBuilder for CompileOptions {
    type Builder = CompilerOptionsBuilder;
}
impl Builder for CompilerOptionsBuilder {
    type Error = ();
    type Output = CompileOptions;

    fn new() -> Self { Self {} }

    fn build(&self) -> Result<Self::Output, Self::Error> {
        Ok(CompileOptions::dev())
    }
}
impl CompileOptions {
    pub fn dev() -> Self {
        Self { profile: Some(CompileProfile::Debug), ..Default::default() }
    }

    pub fn release() -> Self {
        Self { profile: Some(CompileProfile::Release), ..Default::default() }
    }
}
impl Profile for DevProfile {
    fn compile_option(
        &self, _file_path: &Path, _compiler_info: &ComplierInfo,
    ) -> CompileOptions {
        CompileOptions::dev()
    }
}
impl Profile for ReleaseProfile {
    fn compile_option(
        &self, _file_path: &Path, _compiler_info: &ComplierInfo,
    ) -> CompileOptions {
        CompileOptions::release()
    }
}
