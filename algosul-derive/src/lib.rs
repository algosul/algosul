use proc_macro::TokenStream;
use syn::parse2;

use crate::from_dir::Input;
mod from_dir;
/// # Examples
///
/// ```
/// # use algosul_derive::from_dir;
/// from_dir!(pub asset, "/path/to/asset");
/// ```
#[proc_macro]
pub fn from_dir(input: TokenStream) -> TokenStream {
    env_logger::init();
    let path = proc_macro::Span::call_site().local_file().unwrap();
    let base = path.parent().unwrap();
    let mut input: Input = parse2(input.into()).unwrap();
    input.path = base.join(input.path);
    from_dir::from_dir(base, input).unwrap().into()
}
