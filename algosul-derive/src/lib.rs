use std::borrow::Cow;

use itertools::Itertools;
use log::trace;
use proc_macro::TokenStream;
use syn::parse2;

use crate::from_dir::Input;
mod array;
mod filter;
mod from_dir;
/// # Examples
///
/// ```
/// # use algosul_derive::from_dir;
/// from_dir!(
///     pub mod asset from "/path/to/asset" filter [
///         include ["**"] exclude []
///     ]
/// );
/// ```
#[proc_macro]
pub fn from_dir(input: TokenStream) -> TokenStream {
    let _ = env_logger::try_init();
    let path = proc_macro::Span::call_site().local_file().unwrap();
    let base = path.parent().unwrap();
    let mut input: Input = parse2(input.into()).unwrap();
    input.path = Cow::Owned(base.join(&input.path));
    let debug: Vec<_> = input
        .filter
        .items
        .iter()
        .map(|item| {
            let includes: Vec<_> =
                item.includes.iter().map(|item| item.as_str()).collect();
            let excludes: Vec<_> =
                item.excludes.iter().map(|item| item.as_str()).collect();
            format!("include {includes:?} exclude {excludes:?}")
        })
        .collect();
    trace!("filter {debug:?}");
    from_dir::from_dir(base, input).unwrap().into()
}
