#![feature(portable_simd)]
#![feature(f16, f128)]
#![feature(impl_trait_in_assoc_type)]
#![feature(array_repeat)]
#![feature(negative_impls)]
//! # Example
//! + `Vector<T, N>`
//! + `Matrix<T, ROW, COL>`
//! + `Color<T; N>`

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "color")]
pub mod color;

pub mod num;
