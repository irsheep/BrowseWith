use std::path::{ Path };

#[cfg(target_os= "linux")]
mod linux;

#[cfg(target_os= "windows")]
mod windows;

mod shared;

pub fn set_default_browser(system_wide:bool) {
  #[cfg(target_os= "linux")]
  linux::set_default_browser(system_wide);

  #[cfg(target_os= "windows")]
  windows::set_default_browser(system_wide);
}

pub fn is_privileged_user() -> bool {
  #[cfg(target_os= "linux")]
  return linux::is_privileged_user();

  #[cfg(target_os= "windows")]
  return windows::is_privileged_user();
}
