use std::{
  borrow::Cow,
  ffi::{OsStr, OsString},
};

pub trait MapToArg<'b>
{
  fn map_to_arg(self) -> Cow<'b, OsStr>;
}
pub trait MapToArgs<'b>
{
  fn map_to_args(self) -> impl Iterator<Item = Cow<'b, OsStr>>;
}

/// [issue: #1958](https://github.com/rust-lang/reference/issues/1958)
pub trait CowExt<'a, T: ?Sized + ToOwned + 'a>
{
  fn map_ref_or_owned<'b, U: ?Sized + ToOwned, B, O>(
    self, b: B, o: O,
  ) -> Cow<'b, U>
  where
    B: FnOnce(&'a T) -> &'b U,
    O: FnOnce(T::Owned) -> U::Owned;
  fn map_to_cow<'b, U: ?Sized + ToOwned, B, O>(self, b: B, o: O) -> Cow<'b, U>
  where
    B: FnOnce(&'a T) -> Cow<'b, U>,
    O: FnOnce(T::Owned) -> Cow<'b, U>;
}
impl<'a, T: ?Sized + ToOwned> CowExt<'a, T> for Cow<'a, T>
{
  #[inline]
  fn map_ref_or_owned<'b, U: ?Sized + ToOwned, B, O>(
    self, b: B, o: O,
  ) -> Cow<'b, U>
  where
    B: FnOnce(&'a T) -> &'b U,
    O: FnOnce(T::Owned) -> U::Owned,
  {
    match self
    {
      Cow::Borrowed(borrow) => Cow::Borrowed(b(borrow)),
      Cow::Owned(owned) => Cow::Owned(o(owned)),
    }
  }

  #[inline]
  fn map_to_cow<'b, U: ?Sized + ToOwned, B, O>(self, b: B, o: O) -> Cow<'b, U>
  where
    B: FnOnce(&'a T) -> Cow<'b, U>,
    O: FnOnce(T::Owned) -> Cow<'b, U>,
  {
    match self
    {
      Cow::Borrowed(borrow) => b(borrow),
      Cow::Owned(owned) => o(owned),
    }
  }
}
impl<'a: 'b, 'b, S> MapToArg<'b> for Cow<'a, S>
where
  S: ?Sized + ToOwned + AsRef<OsStr> + 'a,
  S::Owned: Into<OsString>,
{
  #[inline]
  fn map_to_arg(self) -> Cow<'b, OsStr>
  {
    self.map_ref_or_owned(OsStr::new, Into::into)
  }
}
impl<'a: 'b, 'b, T, S> MapToArgs<'b> for T
where
  S: ?Sized + ToOwned + AsRef<OsStr> + 'a,
  S::Owned: Into<OsString>,
  T: IntoIterator<Item = Cow<'a, S>>,
{
  #[inline]
  fn map_to_args(self) -> impl Iterator<Item = Cow<'b, OsStr>>
  {
    self.into_iter().map(MapToArg::map_to_arg)
  }
}

#[cfg(test)]
mod tests
{
  use std::ffi::OsStr;

  use super::*;
  #[test]
  fn test_demo()
  {
    demo("test");
  }

  fn demo(input: &str)
  {
    let cow: Cow<'_, str> = Cow::Borrowed(input);

    {
      let result: Cow<'_, OsStr> = cow.map_ref_or_owned(OsStr::new, Into::into);

      println!("{:?}", result);
    }
  }
}
