use std::{borrow::Cow, path::Path};
pub mod c;
pub mod cpp;
pub mod csharp;
pub mod rust;
pub trait LanguageInfo {
    fn lang_name() -> Cow<'static, str>;
}
pub use c::C;
pub use cpp::CPP;
pub use csharp::CSharp;
pub use rust::Rust;
/// langauge
#[derive(Debug, Clone, Copy)]
pub enum Langauge {
    /// ISO C
    C,
    /// ISO C++
    CPP,
    /// C#
    CSharp,
    /// Rust
    Rust,
}
/// langauge file type
#[derive(Debug, Clone, Copy)]
pub enum FileType {
    /// `.c`
    CSource,
    /// `.h`
    CHeader,
    /// `.cpp`
    CPPSource,
    /// `.hpp`
    CPPHeader,
    /// `.ixx`
    CPPModule,
    /// `.cs`
    CSharp,
    /// `.rs`
    Rust,
}
/// complier information
#[derive(Debug, Clone)]
pub struct ComplierInfo {
    /// e.g. rustc/MSVC/MinGW
    name:     String,
    version:  String,
    langauge: Langauge,
}
impl FileType {
    pub fn from_file_extension(ext: impl AsRef<str>) -> Option<Self> {
        match ext.as_ref() {
            "rs" => Some(Self::Rust),
            "hpp" | "h++" | "hxx" | "hh" => Some(Self::CPPHeader),
            "cpp" | "c++" | "cxx" | "cc" => Some(Self::CPPSource),
            "h" => Some(Self::CHeader),
            "c" => Some(Self::CSource),
            "cppm" | "c++m" | "cxxm" | "ccm" | "ixx" | "ii" => {
                Some(Self::CPPModule)
            }
            "cs" => Some(Self::CSharp),
            _ => None,
        }
    }

    pub fn from_file(value: impl AsRef<Path>) -> Option<Self> {
        Self::from_file_extension(value.as_ref().extension()?.to_str()?)
    }
}
