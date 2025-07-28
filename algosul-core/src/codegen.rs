//! # Example
//! ```
//! use proc_macro2::{Ident, Span};
//! use syn::{parse_quote, File, ItemMod};
//! use algosul_core::codegen::file::FileExt;
//! use algosul_core::codegen::ident::StrExt;
//! use algosul_core::codegen::module::ModuleExt;
//! let mut file: File = parse_quote!(
//!     #![allow(non_upper_case_globals)]
//! );
//! file.add_mod({
//!     let mut module: ItemMod = parse_quote!(
//!         pub mod lang;
//!     );
//!     module.add_include_str_by_glob("rc/lang/*.toml", |path| {
//!         Ident::new(
//!             path.file_stem()
//!                 .unwrap()
//!                 .to_valid_ident()
//!                 .to_str()
//!                 .unwrap(),
//!             Span::call_site(),
//!         )
//!     }).unwrap();
//!     module
//! });
//! ```
pub mod file;
pub mod filter;
pub mod fs;
pub mod ident;
pub mod module;
pub mod tokens;
