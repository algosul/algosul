#[macro_export]
macro_rules! mod_all_os {
    {$vis:vis} => {
        #[cfg(any(doc, target_os = "linux", target_os = "macos", target_os = "windows"))]
        $vis mod desktop;
        #[cfg(any(doc, target_os = "android", target_os = "ios"))]
        $vis mod mobile;
        #[cfg(any(doc, target_os = "linux"))]
        $vis mod linux;
        #[cfg(any(doc, target_os = "macos"))]
        $vis mod macos;
        #[cfg(any(doc, target_os = "windows"))]
        $vis mod windows;
        #[cfg(any(doc, target_os = "ios"))]
        $vis mod ios;
        #[cfg(any(doc, target_os = "android"))]
        $vis mod android;
    };
}
#[macro_export]
macro_rules! use_all_os {
    {$vis:vis, $base:path :os: $($imports:tt)*} => {
        #[cfg(any(doc, target_os = "linux", target_os = "macos", target_os = "windows"))]
        $vis use $base::{desktop::$($imports)*};
        #[cfg(any(doc, target_os = "android", target_os = "ios"))]
        $vis use $base::{mobile::$($imports)*};
        #[cfg(any(doc, target_os = "linux"))]
        $vis use $base::{linux::$($imports)*};
        #[cfg(any(doc, target_os = "macos"))]
        $vis use $base::{macos::$($imports)*};
        #[cfg(any(doc, target_os = "windows"))]
        $vis use $base::{windows::$($imports)*};
        #[cfg(any(doc, target_os = "ios"))]
        $vis use $base::{ios::$($imports)*};
        #[cfg(any(doc, target_os = "android"))]
        $vis use $base::{android::$($imports)*};
    };
    {$vis:vis, $($imports:tt)*} => {
        #[cfg(any(doc, target_os = "linux", target_os = "macos", target_os = "windows"))]
        $vis use self::{desktop::$($imports)*};
        #[cfg(any(doc, target_os = "android", target_os = "ios"))]
        $vis use self::{mobile::$($imports)*};
        #[cfg(any(doc, target_os = "linux"))]
        $vis use self::{linux::$($imports)*};
        #[cfg(any(doc, target_os = "macos"))]
        $vis use self::{macos::$($imports)*};
        #[cfg(any(doc, target_os = "windows"))]
        $vis use self::{windows::$($imports)*};
        #[cfg(any(doc, target_os = "ios"))]
        $vis use self::{ios::$($imports)*};
        #[cfg(any(doc, target_os = "android"))]
        $vis use self::{android::$($imports)*};
    };
}
