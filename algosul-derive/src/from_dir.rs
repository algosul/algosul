use std::{
    fs,
    io,
    path::{Path, PathBuf},
};

use algosul_core::codegen::ident::StrExt;
use log::{debug, warn};
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

use crate::{
    filter::{FileFilter, FileFilterTokens},
    tokens::Block,
};
custom_keyword!(from);
custom_keyword!(text);
custom_keyword!(binary);
enum InputFilterItemTokens {
    Text {
        #[allow(dead_code)]
        text:   text,
        filter: FileFilterTokens,
    },
    Binary {
        #[allow(dead_code)]
        binary: binary,
        filter: FileFilterTokens,
    },
}
#[derive(Clone)]
enum InputFilterItem {
    Text { filter: FileFilter },
    Binary { filter: FileFilter },
}
struct InputFilterTokens {
    items: Block<InputFilterItemTokens>,
}
#[derive(Clone)]
struct InputFilter {
    items: Vec<InputFilterItem>,
}
struct InputTokens {
    vis:       Visibility,
    #[allow(dead_code)]
    mod_ident: Token![mod],
    name:      Ident,
    #[allow(dead_code)]
    from:      from,
    path:      LitStr,
    filter:    InputFilterTokens,
}
pub struct InputBuf {
    vis:    Visibility,
    name:   Ident,
    path:   PathBuf,
    filter: InputFilter,
}
pub struct Input<'a> {
    vis:    &'a Visibility,
    name:   &'a Ident,
    path:   &'a Path,
    filter: &'a InputFilter,
}
impl InputBuf {
    pub fn path(&self) -> &Path { &self.path }

    pub fn as_ref(&self) -> Input<'_> {
        Input {
            vis:    &self.vis,
            name:   &self.name,
            path:   &self.path,
            filter: &self.filter,
        }
    }
}
impl From<InputFilterItemTokens> for InputFilterItem {
    fn from(tokens: InputFilterItemTokens) -> Self {
        match tokens {
            InputFilterItemTokens::Text { text: _, filter } => {
                Self::Text { filter: filter.into() }
            }
            InputFilterItemTokens::Binary { binary: _, filter } => {
                Self::Binary { filter: filter.into() }
            }
        }
    }
}
impl Parse for InputFilterItemTokens {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(text) {
            let text = input.parse()?;
            let filter = input.parse()?;
            Ok(Self::Text { text, filter })
        } else if lookahead.peek(binary) {
            let binary = input.parse()?;
            let filter = input.parse()?;
            Ok(Self::Binary { binary, filter })
        } else {
            Err(lookahead.error())
        }
    }
}
impl Parse for InputFilterTokens {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self { items: input.parse()? })
    }
}
impl From<InputFilterTokens> for InputFilter {
    fn from(tokens: InputFilterTokens) -> Self {
        Self {
            items: tokens
                .items
                .into_elems()
                .map(InputFilterItem::from)
                .collect(),
        }
    }
}
impl Parse for InputFilter {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(input.parse::<InputFilterTokens>()?.into())
    }
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
impl Parse for InputBuf {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let tokens: InputTokens = input.parse()?;
        let path = proc_macro::Span::call_site().local_file().unwrap();
        let path = path.parent().unwrap();
        Ok(Self {
            vis:    tokens.vis,
            name:   tokens.name,
            path:   path.join(tokens.path.value()),
            filter: tokens.filter.into(),
        })
    }
}
pub(crate) fn from_dir(
    base: &Path, input: &Path, Input { vis, name, path, filter }: Input,
) -> io::Result<TokenStream> {
    let mut items: Vec<TokenStream> = vec![];
    let pub_vis = parse_quote!(pub);
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = &entry.path();
        let name = path
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .into_owned()
            .to_valid_ident();
        let name = &Ident::new(&name, Span::call_site());
        let vis = &pub_vis;
        let item = if entry.file_type()?.is_dir() {
            from_dir(base, input, Input { vis, name, path, filter })
        } else if entry.file_type()?.is_file() {
            from_file(base, input, Input { vis, name, path, filter })
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
    base: &Path, input: &Path, Input { vis, name, path, filter }: Input,
) -> io::Result<TokenStream> {
    let path_str = path.strip_prefix(base).unwrap().to_str().unwrap();
    let path = path.strip_prefix(input).unwrap();
    let on_text = || {
        debug!("[{name}] text: {path_str}");
        Ok(quote_spanned! {Span::call_site()=>
            #vis const #name: &str = include_str!(#path_str);
        })
    };
    let on_binary = || {
        debug!("[{name}] binary: {path_str}");
        Ok(quote_spanned! {Span::call_site()=>
            #vis const #name: &[u8] = include_bytes!(#path_str);
        })
    };
    for item in &filter.items {
        return match item {
            InputFilterItem::Text { filter } => {
                if !filter.path_is_in(path) {
                    continue;
                }
                on_text()
            }
            InputFilterItem::Binary { filter } => {
                if !filter.path_is_in(path) {
                    continue;
                }
                on_binary()
            }
        };
    }
    warn!("ignore: {path:?}");
    Ok(TokenStream::new())
}
