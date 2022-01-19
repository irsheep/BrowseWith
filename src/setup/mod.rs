#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_family = "unix")]
mod unix;

#[cfg(target_family = "windows")]
mod windows;

mod shared;

pub fn set_default_browser(system_wide:bool) {
  #[cfg(target_os = "linux")]
  linux::set_default_browser(system_wide);

  #[cfg(all(target_family = "unix", not(target_os="linux")))]
  unix::set_default_browser(system_wide);

  #[cfg(target_family = "windows")]
  windows::set_default_browser(system_wide);
}

pub fn is_privileged_user() -> bool {
  #[cfg(target_family = "unix")]
  return unix::is_privileged_user();

  #[cfg(target_family = "windows")]
  return windows::is_privileged_user();
}

pub fn load_icon() {
  #[cfg(target_family = "unix")]
  unix::load_icon();

  #[cfg(target_family = "windows")]
  windows::load_icon();

}