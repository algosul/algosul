// Copyright (c) 2025 air (https://yuanair.github.io).
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, version 3 of the License only.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//!
//!
//! # app
//! ```no_run
//! println!("Hello");
//! let rustup = tokio::runtime::Runtime::new().unwrap().block_on( async {
//!     use annasul::app::AppOper;
//!     use annasul::app::apps::rust::{InstallInfo, Rustup};
//!     // install rustup
//!     Rustup::install(InstallInfo::Default).await
//! } ).unwrap();
//! println!("Hello {rustup:#?}");
//! ```
//!

#![allow(async_fn_in_trait)]

#[cfg(any(doc, feature = "app"))]
pub mod app;
mod os_impl;

mod marco;
