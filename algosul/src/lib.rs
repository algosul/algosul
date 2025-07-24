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
mod marco;
mod os;
