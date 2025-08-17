use strum::Display;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Display)]
pub enum LanguageType
{
  #[strum(to_string = "C")]
  C,
  #[strum(to_string = "C++")]
  CPP,
  #[strum(to_string = "C#")]
  CSharp,
  #[strum(to_string = "Rust")]
  Rust,
}
