#![feature(extend_one)]
#![feature(impl_trait_in_assoc_type)]

#[cfg(feature = "codegen")]
pub mod codegen;
#[cfg(feature = "deps")]
pub mod deps;
pub mod utils;
#[macro_export]
macro_rules! mod_all_os {
    {$vis:vis} => {
        #[cfg(unix)]
        $vis mod unix;
        #[cfg(target_os = "linux")]
        $vis mod linux;
        #[cfg(target_os = "macos")]
        $vis mod macos;
        #[cfg(windows)]
        $vis mod windows;
        #[cfg(target_os = "ios")]
        $vis mod ios;
        #[cfg(target_os = "android")]
        $vis mod android;
    };
}
#[macro_export]
macro_rules! args {

  // condition pat
  (
    $args:ident: if let $pat:pat = $ok:expr =>
    $(@$oper:ident)? $arg:expr $(; $($rest:tt)*)?
  ) => {
    if let $pat = $ok {
      $crate::args!($args: $(@$oper)? $arg);
    }
    $($crate::args!($args: $($rest)*);)?
  };

  // condition
  (
    $args:ident: if $condition:expr =>
    $(@$oper:ident)? $arg:expr $(; $($rest:tt)*)?
  ) => {
    if $condition {
      $crate::args!($args: $(@$oper)? $arg);
    }
    $($crate::args!($args: $($rest)*);)?
  };

  // pat
  (
    $args:ident: let $pat:pat = $ok:expr =>
    $(@$oper:ident)? $arg:expr $(; $($rest:tt)*)?
  ) => {
    {
      let $pat = $ok;
      $crate::args!($args: $(@$oper)? $arg);
    }
    $($crate::args!($args: $($rest)*);)?
  };

  // simple
  ($args:ident: $(@$oper:ident)? $arg:expr $(; $($rest:tt)*)?) => {
    $crate::arg!($args: $(@$oper)? $arg);
    $($crate::args!($args: $($rest)*);)?
  };

  // empty
  ($args:ident: ) => {};
}

#[macro_export]
macro_rules! arg {
    // push
  ($args:ident: @push $arg:expr) => {
    $args.push($arg)
  };

  // map push
  ($args:ident: @map $arg:expr) => {
    $args.push($crate::utils::std::MapToArg::map_to_arg($arg))
  };

  // extend
  ($args:ident: @extend $arg:expr) => {
    $args.extend($arg)
  };

  // map extend
  ($args:ident: @maps $arg:expr) => {
    $args.extend($crate::utils::std::MapToArgs::map_to_args($arg))
  };

  // default
  ($args:ident: $arg:expr) => {
    $crate::arg!($args: @maps $arg)
  };
}
