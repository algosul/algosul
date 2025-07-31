use std::{env, path::Path};

use algosul_core::codegen::{ident::StrExt, module::ModuleExt};
use glob::PatternError;
use proc_macro2::{Ident, Span};
use syn::{ItemMod, parse_quote};
fn from_dir(path: impl AsRef<Path>) -> Result<(), PatternError>
{
  let path = path.as_ref();
  println!("cargo:rerun-if-changed={path:?}");
  let mut module: ItemMod = parse_quote! {
      mod lang;
  };
  module.add_include_str_by_glob(
    &format!("{}/lang/*.toml", path.to_str().unwrap()),
    |path| {
      Ident::new(
        path.file_stem().unwrap().to_valid_ident().to_str().unwrap(),
        Span::call_site(),
      )
    },
  )?;
  Ok(())
}
fn main()
{
  env_logger::init();
  env::set_current_dir(env::var("CARGO_MANIFEST_DIR").unwrap()).unwrap();
  from_dir("tests/rc").unwrap();
}
