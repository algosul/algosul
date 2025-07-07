use std::borrow::Cow;

use crate::lang::LanguageInfo;
pub struct CSharp;
impl LanguageInfo for CSharp {
    fn lang_name() -> Cow<'static, str> { "C#".into() }
}
