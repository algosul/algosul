pub mod dynamic;
pub mod std;
#[cfg(feature = "utils-tokio")]
pub mod tokio;

use ::std::cmp::Ordering;

pub trait PartialOrdExt: PartialOrd<Self>
{
  fn partial_min_max<'a>(
    &'a self, right: &'a Self,
  ) -> Option<(&'a Self, &'a Self)>;
}

pub trait OrdExt: Ord + PartialOrdExt
{
  fn min_max<'a>(&'a self, right: &'a Self) -> (&'a Self, &'a Self);
}

impl<N: PartialOrd<N>> PartialOrdExt for N
{
  fn partial_min_max<'a>(
    &'a self, right: &'a Self,
  ) -> Option<(&'a Self, &'a Self)>
  {
    self.partial_cmp(right).map(|ord| match ord
    {
      Ordering::Less => (self, right),
      _ => (right, self),
    })
  }
}

impl<N: Ord> OrdExt for N
{
  fn min_max<'a>(&'a self, right: &'a Self) -> (&'a Self, &'a Self)
  {
    if self < right { (self, right) } else { (right, self) }
  }
}
