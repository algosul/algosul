use std::ops::Range;

use num_traits::{float::FloatCore, Bounded, NumOps};

/// Returns the integer constant `N` as the target numeric type `T`.
///
/// Works for any numeric type (including floats), e.g. `int::<f64, 180>()` ->
/// `180.0`.
/// # Examples
/// ```
/// # use algosul_math::num::int;
/// let x: i32 = int::<i32, 180>();
/// assert_eq!(x, 180);
///
/// let y: f64 = int::<f64, 180>();
/// assert_eq!(y, 180.0);
///
/// let z: i32 = int::<i32, -180>();
/// assert_eq!(z, -180);
///
/// let y: f64 = int::<f64, -180>();
/// assert_eq!(y, -180.0);
/// ```
#[inline]
pub fn int<T: NumInt<N>, const N: i128>() -> T
{
  T::int()
}
/// Returns the integer constant `N` as the target numeric type `T` in a const
/// context.
///
/// Prefer using [`int`] in most cases, as it is more flexible and has the same
/// performance. [`const_int`] is mainly useful in `const fn` or other
/// `const` contexts.
#[inline]
pub const fn const_int<T: NumConstInt<N>, const N: i128>() -> T
{
  T::CONST_INT
}
/// For impl the integer constant `N` as the target numeric type `T`.
///
/// see [Self::int]
pub trait NumInt<const N: i128>: Sized
{
  /// Returns the integer constant `N` as the target numeric type `T`.
  ///
  /// Works for any numeric type (including floats), e.g. `int::<f64, 180>()` ->
  /// `180.0`.
  fn int() -> Self;
  #[inline]
  fn set_int(&mut self)
  {
    *self = Self::int();
  }
  #[inline]
  fn is_int(&self) -> bool
  where Self: PartialEq
  {
    *self == Self::int()
  }
}
/// For impl the integer constant `N` as the target numeric type `T` in a const
/// context.
///
/// Prefer using [`NumInt`] or [`int`] in most cases, as it is more flexible and
/// has the same performance. [`NumConstInt`] and [`const_int`] is mainly useful
/// in `const fn` or other `const` contexts.
pub trait NumConstInt<const N: i128>: NumInt<N>
{
  const CONST_INT: Self;
}
pub trait NumPercent<N = Self>
{
  /// # Safety
  /// + `range.start != range.end`
  fn percent_in<U: NumOps + From<N> + Clone>(self, range: Range<U>) -> U;
  /// # Safety
  /// + `range.start != range.end`
  fn try_percent_in<U: NumOps + TryFrom<N> + Clone>(
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
  where N: NumInt<360>;
  /// # Result
  /// `0..1`
  fn normalize_01(self) -> N
  where N: NumInt<1>;
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
impl<T: NumConstInt<N>, const N: i128> NumInt<N> for T
{
  fn int() -> Self
  {
    Self::CONST_INT
  }
}
impl<N: NumOps> NumPercent<N> for N
{
  fn percent_in<U: NumOps + From<N> + Clone>(self, range: Range<U>) -> U
  {
    (<U as From<N>>::from(self) - range.start.clone())
      / (range.end - range.start)
  }

  fn try_percent_in<U: NumOps + TryFrom<N> + Clone>(
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
impl<N: NumOps + PartialOrd + Clone + NumInt<0>> NumNormalize<N> for N
{
  fn normalize(self, max: N) -> N
  {
    let temp = self % max.clone();
    if temp < int::<N, 0>() { temp + max } else { temp }
  }

  fn normalize_deg(self) -> N
  where N: NumInt<360>
  {
    self.normalize(int::<N, 360>())
  }

  fn normalize_01(self) -> N
  where N: NumInt<1>
  {
    self.normalize(int::<N, 1>())
  }
}
impl<N: NumOps + NumPercent<N>> NumRemap<N> for N
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
impl<N: NumOps, const L: usize> NumsRemap<N, L> for [N; L]
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
macro_rules! impl_i_const {
  ($($ty:ty)*) => {
    $(impl_num_const!(impl $ty =>
      -10 -9 -8 -7 -6 -5 -4 -3 -2 -1 0 1 2 3 4 5 6 7 8 9 10
    );)*
  };
}
macro_rules! impl_u_const {
  ($($ty:ty)*) => {
    $(impl_num_const!(impl $ty =>
      0 1 2 3 4 5 6 7 8 9 10
    );)*
  };
}
macro_rules! impl_i_deg_const {
  ($($ty:ty)*) => {
    $(impl_num_const!(impl $ty =>
      -720 -360 -270 -180 -150 -120 -90 -60 -30 -15
      15 30 60 90 120 150 180 270 360 720
    );)*
  };
}
macro_rules! impl_u_deg_const {
  ($($ty:ty)*) => {
    $(impl_num_const!(impl $ty =>
      15 30 60 90 120 150 180 270 360 720
    );)*
  };
}
macro_rules! impl_f_const {
  ($($ty:ty)*) => {
    $(impl_float_const!(impl $ty =>
      -720 -720.0 -360 -360.0 -270 -270.0 -180 -180.0 -150 -150.0
      -120 -120.0 -90 -90.0 -60 -60.0 -30 -30.0 -15 -15.0
      -10 -10.0 -9 -9.0 -8 -8.0 -7 -7.0 -6 -6.0
      -5 -5.0 -4 -4.0 -3 -3.0 -2 -2.0 -1 -1.0
      0 0.0 1 1.0 2 2.0 3 3.0 4 4.0 5 5.0
      6 6.0 7 7.0 8 8.0 9 9.0 10 10.0
      15 15.0 30 30.0 60 60.0 90 90.0 120 120.0 150 150.0
      180 180.0 270 270.0 360 360.0 720 720.0
    );)*
  };
}
macro_rules! impl_num_const {
  (impl $ty:ty => $($n:literal)+) => {
    $(impl NumConstInt<$n> for $ty { const CONST_INT: Self = $n; })+
  };
}
macro_rules! impl_float_const {
  (impl $ty:ty => $($i:literal $f:literal)+) => {
    $(impl NumConstInt<$i> for $ty { const CONST_INT: Self = $f; })+
  };
}
impl_u_const!(u8 u16 u32 u64 u128);
impl_i_const!(i8 i16 i32 i64 i128);
impl_u_deg_const!(u16 u32 u64 u128);
impl_i_deg_const!(i16 i32 i64 i128);
#[cfg(feature = "unstable-f16-f128")]
impl_f_const!(f16 f128);
impl_f_const!(f32 f64);
