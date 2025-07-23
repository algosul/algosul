use std::{
    any::Any,
    borrow::Cow,
    collections::{HashMap, HashSet},
    fmt::Debug,
};

use log::{error, info, trace};
use proc_macro2::{Ident, Span};
use quote::ToTokens;
use syn::{meta::ParseNestedMeta, spanned::Spanned, LitStr};
pub trait Attr: Debug + Any {
    fn name(&self) -> Cow<'static, str>;
    fn parse_meta(&mut self, meta: ParseNestedMeta) -> syn::Result<()>;
    fn error_attr_already(&self, span: Span) -> syn::Error {
        let name = self.name();
        syn::Error::new(span, format!("{name} attribute is already set"))
    }
    fn check_input_not_empty(&self, meta: &ParseNestedMeta) -> syn::Result<()> {
        if meta.input.is_empty() {
            Err(syn::Error::new(
                meta.path.span(),
                format!("{} attribute requires an argument", self.name()),
            ))
        } else {
            Ok(())
        }
    }
    fn check_input_is_empty(&self, meta: &ParseNestedMeta) -> syn::Result<()> {
        if meta.input.is_empty() {
            Ok(())
        } else {
            Err(syn::Error::new(
                meta.path.span(),
                format!(
                    "{} attribute does not take any arguments",
                    self.name()
                ),
            ))
        }
    }
}
pub trait IntoAttrs {
    fn into_attrs(self) -> HashMap<String, Box<dyn Attr>>;
}
pub trait ParseAttrs<T> {
    fn parse_attrs(&mut self, input: T) -> syn::Result<()>;
}
#[derive(Debug)]
pub struct Ignore(bool);
#[derive(Debug)]
pub struct Rename {
    pub name:  String,
    pub ident: Ident,
}
#[derive(Debug, Clone)]
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
impl Attr for Option<Ignore> {
    fn name(&self) -> Cow<'static, str> { Cow::Borrowed(Ignore::NAME) }

    fn parse_meta(&mut self, meta: ParseNestedMeta) -> syn::Result<()> {
        let name = self.name();
        if self.is_some() {
            return Err(self.error_attr_already(meta.path.span()));
        }
        self.check_input_is_empty(&meta)?;
        *self = Some(Ignore(true));
        info!("I18n({name})");
        Ok(())
    }
}
impl Rename {
    pub const NAME: &str = "rename";
}
impl Attr for Option<Rename> {
    fn name(&self) -> Cow<'static, str> { Cow::Borrowed(Rename::NAME) }

    fn parse_meta(&mut self, meta: ParseNestedMeta) -> syn::Result<()> {
        let rename = self.name();
        if self.is_some() {
            return Err(self.error_attr_already(meta.path.span()));
        }
        self.check_input_not_empty(&meta)?;
        let value: LitStr = meta.value()?.parse()?;
        let (name, span) = (value.value(), value.span());
        let ident = Ident::new(&name, span);
        info!("I18n({rename} = {})", value.to_token_stream());
        *self = Some(Rename { name, ident });
        Ok(())
    }
}
impl Format {
    pub const NAME: &str = "format";
}
impl Attr for Option<Format> {
    fn name(&self) -> Cow<'static, str> { Cow::Borrowed(Format::NAME) }

    fn parse_meta(&mut self, meta: ParseNestedMeta) -> syn::Result<()> {
        let span = meta.path.span();
        if self.is_some() {
            return Err(self.error_attr_already(span));
        }
        self.check_input_not_empty(&meta)?;
        let input = meta.input;
        trace!("Parsing {input}");
        let mut format = Format { fmt: HashSet::<Ident>::new() };
        meta.parse_nested_meta(|meta| {
            let name = meta.path.get_ident().unwrap();
            if !meta.input.is_empty() {
                return Err(syn::Error::new(
                    meta.path.span(),
                    format!("{}(a, b, ...) can't has value", self.name()),
                ));
            }
            trace!("  {name}");
            format.fmt.insert(name.clone());
            Ok(())
        })
        .map_err(|err| {
            error!("Parse failed: {err}");
            err
        })?;
        info!("I18n({})", self.name());
        *self = Some(format);
        Ok(())
    }
}
