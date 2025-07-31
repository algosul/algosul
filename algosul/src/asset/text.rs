use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};
use strfmt::FmtError;
#[derive(Debug)]
pub enum Error {
  /// format parameters mismatch
  FormatParameterMismatch(String),
  /// usually format string error
  FormatError(FmtError),
}
pub type Result<T> = std::result::Result<T, Error>;
pub type Text = String;
#[derive(
  Default,
  Debug,
  Clone,
  Eq,
  PartialEq,
  Ord,
  PartialOrd,
  Hash,
  Serialize,
  Deserialize,
)]
pub struct FormatText {
  fmt: Text,
}
// impl FormatText {
//     fn invalid_key_error(&self, key: &str) -> Error {
//         Error::FormatParameterMismatch(format!("Invalid key: {}", key))
//     }
//
//     fn invalid_strfmt__key_error(&self, key: &str) -> FmtError {
//         FmtError::KeyError(format!("Invalid key: {}", key))
//     }
//
//     pub fn fmt_keys(&self) -> strfmt::Result<HashSet<String>> {
//         let mut buffer = HashSet::new();
//         strfmt_map(&self.fmt, |fmt: strfmt::Formatter| {
//             buffer.insert(fmt.key.to_string());
//             Ok(())
//         })?;
//         Ok(buffer)
//     }
//
//     pub fn fmt_check_only(&self) -> Result<()> {
//         let keys = self.fmt_keys()?;
//         if keys != Args.keys {
//             return Err(Error::FormatParameterMismatch(format!(
//                 "must use [{}], but use {:?}",
//                 Args.keys.iter().join(", "),
//                 keys
//             )));
//         };
//         Ok(())
//     }
//
//     pub fn fmt_without_check(
//         &self, vars: &HashMap<String, String>,
//     ) -> Result<Text> {
//         strfmt_map(&self.fmt, |mut fmt: strfmt::Formatter| {
//             let v = vars
//                 .get(fmt.key)
//                 .ok_or_else(|| self.invalid_strfmt__key_error(fmt.key))?;
//             v.display_str(&mut fmt)
//         })
//         .map_err(Error::FormatError)
//     }
//
//     pub fn fmt_with_check(
//         &self, vars: &HashMap<String, String>,
//     ) -> Result<Text> {
//         let mut buffer = HashSet::new();
//         let result = strfmt_map(&self.fmt, |mut fmt: strfmt::Formatter| {
//             buffer.insert(fmt.key.to_string());
//             let v = vars
//                 .get(fmt.key)
//                 .ok_or_else(|| self.invalid_strfmt__key_error(fmt.key))?;
//             v.display_str(&mut fmt)
//         })?;
//         let keys = self.fmt_keys()?;
//         if keys != Args.keys {
//             return Err(Error::FormatParameterMismatch(format!(
//                 "must use [{}], but use {:?}",
//                 Args.keys.iter().join(", "),
//                 keys
//             )));
//         };
//         Ok(result)
//     }
// }
impl Display for Error {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Error::FormatParameterMismatch(message) => {
        f.write_fmt(format_args!("Format parameter mismatch: {message}"))
      }
      Error::FormatError(err) => {
        f.write_fmt(format_args!("format error: {err}"))
      }
    }
  }
}
impl std::error::Error for Error {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    match self {
      Error::FormatParameterMismatch(_) => None,
      Error::FormatError(err) => Some(err),
    }
  }
}
impl From<strfmt::FmtError> for Error {
  fn from(value: strfmt::FmtError) -> Self { Self::FormatError(value) }
}
