use log::debug;
use proc_macro::TokenStream;
use syn::parse2;

use crate::from_dir::InputBuf;
mod from_dir;
/// Convert from folder to module
/// # Grammar
/// ```ignore
/// // use
/// from_dir!($module)
/// // $module
/// $vis mod $ident from $path {
///     $($filter;)*
/// }
/// // $filter (text)
/// text $file_filter
/// // $filter (binary)
/// binary $file_filter
/// // $file_filter
/// [$(include [$($path),*] exclude [$($path),*]),*]
/// ```
/// # Example
/// ```
/// # use algosul_derive::from_dir;
/// from_dir!(mod assets from "../rc" {
///     text [include ["lang/*.toml"] exclude []];
///     binary [include ["images/*.png"] exclude []];
/// });
/// println!("en-US.toml: {}", assets::lang::en_US);
/// println!("zh-CN.toml: {}", assets::lang::zh_CN);
/// ```
#[proc_macro]
pub fn from_dir(input: TokenStream) -> TokenStream {
    let _ = env_logger::try_init();
    let path = proc_macro::Span::call_site().local_file().unwrap();
    let base = path.parent().unwrap();
    let input: InputBuf = parse2(input.into()).unwrap();
    debug!("base: {base:?}, input: {:?}", input.path());
    from_dir::from_dir(base, input.path(), input.as_ref()).unwrap().into()
}
