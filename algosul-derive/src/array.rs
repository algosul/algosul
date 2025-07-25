use proc_macro2::{Span, TokenStream};
use quote::{quote_spanned, ToTokens};
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token,
    Attribute,
};
#[derive(Clone)]
pub struct Array<T: Parse> {
    pub attrs:         Vec<Attribute>,
    #[allow(dead_code)]
    pub bracket_token: token::Bracket,
    pub elems:         Punctuated<T, token::Comma>,
}
impl<T: Parse + ToTokens> Into<TokenStream> for Array<T> {
    fn into(self) -> TokenStream { self.into_token_stream() }
}
impl<T: Parse> Array<T> {
    pub fn attrs(&self) -> &[Attribute] { &self.attrs }

    pub fn elems(&self) -> impl Iterator<Item = &T> { self.elems.iter() }

    pub fn into_elems(self) -> impl Iterator<Item = T> {
        self.elems.into_iter()
    }

    pub fn to_token_stream(&self) -> TokenStream
    where T: ToTokens {
        let attrs = &self.attrs;
        let elems = self.elems.iter();
        quote_spanned! {Span::call_site()=>
            #(#attrs)* [ #(#elems),* ]
        }
    }

    pub fn into_token_stream(self) -> TokenStream
    where T: ToTokens {
        let attrs = self.attrs;
        let elems = self.elems.into_iter();
        quote_spanned! {Span::call_site()=>
            #(#attrs)* [ #(#elems),* ]
        }
    }
}
impl<T: Parse> Parse for Array<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let bracket_token = bracketed!(content in input);
        let mut elems = Punctuated::new();
        while !content.is_empty() {
            let first: T = content.parse()?;
            elems.push_value(first);
            if content.is_empty() {
                break;
            }
            let punct = content.parse()?;
            elems.push_punct(punct);
        }
        Ok(Array::<T> { attrs: Vec::new(), bracket_token, elems })
    }
}
