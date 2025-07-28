#[cfg(feature = "codegen")]
pub mod codegen;
#[cfg(feature = "deps")]
pub mod deps;
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
