use std::ops::Range;

use algosul_derive::get;
use num_traits::{float::FloatCore, Bounded, Euclid, Num, One};
use thiserror::Error;

use crate::num::{NumLerp, NumPercent, NumsRemap};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Color<T: Clone>
{
  G(ColorG<T>),
  Ga(ColorGa<T>),
  Bgr(ColorBgr<T>),
  Bgra(ColorBgra<T>),
  Rgb(ColorRgb<T>),
  Rgba(ColorRgba<T>),
  Hsv(ColorHsv<T>),
  Hsva(ColorHsva<T>),
}

#[derive(Debug, Clone, Eq, PartialEq, Error)]
pub enum ColorCastError
{
  #[error("Input out of range")]
  InputOutOfRange,
  #[error("Computation out of range")]
  ComputationOutOfRange,
}

pub type ColorCastResult<T> = Result<T, ColorCastError>;

pub trait ColorGrayIndex<T: Clone>
{
  fn gray(&self) -> T;
}
pub trait ColorRgbIndex<T: Clone>
{
  fn r(&self) -> T;
  fn g(&self) -> T;
  fn b(&self) -> T;
}
pub trait ColorHsvIndex<T: Clone>
{
  fn h(&self) -> T;
  fn s(&self) -> T;
  fn v(&self) -> T;
}
pub trait ColorAlphaIndex<T: Clone>
{
  fn a(&self) -> T;
}
pub trait ColorRemap<N>
{
  type Output<U: FloatCore + NumLerp<U> + From<N>>;
  type Result<U: FloatCore + NumLerp<U> + TryFrom<N>>;
  fn remap<U: FloatCore + NumLerp<U> + From<N>>(
    self, from: Range<U>, to: Range<U>,
  ) -> Self::Output<U>;
  fn try_remap<U: FloatCore + NumLerp<U> + TryFrom<N>>(
    self, from: Range<U>, to: Range<U>,
  ) -> Result<Self::Result<U>, U::Error>;

  fn remap01<U: FloatCore + NumLerp<U> + From<N>>(self) -> Self::Output<U>
  where
    N: Bounded,
    Self: Sized,
  {
    self
      .remap(N::min_value().into()..N::max_value().into(), U::zero()..U::one())
  }
  fn try_remap01<U: FloatCore + NumLerp<U> + TryFrom<N>>(
    self,
  ) -> Result<Self::Result<U>, U::Error>
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
/// # Safety
/// `h`, `s`, `v` must in the range `0.0..=1.0`
pub unsafe fn hsv_to_rgb_uncheck<T: FloatCore>([h, s, v]: [T; 3]) -> [T; 3]
{
  let _0 = T::zero();
  let _1 = T::one();
  let _2 = _1 + _1;
  let _6 = _2 + _2 + _2;
  let c = v * s;
  let h_prime = h * _6;
  let x = c * (_1 - ((h_prime % _2) - _1).abs());
  let (r1, g1, b1) = match h_prime.to_u32().unwrap()
  {
    0 => (c, x, _0),
    1 => (x, c, _0),
    2 => (_0, c, x),
    3 => (_0, x, c),
    4 => (x, _0, c),
    _ => (c, _0, x),
  };
  let m = v - c;
  [r1 + m, g1 + m, b1 + m]
}
/// # Safety
/// `r`, `g`, `b` must in the range `0.0..=1.0`
pub unsafe fn rgb_to_hsv_uncheck<T: FloatCore + Euclid>(
  [r, g, b]: [T; 3],
) -> [T; 3]
{
  let _0 = T::zero();
  let _1 = T::one();
  let _2 = _1 + _1;
  let _4 = _2 + _2;
  let _6 = _4 + _2;
  let _10 = _4 + _6;
  let _60 = _6 * _10;
  let _360 = _6 * _6 * _10;
  let max = r.max(g).max(b);
  let min = r.min(g).min(b);
  let delta = max - min;
  let mut h = if delta == _0
  {
    _0
  }
  else if max == r
  {
    _60 * ((g - b) / delta).rem_euclid(&_6)
  }
  else if max == g
  {
    _60 * ((b - r) / delta + _2)
  }
  else
  {
    _60 * ((r - g) / delta + _4)
  };
  if h < _0
  {
    h = h + _360;
  }
  let s = if max == _0 { _0 } else { delta / max };
  [h, s, max]
}
/// # Range
/// `h`, `s`, `v` must in the range `0.0..=1.0`
pub fn hsv_to_rgb<T: FloatCore>(hsv: [T; 3]) -> ColorCastResult<[T; 3]>
{
  if check_color(hsv)
  {
    let rgb = unsafe { hsv_to_rgb_uncheck(hsv) };
    if check_color(rgb)
    {
      Ok(rgb)
    }
    else
    {
      Err(ColorCastError::ComputationOutOfRange)
    }
  }
  else
  {
    Err(ColorCastError::InputOutOfRange)
  }
}
/// # Range
/// `r`, `g`, `b` must in the range `0.0..=1.0`
pub fn rgb_to_hsv<T: FloatCore + Euclid>(rgb: [T; 3])
-> ColorCastResult<[T; 3]>
{
  if check_color(rgb)
  {
    let hsv = unsafe { rgb_to_hsv_uncheck(rgb) };
    if check_color(hsv)
    {
      Ok(hsv)
    }
    else
    {
      Err(ColorCastError::ComputationOutOfRange)
    }
  }
  else
  {
    Err(ColorCastError::InputOutOfRange)
  }
}
/// # Range
/// Must in the range `0.0..=1.0`
pub fn check_color<T: FloatCore, const N: usize>(color: [T; N]) -> bool
{
  color.iter().all(|x| (T::zero()..=T::one()).contains(x))
}
macro_rules! impl_color {
  () => {};
  (
    $name:ident $size:expr =>
    $(@gray => $gray:expr)?,
    $(@rgb => $r:expr, $g:expr, $b:expr)?,
    $(@hsv => $h:expr, $s:expr, $v:expr)?,
    $(@a => $from_ty:ty, $a:expr, $from_no_a:expr)?,
    $(@from => $from:ty, $from_fn:expr $(,)?)*;
    $($rest:tt)*
  ) => {
    #[derive(Default, Debug, Clone, Eq, PartialEq, Hash)]
    pub struct $name<T: Clone>(pub [T; $size]);
    impl<N: Num + Clone + NumPercent<N>> ColorRemap<N> for $name<N> {
      type Output<U: FloatCore + NumLerp<U> + From<N>> = $name<U>;
      type Result<U: FloatCore + NumLerp<U> + TryFrom<N>> = $name<U>;
      fn remap<U: FloatCore + NumLerp<U> + From<N>>(
        self, from: Range<U>, to: Range<U>
      ) -> Self::Output<U> {
        $name::<U>(self.0.remap(from, to))
      }
      fn try_remap<U: FloatCore + NumLerp<U> + TryFrom<N>>(
        self, from: Range<U>, to: Range<U>
      ) -> Result<Self::Result<U>, U::Error> {
        self.0.try_remap(from, to).map($name::<U>)
      }
    }
    $(impl<T: Clone> ColorGrayIndex<T> for $name<T> {
      fn gray(&self) -> T {$gray(self)}
    })?
    $(impl<T: Clone> ColorRgbIndex<T> for $name<T> {
      fn r(&self) -> T {$r(self)}
      fn g(&self) -> T {$g(self)}
      fn b(&self) -> T {$b(self)}
    })?
    $(impl<T: Clone> ColorHsvIndex<T> for $name<T> {
      fn h(&self) -> T {$h(self)}
      fn s(&self) -> T {$s(self)}
      fn v(&self) -> T {$v(self)}
    })?
    $(
      impl<T: Clone> ColorAlphaIndex<T> for $name<T> {
        fn a(&self) -> T {$a(self)}
      }
      impl<T: One + Clone> From<$from_ty> for $name<T>
      {
        fn from(color: $from_ty) -> Self
        {
          Self($from_no_a(color))
        }
      }
    )?
    $(impl<T: Clone> From<$from> for $name<T> {
      fn from(value: $from) -> Self {
        $from_fn(value)
      }
    })*
    impl_color!($($rest)*);
  };
}

impl_color!(
  ColorG 1 =>
    @gray => |&Self([ref g])| g.clone(),
    @rgb =>
    |&Self([ref g])| g.clone(),
    |&Self([ref g])| g.clone(),
    |&Self([ref g])| g.clone(),,,;
  ColorGa 2 =>
    @gray => |&Self([ref g, _])| g.clone(),
    @rgb =>
    |&Self([ref g, _])| g.clone(),
    |&Self([ref g, _])| g.clone(),
    |&Self([ref g, _])| g.clone(),,
    @a => ColorG<T>,
    |&Self([_, ref a])| a.clone(),
    |ColorG::<T>([ref g])| [g.clone(), T::one()],;
  ColorBgr 3 =>
    ,
    @rgb =>
    |&Self([_, _, ref r])| r.clone(),
    |&Self([_, ref g, _])| g.clone(),
    |&Self([ref b, _, _])| b.clone(),,,;
  ColorBgra 4 =>
    ,
    @rgb =>
    |&Self([_, _, ref r, _])| r.clone(),
    |&Self([_, ref g, _, _])| g.clone(),
    |&Self([ref b, _, _, _])| b.clone(),,
    @a => ColorBgr<T>,
    |&Self([_, _, _, ref a])| a.clone(),
    |ColorBgr::<T>([ref b, ref g, ref r])|
      [b.clone(), g.clone(), r.clone(), T::one()],;
  ColorRgb 3 =>
    ,
    @rgb =>
    |&Self([ref r, _, _])| r.clone(),
    |&Self([_, ref g, _])| g.clone(),
    |&Self([_, _, ref b])| b.clone(),,,;
  ColorRgba 4 =>
    ,
    @rgb =>
    |&Self([ref r, _, _, _])| r.clone(),
    |&Self([_, ref g, _, _])| g.clone(),
    |&Self([_, _, ref b, _])| b.clone(),,
    @a => ColorRgb<T>,
    |&Self([_, _, _, ref a])| a.clone(),
    |ColorRgb::<T>([ref r, ref g, ref b])|
      [r.clone(), g.clone(), b.clone(), T::one()],;
  ColorHsv 3 =>
    ,,
    @hsv =>
    |&Self([ref h, _, _])| h.clone(),
    |&Self([_, ref s, _])| s.clone(),
    |&Self([_, _, ref v])| v.clone(),,
    @from => ColorHsva<T>, |hsva: ColorHsva<T>| ColorHsv(get!(hsva.hsv));
  ColorHsva 4 =>
    ,,
    @hsv =>
    |&Self([ref h, _, _, _])| h.clone(),
    |&Self([_, ref s, _, _])| s.clone(),
    |&Self([_, _, ref v, _])| v.clone(),
    @a => ColorHsv<T>,
    |&Self([_, _, _, ref a])| a.clone(),
    |ColorHsv::<T>([ref h, ref s, ref v])|
      [h.clone(), s.clone(), v.clone(), T::one()],;
);
impl<T: FloatCore> ColorHsv<T>
{
  pub fn to_rgb(&self) -> ColorCastResult<ColorRgb<T>>
  {
    hsv_to_rgb(self.0).map(ColorRgb)
  }
}
impl<T: FloatCore + Euclid> ColorRgb<T>
{
  pub fn to_hsv(&self) -> ColorCastResult<ColorHsv<T>>
  {
    rgb_to_hsv(self.0).map(ColorHsv)
  }
}
impl<T: FloatCore> ColorHsva<T>
{
  pub fn to_rgba(&self) -> ColorCastResult<ColorRgba<T>>
  {
    hsv_to_rgb(get!(self.hsv)).map(ColorRgb).map(Into::into)
  }
}
impl<T: FloatCore + Euclid> ColorRgba<T>
{
  pub fn to_hsva(&self) -> ColorCastResult<ColorHsva<T>>
  {
    rgb_to_hsv(get!(self.rgb)).map(ColorHsv).map(Into::into)
  }
}

impl<T: FloatCore> TryFrom<ColorHsv<T>> for ColorRgb<T>
{
  type Error = ColorCastError;

  fn try_from(value: ColorHsv<T>) -> ColorCastResult<Self>
  {
    value.to_rgb()
  }
}
impl<T: FloatCore> TryFrom<ColorHsva<T>> for ColorRgba<T>
{
  type Error = ColorCastError;

  fn try_from(value: ColorHsva<T>) -> ColorCastResult<Self>
  {
    value.to_rgba()
  }
}
impl<T: FloatCore + Euclid> TryFrom<ColorRgba<T>> for ColorHsva<T>
{
  type Error = ColorCastError;

  fn try_from(value: ColorRgba<T>) -> ColorCastResult<Self>
  {
    value.to_hsva()
  }
}
impl<T: FloatCore + Euclid> TryFrom<ColorRgb<T>> for ColorHsv<T>
{
  type Error = ColorCastError;

  fn try_from(value: ColorRgb<T>) -> ColorCastResult<Self>
  {
    value.to_hsv()
  }
}

#[cfg(test)]
mod tests
{
  use crate::{color::ColorG, num::Number};

  fn color<T: Copy, const N: usize>()
  {
    let color: ColorG<T> = ColorG::default();
  }

  #[test]
  fn test_color()
  {
    color::<i32, 3>();
    color::<f32, 3>();
  }
}
