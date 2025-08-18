use std::any::Any;
pub trait AsAny
{
  fn as_any(&self) -> &dyn Any;
}
pub trait DynPartialEq
{
  fn dyn_eq(&self, other: &dyn AsAny) -> bool;
  fn dyn_ne(&self, other: &dyn AsAny) -> bool
  {
    !self.dyn_eq(other)
  }
}
pub trait DynEq: DynPartialEq {}

impl<T: Any> AsAny for T
{
  fn as_any(&self) -> &dyn Any
  {
    self
  }
}
impl<T: PartialEq + 'static> DynPartialEq for T
{
  fn dyn_eq(&self, other: &dyn AsAny) -> bool
  {
    match other.as_any().downcast_ref::<T>()
    {
      Some(other_t) => self.eq(other_t),
      None => false,
    }
  }

  fn dyn_ne(&self, other: &dyn AsAny) -> bool
  {
    match other.as_any().downcast_ref::<T>()
    {
      Some(other_t) => self.ne(other_t),
      None => true,
    }
  }
}
impl<T: Eq + 'static> DynEq for T {}
