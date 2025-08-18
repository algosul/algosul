pub use algosul_core::{arg, args, cow, cows};
#[cfg(feature = "macros")]
pub use algosul_derive::from_dir;

#[macro_export]
macro_rules! cfg_feature {
  (if $features:literal $true:block else $false:block) => {{
    #[cfg(feature = $features)]
    $true
    #[cfg(not(feature = $features))]
    $false
  }};
}
