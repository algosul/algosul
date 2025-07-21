use log::trace;
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput, Fields};
mod i18n;
/// # Example
/// ```
/// # use ::annasul_marco::I18n;
/// #[derive(I18n)]
/// struct Text {
///     #[i18n(ignore)]
///     ignore: (),
///     #[i18n(rename = "name")]
///     name:   String,
///     #[i18n(format(name))]
///     format: String,
/// }
/// ```
#[proc_macro_derive(I18n, attributes(i18n))]
pub fn i18n_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    env_logger::init();
    trace!("Parsing i18n attribute: {input}");
    let input = parse_macro_input!(input as DeriveInput);
    let span = input.span();
    match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => {
                i18n::parse_named_struct(span, input.vis, input.ident, fields)
            }
            Fields::Unnamed(fields) => {
                i18n::parse_unnamed_struct(span, input.vis, input.ident, fields)
            }
            Fields::Unit => {
                i18n::parse_unit_struct(span, input.vis, input.ident)
            }
        },
        Data::Enum(data) => {
            i18n::parse_enum(span, input.vis, input.ident, data)
        }
        Data::Union(_) => panic!("I18n cannot be derived for unions"),
    }
    .unwrap()
    .into()
}
