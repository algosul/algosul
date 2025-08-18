use proc_macro2::{Ident, TokenStream};
use syn::{
  parse::{Parse, ParseStream},
  parse_quote_spanned,
  spanned::Spanned,
  Expr,
  ExprField,
  Member,
};

pub struct InputBuf
{
  expr: ExprField,
}
impl InputBuf
{
  pub fn expr(&self) -> &ExprField
  {
    &self.expr
  }
}
impl Parse for InputBuf
{
  fn parse(input: ParseStream) -> syn::Result<Self>
  {
    Ok(Self { expr: input.parse()? })
  }
}
pub fn get(InputBuf { expr }: InputBuf) -> syn::Result<TokenStream>
{
  let span = expr.span();
  let ident = expr.member;
  let base = expr.base;
  let mut tokens = Vec::<Expr>::new();
  let ident = match ident
  {
    Member::Named(name) => name,
    Member::Unnamed(_) =>
    {
      panic!("Can't support number index");
    }
  };
  for c in ident.to_string().chars()
  {
    let c_ident = Ident::new(&c.to_string(), ident.span());
    tokens.push(parse_quote_spanned! {span=>
      #base.#c_ident()
    });
  }
  Ok(parse_quote_spanned! {span=>
    [#(#tokens),*]
  })
}
