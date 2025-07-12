//! #Example
//! ```no_run
//! #[cfg(feature = "app-apps")]
//! {
//!     let rustup = tokio::runtime::Runtime::new()
//!         .unwrap()
//!         .block_on(async {
//!             use annasul::app::{
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
#[cfg(any(doc, feature = "app"))]
pub mod app;
mod marco;
mod os_impl;
