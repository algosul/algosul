use std::{
    fs,
    io,
    path::{Path, PathBuf},
};

use algosul_core::codegen::ident::StringExt;
use log::{trace, warn};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote_spanned;
use syn::{
    parse::{Parse, ParseStream},
    parse_quote,
    LitStr,
    Token,
    Visibility,
};
struct InputTokens {
    vis:   Visibility,
    name:  Ident,
    #[allow(dead_code)]
    brace: Token![,],
    path:  LitStr,
}
pub struct Input {
    vis:      Visibility,
    name:     Ident,
    pub path: PathBuf,
}
impl Parse for InputTokens {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            vis:   input.parse()?,
            name:  input.parse()?,
            brace: input.parse()?,
            path:  input.parse()?,
        })
    }
}
impl Parse for Input {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let tokens: InputTokens = input.parse()?;
        Ok(Self {
            vis:  tokens.vis,
            name: tokens.name,
            path: PathBuf::from(tokens.path.value()),
        })
    }
}
pub(crate) fn from_dir(
    base: &Path, Input { vis, name, path }: Input,
) -> io::Result<TokenStream> {
    let mut items: Vec<TokenStream> = vec![];
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        let name = path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned()
            .to_valid_ident();
        let name = Ident::new(&name, Span::call_site());
        let vis = parse_quote!(pub);
        let item = if entry.file_type()?.is_dir() {
            from_dir(base, Input { vis, name, path })
        } else if entry.file_type()?.is_file() {
            from_file(base, Input { vis, name, path })
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
    base: &Path, Input { vis, name, path }: Input,
) -> io::Result<TokenStream> {
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
            from_file_no_extension(base, vis, name, filename.unwrap(), path)
        }
    }
}
fn from_file_no_extension(
    base: &Path, vis: Visibility, name: Ident, filename: &str, path: &str,
) -> io::Result<TokenStream> {
    match filename {
        ".gitignore" | ".gitmodules" => Ok(TokenStream::new()),
        _ => Ok(TokenStream::new()),
    }
}
