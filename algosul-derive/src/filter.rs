use std::path::Path;

use syn::{
    parse::{Parse, ParseStream},
    LitStr,
};

use crate::from_dir::filter;
syn::custom_keyword!(include);
syn::custom_keyword!(exclude);
#[derive(Clone)]
pub struct FileFilterItemTokens {
    #[allow(dead_code)]
    include:  include,
    includes: crate::array::Array<LitStr>,
    #[allow(dead_code)]
    exclude:  exclude,
    excludes: crate::array::Array<LitStr>,
}
#[derive(Debug, Clone)]
pub struct FileFilterItem {
    pub includes: Vec<glob::Pattern>,
    pub excludes: Vec<glob::Pattern>,
}
#[derive(Clone)]
pub struct FileFilterTokens {
    #[allow(dead_code)]
    filter_ident: filter,
    items:        crate::array::Array<FileFilterItem>,
}
#[derive(Debug, Clone)]
pub struct FileFilter {
    pub items: Vec<FileFilterItem>,
}
impl Parse for FileFilterItemTokens {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            include:  input.parse()?,
            includes: input.parse()?,
            exclude:  input.parse()?,
            excludes: input.parse()?,
        })
    }
}
impl Parse for FileFilterItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let tokens: FileFilterItemTokens = input.parse()?;
        let includes: Vec<_> = tokens
            .includes
            .into_elems()
            .map(|x| glob::Pattern::new(&x.value()).unwrap())
            .collect();
        let excludes: Vec<_> = tokens
            .excludes
            .into_elems()
            .map(|x| glob::Pattern::new(&x.value()).unwrap())
            .collect();
        Ok(Self { includes, excludes })
    }
}
impl Parse for FileFilterTokens {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self { filter_ident: input.parse()?, items: input.parse()? })
    }
}
impl Parse for FileFilter {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let tokens: FileFilterTokens = input.parse()?;
        Ok(Self { items: tokens.items.into_elems().collect() })
    }
}
impl FileFilterItem {
    pub fn dir_is_in_includes(&self, dir: &Path) -> bool {
        self.includes.iter().any(|item| item.matches_path(dir))
    }

    pub fn dir_is_in_excludes(&self, dir: &Path) -> bool {
        self.excludes.iter().any(|item| item.matches_path(dir))
    }

    pub fn path_is_in(&self, file: &Path) -> bool {
        self.includes.iter().any(|item| item.matches_path(file))
            && !self.excludes.iter().any(|item| item.matches_path(file))
    }
}
impl FileFilter {
    pub fn dir_is_in_includes(&self, dir: &Path) -> bool {
        self.items.iter().any(|item| item.dir_is_in_includes(dir))
    }

    pub fn dir_is_in_excludes(&self, dir: &Path) -> bool {
        self.items.iter().any(|item| item.dir_is_in_excludes(dir))
    }

    pub fn path_is_in(&self, file: &Path) -> bool {
        self.items.iter().any(|item| item.path_is_in(file))
    }
}
