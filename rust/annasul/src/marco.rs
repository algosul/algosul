// Copyright (c) 2025 air (https://yuanair.github.io).
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, version 3 of the License only.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

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
