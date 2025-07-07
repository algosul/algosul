use std::borrow::Cow;

use crate::lang::LanguageInfo;
pub struct CPP;
impl LanguageInfo for CPP {
    fn lang_name() -> Cow<'static, str> { "C++".into() }
}
