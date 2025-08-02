use std::{
    fmt::{Debug, Formatter},
    path::Path,
};

use syn::{
    parse::{Parse, ParseStream},
    LitStr,
};

use crate::codegen::tokens::Array;
syn::custom_keyword!(include);
syn::custom_keyword!(exclude);
#[derive(Clone)]
pub struct FileFilterItemTokens {
    #[allow(dead_code)]
    include:  include,
    includes: Array<LitStr>,
    #[allow(dead_code)]
    exclude:  exclude,
    excludes: Array<LitStr>,
}
#[derive(Clone)]
pub struct FileFilterItem {
    pub includes: Vec<glob::Pattern>,
    pub excludes: Vec<glob::Pattern>,
}
#[derive(Clone)]
pub struct FileFilterTokens {
    items: Array<FileFilterItemTokens>,
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
impl From<FileFilterItemTokens> for FileFilterItem {
    fn from(tokens: FileFilterItemTokens) -> Self {
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
        Self { includes, excludes }
    }
}
impl Parse for FileFilterItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(input.parse::<FileFilterItemTokens>()?.into())
    }
}
impl From<FileFilterTokens> for FileFilter {
    fn from(tokens: FileFilterTokens) -> Self {
        Self {
            items: tokens
                .items
                .into_elems()
                .map(FileFilterItem::from)
                .collect(),
        }
    }
}
impl Parse for FileFilterTokens {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self { items: input.parse()? })
    }
}
impl Parse for FileFilter {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(input.parse::<FileFilterTokens>()?.into())
    }
}
impl FileFilterItem {
    pub fn path_is_in(&self, file: &Path) -> bool {
        self.includes.iter().any(|item| item.matches_path(file))
            && !self.excludes.iter().any(|item| item.matches_path(file))
    }
}
impl FileFilter {
    pub fn path_is_in(&self, file: &Path) -> bool {
        self.items.iter().any(|item| item.path_is_in(file))
    }
}
impl Debug for FileFilterItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileFilterItem")
            .field(
                "include",
                &self
                    .includes
                    .iter()
                    .map(|item| item.as_str())
                    .collect::<Vec<_>>(),
            )
            .field(
                "exclude",
                &self
                    .excludes
                    .iter()
                    .map(|item| item.as_str())
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}
