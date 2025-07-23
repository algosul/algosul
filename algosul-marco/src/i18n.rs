use std::{any::Any, collections::HashMap, fmt::Debug};

use itertools::Itertools;
use log::trace;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote_spanned, ToTokens};
use syn::{
    ext::IdentExt,
    meta::ParseNestedMeta,
    spanned::Spanned,
    Attribute,
    DataEnum,
    Field,
    Fields,
    FieldsNamed,
    FieldsUnnamed,
    LitStr,
    Visibility,
};

use crate::i18n::attrs::{Attr, IntoAttrs, ParseAttrs};
mod attrs;
#[derive(Debug)]
struct I18nField {
    name:          Ident,
    original_name: Option<Ident>,
    format:        Option<attrs::Format>,
    span:          Span,
}
fn parse_attr_meta(
    meta: ParseNestedMeta, context: &mut HashMap<String, Box<dyn Attr>>,
) -> syn::Result<()> {
    context.parse_attrs(meta)
}
fn parse_attrs(
    span: Span, name: Option<Ident>, attrs: impl AsRef<[Attribute]>,
) -> syn::Result<Option<I18nField>> {
    let context: Vec<Box<dyn Attr>> = vec![
        Box::new(Option::<attrs::Ignore>::default()),
        Box::new(Option::<attrs::Rename>::default()),
        Box::new(Option::<attrs::Format>::default()),
    ];
    let mut context = context.into_attrs();
    let mut error: Option<syn::Error> = None;
    for attr in attrs.as_ref() {
        if !attr.path().is_ident("i18n") {
            trace!(
                "  Skipping non-i18n attribute: {}",
                attr.path().to_token_stream()
            );
            continue;
        }
        trace!("  {}", attr.to_token_stream());
        trace!("  {{");
        let err = attr
            .parse_nested_meta(|meta| parse_attr_meta(meta, &mut context))
            .err();
        match &mut error {
            None => error = err,
            Some(error) => {
                let _ = err.map(|err| error.combine(err));
            }
        };
        trace!("  }}");
    }
    if let Some(err) = error {
        return Err(err);
    }
    let ignore = (context.remove(attrs::Ignore::NAME).unwrap() as Box<dyn Any>)
        .downcast::<Option<attrs::Ignore>>()
        .unwrap_or_else(|_| panic!("~~~ Inner ERROR: downcast error"))
        .map(|ignore| ignore.value());
    let rename = (context.remove(attrs::Rename::NAME).unwrap() as Box<dyn Any>)
        .downcast::<Option<attrs::Rename>>()
        .unwrap_or_else(|_| panic!("~~~ Inner ERROR: downcast error"));
    let format = (context.remove(attrs::Format::NAME).unwrap() as Box<dyn Any>)
        .downcast::<Option<attrs::Format>>()
        .unwrap_or_else(|_| panic!("~~~ Inner ERROR: downcast error"));
    const IGNORE: &str = attrs::Ignore::NAME;
    const RENAME: &str = attrs::Rename::NAME;
    const _FORMAT: &str = attrs::Format::NAME;
    match (ignore, name, *rename, *format) {
        (Some(true), _, None, None) => Ok(None),
        (Some(true), _, _, _) => Err(syn::Error::new(
            span,
            format!("'{IGNORE}' cannot be used with other attributes"),
        )),
        (_, Some(name), Some(rename), format)
            if name.unraw() == rename.ident.unraw() =>
        {
            eprintln!("WARNING: '{RENAME}' is the same as original name");
            Ok(Some(I18nField {
                name: rename.ident.clone(),
                original_name: Some(name),
                format,
                span,
            }))
        }
        (_, name, Some(rename), format) => Ok(Some(I18nField {
            name: rename.ident.clone(),
            original_name: name,
            format,
            span,
        })),
        (_, Some(name), None, format) => Ok(Some(I18nField {
            name: name.clone(),
            original_name: Some(name),
            format,
            span,
        })),
        (_, None, None, _) => Err(syn::Error::new(
            span,
            format!("unnamed field must provide '{RENAME}' attribute"),
        )),
    }
}
fn parse_field(field: Field) -> syn::Result<Option<I18nField>> {
    parse_attrs(field.span(), field.ident, &field.attrs)
}
fn parse_fields(
    fields: impl Iterator<Item = Field>,
) -> syn::Result<Vec<I18nField>> {
    let mut result = Vec::new();
    let mut errors: Option<syn::Error> = None;
    for field in fields {
        match parse_field(field) {
            Ok(None) => {}
            Ok(Some(value)) => result.push(value),
            Err(err) => {
                if let Some(ref mut e) = errors {
                    e.combine(err);
                } else {
                    errors = Some(err);
                }
            }
        }
    }
    if let Some(e) = errors { Err(e) } else { Ok(result) }
}
macro_rules! data_name {
    ($ident:ident) => {
        format_ident!("__algosul_i18n_{}_Data", $ident)
    };
}
fn parse_i18n_fields(
    span: Span, vis: Visibility, name: Ident, fields: Vec<I18nField>,
) -> syn::Result<TokenStream> {
    let data_name = data_name!(name);
    trace!("{data_name}");
    let mut to_data_tokens = TokenStream::new();
    let mut into_data_tokens = TokenStream::new();
    let mut data_fields_tokens = TokenStream::new();
    let mut check_tokens = TokenStream::new();
    for field in fields {
        let o_name = field.original_name.unwrap();
        let name = field.name;
        let name_str = name.to_string();
        let span = field.span;
        if let Some(fmt) = field.format {
            let fmt = fmt.fmt;
            let error_fmt = "'self.{}' must use [{}], but use {:?}";
            let must_use_str =
                fmt.iter().map(|ident| ident.to_string()).join(", ");
            let items: Vec<_> = fmt
                .iter()
                .map(|ident| LitStr::new(&ident.to_string(), ident.span()))
                .collect();
            trace!("error_fmt = {error_fmt}");
            check_tokens.extend::<TokenStream>(quote_spanned! {span => {
                let keys = keys(&self.#name)?;
                let required_keys: HashSet<&str> =
                    HashSet::from([#(#items),*]);
                ::log::info!("keys = {keys:?}");
                ::log::info!("required_keys = {required_keys:?}");
                if keys
                    .iter()
                    .map(|x| x.as_ref())
                    .collect::<HashSet<_>>()
                    != required_keys {
                    return Err(
                        ::algosul_core::i18n::Error::FormatParameterMismatch(
                            ::std::format!(
                                #error_fmt,
                                #name_str,
                                #must_use_str,
                                keys
                            )
                        )
                    );
                }
            }});
        };
        to_data_tokens.extend::<TokenStream>(quote_spanned! {span =>
            #name: self.#o_name.clone(),
        });
        into_data_tokens.extend::<TokenStream>(quote_spanned! {span =>
            #name: self.#o_name,
        });
        data_fields_tokens.extend::<TokenStream>(quote_spanned! {span =>
            #name: String,
        });
    }
    let tokens = quote_spanned! {span =>
        impl ::algosul_core::i18n::I18n for #name {
            type DataType = #data_name;
            fn to_data(&self) -> Self::DataType {
                Self::DataType {
                    #to_data_tokens
                }
            }
            fn into_data(self) -> Self::DataType {
                Self::DataType {
                    #into_data_tokens
                }
            }
        }
        #[allow(non_camel_case_types)]
        #[doc(hidden)]
        #[derive(
            ::core::fmt::Debug,
            ::serde::Serialize,
            ::serde::Deserialize
        )]
        #vis struct #data_name {
            #data_fields_tokens
        }
        impl ::algosul_core::i18n::I18nData for #data_name {
            type I18n = #name;
            fn check(&self) -> ::algosul_core::i18n::Result<()> {
                use ::std::collections::HashSet;
                fn keys(fmt: &str) -> ::strfmt::Result<HashSet<String>> {
                    let mut buffer = HashSet::new();
                    ::strfmt::strfmt_map(fmt, |fmt: ::strfmt::Formatter| {
                        buffer.insert(fmt.key.to_string());
                        Ok(())
                    })?;
                    Ok(buffer)
                }
                #check_tokens
                Ok(())
            }
        }
        impl ::core::convert::Into<#data_name> for #name {
            fn into(self) -> #data_name {
                use ::algosul_core::i18n::I18n;
                self.into_data()
            }
        }
    };
    Ok(tokens)
}
pub(crate) fn parse_named_struct(
    span: Span, vis: Visibility, name: Ident, fields: FieldsNamed,
) -> syn::Result<TokenStream> {
    trace!("named struct");
    trace!("{{");
    trace!("fields:");
    let fields = parse_fields(fields.named.into_iter())?;
    trace!("}}");
    parse_i18n_fields(span, vis, name, fields)
}
pub(crate) fn parse_unnamed_struct(
    span: Span, vis: Visibility, name: Ident, fields: FieldsUnnamed,
) -> syn::Result<TokenStream> {
    trace!("unnamed struct");
    trace!("{{");
    trace!("fields:");
    let fields = parse_fields(fields.unnamed.into_iter())?;
    trace!("}}");
    parse_i18n_fields(span, vis, name, fields)
}
pub(crate) fn parse_unit_struct(
    span: Span, vis: Visibility, name: Ident,
) -> syn::Result<TokenStream> {
    trace!("unit struct");
    trace!("{{");
    trace!("}}");
    parse_i18n_fields(span, vis, name, vec![])
}
pub(crate) fn parse_enum(
    _span: Span, vis: Visibility, _name: Ident, data: DataEnum,
) -> syn::Result<TokenStream> {
    trace!("enum");
    trace!("{{");
    for variant in data.variants.into_iter() {
        let vis = vis.clone();
        let span = variant.span();
        let ident = variant.ident.clone();
        parse_attrs(span, Some(ident), &variant.attrs)?;
        let _ = match (variant.fields, variant.ident) {
            (Fields::Named(fields), name) => {
                parse_named_struct(span, vis, name, fields)
            }
            (Fields::Unnamed(fields), name) => {
                parse_unnamed_struct(span, vis, name, fields)
            }
            (Fields::Unit, name) => parse_unit_struct(span, vis, name),
        };
    }
    trace!("}}");
    todo!()
}
