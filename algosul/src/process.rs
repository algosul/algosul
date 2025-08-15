use std::{cmp::Ordering, fmt::Debug, hash::Hash};
pub trait Process {
  type Error: std::error::Error;
  type Output;
  type Status;
  async fn run(self) -> Result<Self::Output, Self::Error>;
  fn on_status_changed(
    &mut self,
    f: impl Fn(&Self::Status) -> Result<(), Self::Error> + Send + 'static,
  ) -> Result<(), Self::Error>;
}
pub struct StatusObserver<T, E> {
  inner:      T,
  on_changed: Option<Box<dyn Fn(&T) -> Result<(), E> + Send>>,
}
impl<T: Default, E> Default for StatusObserver<T, E> {
  fn default() -> Self { Self { inner: T::default(), on_changed: None } }
}
impl<T: Debug, E> Debug for StatusObserver<T, E> {
  fn fmt(
    &self, f: &mut std::fmt::Formatter<'_>,
  ) -> Result<(), std::fmt::Error> {
    self.inner.fmt(f)
  }
}
impl<T: PartialEq, E> PartialEq for StatusObserver<T, E> {
  fn eq(&self, other: &Self) -> bool { self.inner == other.inner }
}
impl<T: Eq, E> Eq for StatusObserver<T, E> {}
impl<T: Hash, E> Hash for StatusObserver<T, E> {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.inner.hash(state);
  }
}
impl<T: PartialOrd, E> PartialOrd for StatusObserver<T, E> {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    self.inner.partial_cmp(&other.inner)
  }
}
impl<T: Ord, E> Ord for StatusObserver<T, E> {
  fn cmp(&self, other: &Self) -> Ordering { self.inner.cmp(&other.inner) }
}
impl<T, E> StatusObserver<T, E> {
  pub fn new(inner: T) -> Self { Self { inner, on_changed: None } }

  pub fn on_changed<F>(&mut self, f: F)
  where F: Fn(&T) -> Result<(), E> + Send + 'static {
    self.on_changed = Some(Box::new(f));
  }

  pub fn change(&mut self, status: T) -> Result<(), E> {
    self.inner = status;
    if let Some(ref on_changed) = self.on_changed {
      on_changed(&self.inner)?;
    }
    Ok(())
  }
}
impl<T, E> std::ops::Deref for StatusObserver<T, E> {
  type Target = T;

  fn deref(&self) -> &Self::Target { &self.inner }
}
