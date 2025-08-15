use std::ops::Range;

use num_traits::{float::FloatCore, Bounded, Num};
pub trait NumPercent<N>
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
pub trait NumLerp<N: FloatCore>
{
  fn lerp_in(self, range: Range<N>) -> N;
  fn lerp(range: Range<N>, alpha: N) -> N;
}
pub trait NumRemap<N>
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
    let mut arr: [std::mem::MaybeUninit<U>; L] =
      unsafe { std::mem::MaybeUninit::uninit().assume_init() };
    for (elem, n) in arr.iter_mut().zip(self.into_iter())
    {
      *elem = std::mem::MaybeUninit::new(
        <U as From<N>>::from(n).remap(from.clone(), to.clone()),
      );
    }
    unsafe { std::mem::transmute_copy::<_, [U; L]>(&arr) }
  }

  fn try_remap<U: FloatCore + NumRemap<U> + TryFrom<N>>(
    self, from: Range<U>, to: Range<U>,
  ) -> Result<[U; L], U::Error>
  {
    let mut arr: [std::mem::MaybeUninit<U>; L] =
      unsafe { std::mem::MaybeUninit::uninit().assume_init() };
    for (elem, n) in arr.iter_mut().zip(self.into_iter())
    {
      *elem = std::mem::MaybeUninit::new(
        <U as TryFrom<N>>::try_from(n)?.remap(from.clone(), to.clone()),
      );
    }
    Ok(unsafe { std::mem::transmute_copy::<_, [U; L]>(&arr) })
  }
}
