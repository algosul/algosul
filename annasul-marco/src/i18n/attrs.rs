use std::{
    any::Any,
    borrow::Cow,
    collections::{HashMap, HashSet},
    fmt::Debug,
};

use log::{error, info, trace};
use proc_macro2::Ident;
use quote::ToTokens;
use syn::{meta::ParseNestedMeta, spanned::Spanned, LitStr};
pub trait Attr: Debug + Any {
    fn name(&self) -> Cow<'static, str>;
    fn parse_meta(&mut self, meta: ParseNestedMeta) -> syn::Result<()>;
}
pub trait IntoAttrs {
    fn into_attrs(self) -> HashMap<String, Box<dyn Attr>>;
}
pub trait ParseAttrs<T> {
    fn parse_attrs(&mut self, input: T) -> syn::Result<()>;
}
#[derive(Default, Debug)]
pub struct Ignore(bool);
#[derive(Default, Debug)]
pub struct Rename(Option<String>);
#[derive(Default, Debug, Clone)]
pub struct Format {
    pub fmt: HashSet<Ident>,
}
impl IntoAttrs for Vec<Box<dyn Attr>> {
    fn into_attrs(self) -> HashMap<String, Box<dyn Attr>> {
        HashMap::from_iter(self.into_iter().map(|x| (x.name().into_owned(), x)))
    }
}
impl ParseAttrs<ParseNestedMeta<'_>> for HashMap<String, Box<dyn Attr>> {
    fn parse_attrs(&mut self, meta: ParseNestedMeta) -> syn::Result<()> {
        let span = meta.path.span();
        let ident = meta
            .path
            .get_ident()
            .ok_or_else(|| syn::Error::new(span, "~~~ empty attr"))?;
        self.get_mut(&ident.to_string())
            .ok_or_else(|| {
                syn::Error::new(span, format!("~~~ unknown attr: {ident}"))
            })?
            .parse_meta(meta)
    }
}
impl Ignore {
    pub const NAME: &str = "ignore";

    pub fn value(&self) -> bool { self.0 }
}
impl Attr for Ignore {
    fn name(&self) -> Cow<'static, str> { Cow::Borrowed(Self::NAME) }

    fn parse_meta(&mut self, meta: ParseNestedMeta) -> syn::Result<()> {
        let name = self.name();
        if self.0 {
            return Err(syn::Error::new(
                meta.path.span(),
                "ignore attribute is already set",
            ));
        }
        if !meta.input.is_empty() {
            return Err(syn::Error::new(
                meta.path.span(),
                "ignore attribute does not take any arguments",
            ));
        }
        self.0 = true;
        info!("I18n({name})");
        Ok(())
    }
}
impl Rename {
    pub const NAME: &str = "rename";

    pub fn value(&self) -> Option<&str> { self.0.as_deref() }
}
impl Attr for Rename {
    fn name(&self) -> Cow<'static, str> { Cow::Borrowed(Self::NAME) }

    fn parse_meta(&mut self, meta: ParseNestedMeta) -> syn::Result<()> {
        let name = self.name();
        if self.0.is_some() {
            return Err(syn::Error::new(
                meta.path.span(),
                "rename attribute is already set",
            ));
        }
        if meta.input.is_empty() {
            return Err(syn::Error::new(
                meta.path.span(),
                "rename attribute requires an argument",
            ));
        }
        let value: LitStr = meta.value()?.parse()?;
        self.0 = Some(value.value());
        info!("I18n({name} = {})", value.to_token_stream());
        Ok(())
    }
}
impl Format {
    pub const NAME: &str = "format";

    pub fn fmt(&self) -> &HashSet<Ident> { &self.fmt }
}
impl Attr for Format {
    fn name(&self) -> Cow<'static, str> { Cow::Borrowed(Self::NAME) }

    fn parse_meta(&mut self, meta: ParseNestedMeta) -> syn::Result<()> {
        let span = meta.path.span();
        if !self.fmt.is_empty() {
            return Err(syn::Error::new(
                span,
                "format attribute is already set",
            ));
        }
        let input = meta.input;
        trace!("Parsing {input}");
        meta.parse_nested_meta(|meta| {
            let name = meta.path.get_ident().unwrap();
            if !meta.input.is_empty() {
                return Err(syn::Error::new(
                    meta.path.span(),
                    "format(a, b, ...) can't has value",
                ));
            }
            trace!("  {name}");
            self.fmt.insert(name.clone());
            Ok(())
        })
        .map_err(|err| {
            error!("Parse failed: {err}");
            err
        })?;
        info!("I18n(format)");
        Ok(())
    }
}
