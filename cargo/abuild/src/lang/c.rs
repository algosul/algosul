use std::borrow::Cow;

use crate::lang::LanguageInfo;
pub struct C;
impl LanguageInfo for C {
    fn lang_name() -> Cow<'static, str> { "C".into() }
}
