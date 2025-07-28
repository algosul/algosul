use proc_macro2::{Span, TokenStream};
use quote::{quote_spanned, ToTokens};
use syn::{
    braced,
    bracketed,
    parse::{Parse, ParseBuffer, ParseStream},
    punctuated::Punctuated,
    token,
};
#[derive(Clone)]
pub struct ArrayBase<B, T: Parse, S: Parse> {
    #[allow(dead_code)]
    pub token: B,
    pub elems: Punctuated<T, S>,
}
pub type Array<T> = ArrayBase<token::Bracket, T, token::Comma>;
pub type Block<T> = ArrayBase<token::Brace, T, token::Semi>;
impl<B: ToTokens, T: Parse + ToTokens, S: Parse + ToTokens>
    From<ArrayBase<B, T, S>> for TokenStream
{
    fn from(val: ArrayBase<B, T, S>) -> Self { val.into_token_stream() }
}
impl<B: ToTokens, T: Parse + ToTokens, S: Parse + ToTokens>
    From<ArrayBase<B, T, S>> for Vec<T>
{
    fn from(val: ArrayBase<B, T, S>) -> Self { val.into_elems().collect() }
}
impl<B, T: Parse, S: Parse> ArrayBase<B, T, S> {
    pub fn elems(&self) -> impl Iterator<Item = &T> { self.elems.iter() }

    pub fn into_elems(self) -> impl Iterator<Item = T> {
        self.elems.into_iter()
    }

    pub fn to_token_stream(&self) -> TokenStream
    where
        B: ToTokens,
        T: ToTokens,
        S: ToTokens,
    {
        let token = &self.token;
        let elems = &self.elems;
        quote_spanned! {Span::call_site()=>
            #token #elems #token
        }
    }

    pub fn into_token_stream(self) -> TokenStream
    where
        B: ToTokens,
        T: ToTokens,
        S: ToTokens,
    {
        let token = self.token;
        let elems = self.elems;
        quote_spanned! {Span::call_site()=>
            #token #elems #token
        }
    }

    fn parse_from_context(content: ParseBuffer, token: B) -> syn::Result<Self> {
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
        Ok(Self { token, elems })
    }
}
impl<T: Parse> Parse for Array<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let token = bracketed!(content in input);
        Self::parse_from_context(content, token)
    }
}
impl<T: Parse> Parse for Block<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let token = braced!(content in input);
        Self::parse_from_context(content, token)
    }
}
