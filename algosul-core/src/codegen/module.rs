use std::{
    env,
    path::{Path, PathBuf},
};

use glob::PatternError;
use proc_macro2::Ident;
use syn::{parse_quote, token::Brace, Item, ItemMod};
pub trait ModuleExt {
    /// 添加元素
    fn add_item(&mut self, item: Item) -> &mut Self;
    fn add_item_by_glob<F: Fn(&Path) -> Item>(
        &mut self, pattern: &str, f: F,
    ) -> Result<&mut Self, PatternError> {
        let manifest_dir =
            PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let full_pattern = manifest_dir.join(pattern);
        let pattern_str = full_pattern.to_str().unwrap();
        for entry in glob::glob(pattern_str)? {
            let entry = entry.unwrap();
            let rel_path = entry.strip_prefix(&manifest_dir).unwrap();
            self.add_item(f(rel_path));
        }
        Ok(self)
    }
    fn add_include_str_by_glob<F: Fn(&Path) -> Ident>(
        &mut self, pattern: &str, f: F,
    ) -> Result<&mut Self, PatternError> {
        self.add_item_by_glob(pattern, |entry| {
            let ident = f(entry);
            let entry = entry.to_str().unwrap();
            parse_quote!(
                pub const #ident: &str = include_str!(
                    concat!(env!("CARGO_MANIFEST_DIR"), "/", #entry)
                );
            )
        })
    }
    fn add_include_byte_by_glob<F: Fn(&Path) -> Ident>(
        &mut self, pattern: &str, f: F,
    ) -> Result<&mut Self, PatternError> {
        self.add_item_by_glob(pattern, |entry| {
            let ident = f(entry);
            let entry = entry.to_str().unwrap();
            parse_quote!(
                pub const #ident: &[u8] = include_byte!(
                    concat!(env!("CARGO_MANIFEST_DIR"), "/", #entry)
                );
            )
        })
    }
}
impl ModuleExt for ItemMod {
    fn add_item(&mut self, item: Item) -> &mut Self {
        self.content
            .get_or_insert_with(|| (Brace::default(), Vec::new()))
            .1
            .push(item);
        self
    }
}
