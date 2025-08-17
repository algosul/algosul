use std::ops::Range;

use num_traits::{float::FloatCore, Bounded, Num};
pub trait NumConst
{
  const ZERO: Self;
  const ONE: Self;
  const TWO: Self;
  const THREE: Self;
  const FOUR: Self;
  const FIVE: Self;
  const SIX: Self;
  const SEVEN: Self;
  const EIGHT: Self;
  const NINE: Self;
  const TEN: Self;
}
/// + this trait can't implement for u8/i8
pub trait NumDegConst: NumConst
{
  const DEG_30: Self;
  const DEG_45: Self;
  const DEG_60: Self;
  const DEG_90: Self;
  const DEG_180: Self;
  const DEG_270: Self;
  const DEG_360: Self;
  const DEG_720: Self;
}
pub trait NumPercent<N = Self>
{
  /// # Safety
  /// + `range.start != range.end`
  fn percent_in<U: Num + From<N> + Clone>(self, range: Range<U>) -> U;
  /// # Safety
  /// + `range.start != range.end`
  fn try_percent_in<U: Num + TryFrom<N> + Clone>(
    self, range: Range<U>,
  ) -> Result<U, U::Error>;
}
pub trait NumLerp<N: FloatCore = Self>
{
  fn lerp_in(self, range: Range<N>) -> N;
  fn lerp(range: Range<N>, alpha: N) -> N;
}
pub trait NumNormalize<N = Self>
{
  /// # Result
  /// `0..max`
  fn normalize(self, max: N) -> N;
  /// # Result
  /// `0..360`
  fn normalize_deg(self) -> N
  where N: NumDegConst;
  /// # Result
  /// `0..1`
  fn normalize_01(self) -> N;
}
pub trait NumRemap<N = Self>
{
  fn remap<U: FloatCore + NumLerp<U> + From<N>>(
    self, from: Range<U>, to: Range<U>,
  ) -> U;
  fn try_remap<U: FloatCore + NumLerp<U> + TryFrom<N>>(
    self, from: Range<U>, to: Range<U>,
  ) -> Result<U, U::Error>;
  fn remap01<U: FloatCore + NumLerp<U> + From<N>>(self) -> U
  where
    N: Bounded,
    Self: Sized,
  {
    self
      .remap(N::min_value().into()..N::max_value().into(), U::zero()..U::one())
  }
  fn try_remap01<U: FloatCore + NumLerp<U> + TryFrom<N>>(
    self,
  ) -> Result<U, U::Error>
  where
    N: Bounded,
    Self: Sized,
  {
    Ok(self.try_remap(
      N::min_value().try_into()?..N::max_value().try_into()?,
      U::zero()..U::one(),
    ))?
  }
}
pub trait NumsRemap<N, const L: usize>
{
  fn remap<U: FloatCore + NumRemap<U> + From<N>>(
    self, from: Range<U>, to: Range<U>,
  ) -> [U; L];
  fn try_remap<U: FloatCore + NumRemap<U> + TryFrom<N>>(
    self, from: Range<U>, to: Range<U>,
  ) -> Result<[U; L], U::Error>;
  fn remap01<U: FloatCore + NumLerp<U> + From<N>>(self) -> [U; L]
  where
    N: Bounded,
    Self: Sized,
  {
    self
      .remap(N::min_value().into()..N::max_value().into(), U::zero()..U::one())
  }
  fn try_remap01<U: FloatCore + NumLerp<U> + TryFrom<N>>(
    self,
  ) -> Result<[U; L], U::Error>
  where
    N: Bounded,
    Self: Sized,
  {
    Ok(self.try_remap(
      N::min_value().try_into()?..N::max_value().try_into()?,
      U::zero()..U::one(),
    ))?
  }
}
impl<N: Num + PartialOrd + Clone + NumConst> NumNormalize<N> for N
{
  fn normalize(self, max: N) -> N
  {
    let temp = self % max.clone();
    if temp < N::ZERO { temp + max } else { temp }
  }

  fn normalize_deg(self) -> N
  where N: NumDegConst
  {
    self.normalize(N::DEG_360)
  }

  fn normalize_01(self) -> N
  {
    self.normalize(N::ONE)
  }
}
impl<N: Num> NumPercent<N> for N
{
  fn percent_in<U: Num + From<N> + Clone>(self, range: Range<U>) -> U
  {
    (<U as From<N>>::from(self) - range.start.clone())
      / (range.end - range.start)
  }

  fn try_percent_in<U: Num + TryFrom<N> + Clone>(
    self, range: Range<U>,
  ) -> Result<U, U::Error>
  {
    Ok(
      (<U as TryFrom<N>>::try_from(self)? - range.start.clone())
        / (range.end - range.start),
    )
  }
}
impl<N: FloatCore> NumLerp<N> for N
{
  fn lerp_in(self, range: Range<N>) -> N
  {
    Self::lerp(range, self)
  }

  fn lerp(range: Range<N>, alpha: N) -> N
  {
    ((N::one() - alpha) * range.start) + (alpha * range.end)
  }
}
impl<N: Num + NumPercent<N>> NumRemap<N> for N
{
  fn remap<U: FloatCore + NumLerp<U> + From<N>>(
    self, from: Range<U>, to: Range<U>,
  ) -> U
  {
    if from.start == from.end
    {
      to.start
    }
    else
    {
      let percent = self.percent_in::<U>(from);
      percent.lerp_in(to)
    }
  }

  fn try_remap<U: FloatCore + NumLerp<U> + TryFrom<N>>(
    self, from: Range<U>, to: Range<U>,
  ) -> Result<U, U::Error>
  {
    Ok(
      if from.start == from.end
      {
        to.start
      }
      else
      {
        let percent = self.try_percent_in::<U>(from)?;
        percent.lerp_in(to)
      },
    )
  }
}
impl<N: Num, const L: usize> NumsRemap<N, L> for [N; L]
{
  fn remap<U: FloatCore + NumRemap<U> + From<N>>(
    self, from: Range<U>, to: Range<U>,
  ) -> [U; L]
  {
    self.map(|x| <U as From<N>>::from(x).remap(from.clone(), to.clone()))
  }

  fn try_remap<U: FloatCore + NumRemap<U> + TryFrom<N>>(
    self, from: Range<U>, to: Range<U>,
  ) -> Result<[U; L], U::Error>
  {
    self.try_map(|x| {
      Ok(<U as TryFrom<N>>::try_from(x)?.remap(from.clone(), to.clone()))
    })
  }
}
macro_rules! impl_num_const {
  ($($ty:ty)*) => {
    $(
      impl NumConst for $ty {
        const ZERO: Self = 0;
        const ONE: Self = 1;
        const TWO: Self = 2;
        const THREE: Self = 3;
        const FOUR: Self = 4;
        const FIVE: Self = 5;
        const SIX: Self = 6;
        const SEVEN: Self = 7;
        const EIGHT: Self = 8;
        const NINE: Self = 9;
        const TEN: Self = 10;
      }
    )*
  };
}
macro_rules! impl_num_deg_const {
  ($($ty:ty)*) => {
    $(
      impl NumDegConst for $ty {
        const DEG_180: Self = 180;
        const DEG_270: Self = 270;
        const DEG_30: Self = 30;
        const DEG_360: Self = 360;
        const DEG_45: Self = 45;
        const DEG_60: Self = 60;
        const DEG_720: Self = 720;
        const DEG_90: Self = 90;
      }
    )*
  };
}
macro_rules! impl_float_const {
  ($($ty:ty)*) => {
    $(
      impl NumConst for $ty {
        const ZERO: Self = 0.0;
        const ONE: Self = 1.0;
        const TWO: Self = 2.0;
        const THREE: Self = 3.0;
        const FOUR: Self = 4.0;
        const FIVE: Self = 5.0;
        const SIX: Self = 6.0;
        const SEVEN: Self = 7.0;
        const EIGHT: Self = 8.0;
        const NINE: Self = 9.0;
        const TEN: Self = 10.0;
      }
      impl NumDegConst for $ty {
        const DEG_180: Self = 180.0;
        const DEG_270: Self = 270.0;
        const DEG_30: Self = 30.0;
        const DEG_360: Self = 360.0;
        const DEG_45: Self = 45.0;
        const DEG_60: Self = 60.0;
        const DEG_720: Self = 720.0;
        const DEG_90: Self = 90.0;
      }
    )*
  };
}
impl_num_const!(u8 i8 u16 i16 u32 i32 u64 i64 u128 i128);
impl_num_deg_const!(u16 i16 u32 i32 u64 i64 u128 i128);
#[cfg(feature = "unstable-f16-f128")]
impl_float_const!(f16 f128);
impl_float_const!(f32 f64);
