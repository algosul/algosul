#![cfg_attr(feature = "unstable", feature(array_try_map))]
//! # Example
//! + `Vector<T, N>`
//! + `Matrix<T, ROW, COL>`
//! + `Color<T; N>`

use std::ops::Range;

use num_traits::{float::FloatCore, Bounded};

#[cfg(feature = "color")]
pub mod color;

pub mod num;

pub trait Remap<N>
{
  type Output<U: FloatCore + From<N>>;
  type TryOutput<U: FloatCore + TryFrom<N>>;

  fn remap<U>(self, from: Range<U>, to: Range<U>) -> Self::Output<U>
  where U: FloatCore + From<N>;

  fn try_remap<U>(
    self, from: Range<U>, to: Range<U>,
  ) -> Result<Self::TryOutput<U>, U::Error>
  where U: FloatCore + TryFrom<N>;

  fn remap01<U>(self) -> Self::Output<U>
  where
    U: FloatCore + From<N>,
    N: Bounded,
    Self: Sized,
  {
    self
      .remap(N::min_value().into()..N::max_value().into(), U::zero()..U::one())
  }

  fn try_remap01<U>(self) -> Result<Self::TryOutput<U>, U::Error>
  where
    U: FloatCore + TryFrom<N>,
    N: Bounded,
    Self: Sized,
  {
    Ok(self.try_remap(
      N::min_value().try_into()?..N::max_value().try_into()?,
      U::zero()..U::one(),
    ))?
  }
}
