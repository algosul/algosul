use std::borrow::Cow;

use crate::lang::LanguageInfo;
pub struct Rust;
impl LanguageInfo for Rust {
    fn lang_name() -> Cow<'static, str> { "Rust".into() }
}
