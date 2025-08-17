use std::{
  cmp::{
    Ordering,
    Ordering::{Greater, Less},
  },
  ops::Range,
};

use algosul_derive::get;
use num_traits::{float::FloatCore, Bounded, Euclid, Num, One};
use thiserror::Error;

use crate::{
  num::{NumConst, NumDegConst, NumLerp, NumNormalize, NumPercent, NumsRemap},
  Remap,
};

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
  #[error("{0} is not a integer")]
  NotAInteger(&'static str),
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
/// # Range
/// `0.0..=1.0`
pub fn hsv_to_rgb<T: FloatCore + NumNormalize + NumConst>(
  [h, s, v]: [T; 3],
) -> ColorCastResult<[T; 3]>
{
  let h = h.normalize_01(); // h: 0.0..1.0
  let c = v * s;
  let h_prime = h * T::SIX; // h: 0.0..6.0
  let x = c * (T::ONE - ((h_prime % T::TWO) - T::ONE).abs());
  let h_prime = h_prime.floor(); // h: 0..6
  let (r1, g1, b1) = if T::ZERO == h_prime
  {
    (c, x, T::ZERO)
  }
  else if T::ONE == h_prime
  {
    (x, c, T::ZERO)
  }
  else if T::TWO == h_prime
  {
    (T::ZERO, c, x)
  }
  else if T::THREE == h_prime
  {
    (T::ZERO, x, c)
  }
  else if T::FOUR == h_prime
  {
    (x, T::ZERO, c)
  }
  else if T::FIVE == h_prime
  {
    (c, T::ZERO, x)
  }
  else
  {
    return Err(ColorCastError::NotAInteger(stringify!((v * s * 6).floor())));
  };
  let m = v - c;
  Ok([r1 + m, g1 + m, b1 + m])
}
/// # Range
/// `0.0..=1.0`
pub fn rgb_to_hsv<T: FloatCore + NumDegConst + Euclid>(
  [r, g, b]: [T; 3],
) -> ColorCastResult<[T; 3]>
{
  use Ordering::*;
  match (r.partial_cmp(g), g.partial_cmp(b))
  {
    // r > g > b
    (Some(Greater), Some(Greater)) =>
    {
      let delta = max - min;
      T::DEG_60 * ((g - b) / delta).rem_euclid(&T::SIX)
    }
    // r > g, g <= b
    (Some(Greater), Some(_)) =>
    {
      let min = g.min(b);
      let delta = max - min;
    }
    // r < g < b
    (Some(Less), Some(Less)) =>
    {
      let delta = max - min;
      T::DEG_60 * ((r - g) / delta + T::FOUR)
    }
    // r < g, g > b
    (Some(Less), Some(Greater)) =>
    {
      let delta = max - min;
      T::DEG_60 * ((b - r) / delta + T::TWO)
    }
    (None, None) =>
    {}
  }
  let max = r.max(g).max(b);
  let min = r.min(g).min(b);
  let delta = max - min;
  let mut h = if delta == T::ZERO
  {
    T::ZERO
  }
  else if max == r
  {
    T::DEG_60 * ((g - b) / delta).rem_euclid(&T::SIX)
  }
  else if max == g
  {
    T::DEG_60 * ((b - r) / delta + T::TWO)
  }
  else
  {
    T::DEG_60 * ((r - g) / delta + T::FOUR)
  };
  if h < T::ZERO
  {
    h = h + T::DEG_360;
    unimplemented!();
  }
  let s = if max == T::ZERO { T::ZERO } else { delta / max };
  Ok([h, s, max])
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
impl<T: FloatCore + NumConst> ColorHsv<T>
{
  pub fn to_rgb(&self) -> ColorCastResult<ColorRgb<T>>
  {
    hsv_to_rgb(self.0).map(ColorRgb)
  }
}
impl<T: FloatCore + Euclid + NumDegConst> ColorRgb<T>
{
  pub fn to_hsv(&self) -> ColorCastResult<ColorHsv<T>>
  {
    rgb_to_hsv(self.0).map(ColorHsv)
  }
}
impl<T: FloatCore + NumConst> ColorHsva<T>
{
  pub fn to_rgba(&self) -> ColorCastResult<ColorRgba<T>>
  {
    hsv_to_rgb(get!(self.hsv)).map(ColorRgb).map(Into::into)
  }
}
impl<T: FloatCore + Euclid + NumDegConst> ColorRgba<T>
{
  pub fn to_hsva(&self) -> ColorCastResult<ColorHsva<T>>
  {
    rgb_to_hsv(get!(self.rgb)).map(ColorHsv).map(Into::into)
  }
}

impl<T: FloatCore + NumConst> TryFrom<ColorHsv<T>> for ColorRgb<T>
{
  type Error = ColorCastError;

  fn try_from(value: ColorHsv<T>) -> ColorCastResult<Self>
  {
    value.to_rgb()
  }
}
impl<T: FloatCore + NumConst> TryFrom<ColorHsva<T>> for ColorRgba<T>
{
  type Error = ColorCastError;

  fn try_from(value: ColorHsva<T>) -> ColorCastResult<Self>
  {
    value.to_rgba()
  }
}
impl<T: FloatCore + Euclid + NumDegConst> TryFrom<ColorRgba<T>> for ColorHsva<T>
{
  type Error = ColorCastError;

  fn try_from(value: ColorRgba<T>) -> ColorCastResult<Self>
  {
    value.to_hsva()
  }
}
impl<T: FloatCore + Euclid + NumDegConst> TryFrom<ColorRgb<T>> for ColorHsv<T>
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
  use crate::color::{
    Color,
    ColorG,
    ColorGa,
    ColorHsv,
    ColorHsva,
    ColorRgb,
    ColorRgba,
  };

  #[test]
  fn test_rgb_to_hsv_and_back()
  {
    let rgb = ColorRgb([0.2f32, 0.4, 0.6]);
    let hsv = rgb.to_hsv().expect("RGB to HSV failed");
    let rgb2 = hsv.to_rgb().expect("HSV to RGB failed");
    for (a, b) in rgb.0.iter().zip(rgb2.0.iter())
    {
      assert!(
        (a - b).abs() < 1e-5,
        "RGB->HSV->RGB not lossless: {} vs {}",
        a,
        b
      );
    }
  }

  #[test]
  fn test_hsv_to_rgb_and_back()
  {
    let hsv = ColorHsv([0.6f32, 0.5, 0.8]);
    let rgb = hsv.to_rgb().expect("HSV to RGB failed");
    let hsv2 = rgb.to_hsv().expect("RGB to HSV failed");
    for (a, b) in hsv.0.iter().zip(hsv2.0.iter())
    {
      assert!(
        (a - b).abs() < 1e-5,
        "HSV->RGB->HSV not lossless: {} vs {}",
        a,
        b
      );
    }
  }

  #[test]
  fn test_color_enum_variants()
  {
    let g = Color::G(ColorG([0.5f32]));
    let ga = Color::Ga(ColorGa([0.5f32, 1.0]));
    let rgb = Color::Rgb(ColorRgb([0.1f32, 0.2, 0.3]));
    let rgba = Color::Rgba(ColorRgba([0.1f32, 0.2, 0.3, 1.0]));
    let hsv = Color::Hsv(ColorHsv([0.7f32, 0.8, 0.9]));
    let hsva = Color::Hsva(ColorHsva([0.7f32, 0.8, 0.9, 1.0]));
    assert_eq!(g, g.clone());
    assert_eq!(ga, ga.clone());
    assert_eq!(rgb, rgb.clone());
    assert_eq!(rgba, rgba.clone());
    assert_eq!(hsv, hsv.clone());
    assert_eq!(hsva, hsva.clone());
  }
}
