#![allow(async_fn_in_trait)]
#![feature(string_from_utf8_lossy_owned)]
//! #Example
//! ```
//! use algosul::{
//!   app::{
//!     AppOper,
//!     apps::rust::{Result, Rustup},
//!   },
//!   process::Process,
//! };
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!   let mut installer = Rustup::installer().await?;
//!   installer.on_status_changed(|status| {
//!     println!("status: {status:?}");
//!     Ok(())
//!   })?;
//!   let rustup = installer.run().await?;
//!   println!("rustup installed: {rustup:?}");
//!   Ok(())
//! }
//! ```
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
#[cfg(feature = "process")]
pub mod process;
#[cfg(not(feature = "process"))]
pub(crate) mod process;
pub mod utils;
