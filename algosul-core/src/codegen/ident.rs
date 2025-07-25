use std::ffi::{CStr, CString, OsStr, OsString};

use unicode_xid::UnicodeXID;
pub trait StrExt {
    fn is_valid_ident(&self) -> bool;
}
pub trait StrMapExt {
    type Item;
    fn map_to_valid_ident(&self) -> impl Iterator<Item = Self::Item>;
}
pub trait StringExt: Sized {
    fn to_valid_ident(&self) -> Self
    where Self: Sized;
}
impl StrExt for str {
    fn is_valid_ident(&self) -> bool {
        let mut chars = self.chars();
        match chars.next() {
            Some(c) if c.is_xid_start() || c == '_' => {
                chars.all(|c| c.is_xid_continue())
            }
            _ => false,
        }
    }
}
impl StrMapExt for str {
    type Item = char;

    fn map_to_valid_ident(&self) -> impl Iterator<Item = char> {
        let mut chars = self.chars();
        std::iter::once(
            chars.next().filter(|&c| c.is_xid_start()).unwrap_or('_'),
        )
        .chain(chars.map(|c| if c.is_xid_continue() { c } else { '_' }))
    }
}
impl StrExt for OsStr {
    fn is_valid_ident(&self) -> bool {
        self.to_str().map(str::is_valid_ident).unwrap_or(false)
    }
}
impl StrExt for CStr {
    fn is_valid_ident(&self) -> bool {
        self.to_str().map(str::is_valid_ident).unwrap_or(false)
    }
}
impl StringExt for String {
    fn to_valid_ident(&self) -> Self
    where Self: Sized {
        self.map_to_valid_ident().collect()
    }
}
impl StringExt for OsString {
    fn to_valid_ident(&self) -> Self
    where Self: Sized {
        OsString::from(
            self.to_string_lossy().map_to_valid_ident().collect::<String>(),
        )
    }
}
impl StringExt for CString {
    fn to_valid_ident(&self) -> Self
    where Self: Sized {
        CString::new(
            self.to_string_lossy().map_to_valid_ident().collect::<String>(),
        )
        .unwrap()
    }
}
#[cfg(test)]
mod tests {
    use crate::codegen::ident::StrExt;
    #[test]
    fn test_is_valid_ident() {
        let valids = ["_", "_abc", "_123", "abc_", "hello_world"];
        for s in valids {
            assert!(s.is_valid_ident(), "{s:?} should be valid");
        }
        assert!("_".is_valid_ident());
        assert!("_abc".is_valid_ident());
        assert!("_123".is_valid_ident());
        assert!("abc".is_valid_ident());
        let invalids = ["", "0", " ", "_0123-", "-", "!hello", "hello-world"];
        for s in invalids {
            assert!(!s.is_valid_ident(), "{s:?} should not be valid");
        }
    }
}
