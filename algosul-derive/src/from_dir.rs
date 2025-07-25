use std::{
    borrow::Cow,
    fs,
    io,
    path::{Path, PathBuf},
};

use algosul_core::codegen::ident::StringExt;
use log::{trace, warn};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote_spanned;
use syn::{
    custom_keyword,
    parse::{Parse, ParseStream},
    parse_quote,
    LitStr,
    Token,
    Visibility,
};

use crate::filter::FileFilter;
custom_keyword!(from);
custom_keyword!(filter);
struct InputTokens {
    vis:       Visibility,
    #[allow(dead_code)]
    mod_ident: Token![mod],
    name:      Ident,
    #[allow(dead_code)]
    from:      from,
    path:      LitStr,
    filter:    FileFilter,
}
pub struct Input<'a> {
    vis:        Cow<'a, Visibility>,
    name:       Cow<'a, Ident>,
    pub path:   Cow<'a, Path>,
    pub filter: Cow<'a, FileFilter>,
}
impl Parse for InputTokens {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            vis:       input.parse()?,
            mod_ident: input.parse()?,
            name:      input.parse()?,
            from:      input.parse()?,
            path:      input.parse()?,
            filter:    input.parse()?,
        })
    }
}
impl Parse for Input<'_> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let tokens: InputTokens = input.parse()?;
        Ok(Self {
            vis:    Cow::Owned(tokens.vis),
            name:   Cow::Owned(tokens.name),
            path:   Cow::Owned(PathBuf::from(tokens.path.value())),
            filter: Cow::Owned(tokens.filter),
        })
    }
}
pub(crate) fn from_dir(
    base: &Path, Input { vis, name, path, filter }: Input,
) -> io::Result<TokenStream> {
    let mut items: Vec<TokenStream> = vec![];
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path: Cow<'_, Path> = Cow::Owned(entry.path());
        let name = path
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .into_owned()
            .to_valid_ident();
        let name = Cow::Owned(Ident::new(&name, Span::call_site()));
        let vis = Cow::Owned(parse_quote!(pub));
        let filter = filter.clone();
        let item = if entry.file_type()?.is_dir() {
            from_dir(base, Input { vis, name, path, filter })
        } else if entry.file_type()?.is_file() {
            from_file(base, Input { vis, name, path, filter })
        } else {
            warn!("unknown file type: {path:?}");
            continue;
        }?;
        items.push(item);
    }
    Ok(quote_spanned! { Span::call_site() =>
        #vis mod #name {
            #(#items)*
        }
    })
}
fn from_file(
    base: &Path, Input { vis, name, path, filter }: Input,
) -> io::Result<TokenStream> {
    if !filter.path_is_in(&path) {
        return Ok(TokenStream::new());
    }
    let filename = path.file_name().map(|os_str| os_str.to_str().unwrap());
    let extension = path.extension().map(|os_str| os_str.to_str().unwrap());
    let path = path.strip_prefix(base).unwrap().to_str().unwrap();
    trace!("path: {path:?}");
    match extension {
        Some(
            "txt" | "md" | "log" | "csv" | "tsv" | "json" | "xml" | "yml"
            | "yaml" | "toml" | "ini" | "conf",
        ) => Ok(quote_spanned! {Span::call_site()=>
            #vis const #name: &str = include_str!(#path);
        }),
        Some("bin" | "png" | "jpg" | "gif") => {
            Ok(quote_spanned! {Span::call_site()=>
                #vis const #name: &[u8] = include_bytes!(#path);
            })
        }
        Some(_) => Ok(TokenStream::new()),
        None => {
            let vis = &vis;
            let name = &name;
            from_file_no_extension(base, vis, name, filename.unwrap(), path)
        }
    }
}
fn from_file_no_extension(
    base: &Path, vis: &Visibility, name: &Ident, filename: &str, path: &str,
) -> io::Result<TokenStream> {
    match filename {
        ".gitignore" | ".gitmodules" => Ok(TokenStream::new()),
        _ => Ok(TokenStream::new()),
    }
}
