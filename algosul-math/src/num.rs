use std::ops::Range;

use num_traits::{float::FloatCore, Bounded, FloatConst, NumOps};

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
pub trait NumNormalizeAngle<N = Self>
{
  /// # Iuput
  /// `-max..max`
  /// # Result
  /// `0..max`
  fn normalize_cyclic(self, max: N) -> N;
  /// # Iuput
  /// `-360..360`
  /// # Result
  /// `0..360`
  fn normalize_degrees(self) -> N
  where N: From<u16>;
  /// # Input
  /// `-2 * PI..2 * PI`
  /// # Result
  /// `0..2π`
  fn normalize_radians(self) -> N
  where N: FloatConst;
  /// # Iuput
  /// `-1..1`
  /// # Result
  /// `0..1`
  fn normalize_unit(self) -> N
  where N: From<u8>;
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
impl<N: NumOps + PartialOrd + Clone + From<u8>> NumNormalizeAngle<N> for N
{
  fn normalize_cyclic(self, max: N) -> N
  {
    let temp = self % max.clone();
    if temp < N::from(0) { temp + max } else { temp }
  }

  fn normalize_degrees(self) -> N
  where N: From<u16>
  {
    self.normalize_cyclic(<N as From<u16>>::from(360))
  }

  fn normalize_radians(self) -> N
  where N: FloatConst
  {
    self.normalize_cyclic(N::PI() + N::PI())
  }

  fn normalize_unit(self) -> N
  {
    self.normalize_cyclic(<N as From<_>>::from(1))
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
    #[cfg(feature = "unstable")]
    {
      self.try_map(|x| x.try_remap(from.clone(), to.clone()))
    }
    #[cfg(not(feature = "unstable"))]
    {
      use std::mem::MaybeUninit;
      let mut out: [MaybeUninit<U>; L] =
        unsafe { MaybeUninit::uninit().assume_init() };
      for (i, x) in self.into_iter().enumerate()
      {
        let val = x.try_remap(from.clone(), to.clone())?;
        out[i] = MaybeUninit::new(val);
      }
      Ok(unsafe { std::mem::transmute_copy::<_, [U; L]>(&out) })
    }
  }
}
