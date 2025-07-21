use std::fmt::{Debug, Display, Formatter};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
#[derive(Debug)]
pub enum Error {
    FormatParameterMismatch(String),
    FormatError(strfmt::FmtError),
}
impl From<strfmt::FmtError> for Error {
    fn from(value: strfmt::FmtError) -> Self { Self::FormatError(value) }
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::FormatParameterMismatch(message) => f.write_fmt(
                format_args!("Format parameter mismatch: {message}"),
            ),
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
pub type Result<T> = std::result::Result<T, Error>;
pub trait I18n {
    type DataType: I18nData<I18n = Self>;
    fn i18n_from_toml(
        toml: &str,
    ) -> std::result::Result<Self::DataType, toml::de::Error> {
        Self::DataType::deserialize(toml::Deserializer::new(toml))
    }
    fn to_data(&self) -> Self::DataType;
    fn into_data(self) -> Self::DataType;
    fn to_toml(&self) -> std::result::Result<String, toml::ser::Error> {
        self.to_data().to_toml()
    }
    fn into_toml(self) -> std::result::Result<String, toml::ser::Error>
    where Self: Sized {
        self.into_data().to_toml()
    }
}
pub trait I18nData: Debug + Serialize + DeserializeOwned {
    type I18n: I18n<DataType = Self>;
    fn check(&self) -> Result<()>;
    fn to_toml(&self) -> std::result::Result<String, toml::ser::Error> {
        let mut buffer = String::new();
        self.serialize(toml::ser::Serializer::new(&mut buffer))?;
        Ok(buffer)
    }
}
