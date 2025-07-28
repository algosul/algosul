//! #Example
//! ```no_run
//! #[cfg(feature = "app-apps")]
//! {
//!     let rustup = tokio::runtime::Runtime::new()
//!         .unwrap()
//!         .block_on(async {
//!             use algosul::app::{
//!                 AppOper,
//!                 apps::rust::{InstallInfo, Rustup},
//!             };
//!             // install rustup
//!             Rustup::install(InstallInfo::Default).await
//!         })
//!         .unwrap();
//!     println!("Hello {rustup:#?}");
//! }
//! ```

#![allow(async_fn_in_trait)]
#[cfg(feature = "app")]
pub mod app;
#[cfg(feature = "asset")]
pub mod asset;
#[cfg(feature = "codegen")]
pub mod codegen;
#[cfg(feature = "deps")]
pub mod deps;
#[cfg(feature = "macros")]
pub mod macros;
#[cfg(not(feature = "macros"))]
pub(crate) mod macros;
mod os;
