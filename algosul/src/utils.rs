pub mod std;
#[cfg(feature = "utils-tokio")]
pub mod tokio;

use log::error;
use rfd::MessageLevel;
use ::std::fmt::Display;

pub trait ResultExt
{
  type Output;
  fn unwarp_or_log(self) -> Self::Output;
  fn unwarp_or_print(self) -> Self::Output;
  fn or_log(self) -> Self;
  fn or_print(self) -> Self;
}
impl<T: Default, E: Display> ResultExt for Result<T, E>
{
  type Output = T;

  fn unwarp_or_log(self) -> Self::Output
  {
    self.unwrap_or_else(|e| {
      error!("{e}");
      T::default()
    })
  }

  fn unwarp_or_print(self) -> Self::Output
  {
    self.unwrap_or_else(|e| {
      println!("{e}");
      T::default()
    })
  }

  fn or_log(self) -> Self
  {
    self.map_err(|e| {
      error!("{e}");
      e
    })
  }

  fn or_print(self) -> Self
  {
    self.map_err(|e| {
      println!("{e}");
      e
    })
  }
}
