use std::{
    borrow::Borrow,
    ffi::{CStr, CString, OsStr, OsString},
    str::FromStr,
};

use unicode_xid::UnicodeXID;
pub trait StrExt {
    type Owned: Borrow<Self>;
    fn is_valid_ident(&self) -> bool;
    fn to_valid_ident(&self) -> Self::Owned;
}
impl StrExt for str {
    type Owned = String;

    fn is_valid_ident(&self) -> bool {
        let mut chars = self.chars();
        match chars.next() {
            Some(c) if c.is_xid_start() || c == '_' => {
                chars.all(|c| c.is_xid_continue())
            }
            _ => false,
        }
    }

    fn to_valid_ident(&self) -> Self::Owned {
        let mut chars = self.chars();
        let mut buffer = String::new();
        let first = chars.next();
        let first = first
            .filter(|&c| {
                if c.is_ascii_digit() {
                    buffer.push('_');
                    return true;
                };
                c.is_xid_start()
            })
            .unwrap_or('_');
        buffer.push(first);
        chars.for_each(|c| {
            if c.is_xid_continue() { buffer.push(c) } else { buffer.push('_') }
        });
        buffer
    }
}
impl StrExt for OsStr {
    type Owned = OsString;

    fn is_valid_ident(&self) -> bool {
        self.to_str().map(str::is_valid_ident).unwrap_or(false)
    }

    fn to_valid_ident(&self) -> Self::Owned {
        OsString::from(self.to_string_lossy().to_valid_ident())
    }
}
impl StrExt for CStr {
    type Owned = CString;

    fn is_valid_ident(&self) -> bool {
        self.to_str().map(str::is_valid_ident).unwrap_or(false)
    }

    fn to_valid_ident(&self) -> Self::Owned {
        CString::from_str(&self.to_string_lossy().to_valid_ident()).unwrap()
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
