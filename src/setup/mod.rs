#[cfg(target_family = "unix")]
mod unix;

#[cfg(target_family = "windows")]
mod windows;

pub fn install() {
  #[cfg(target_family = "unix")]
  unix::install();
  
  #[cfg(target_family = "windows")]
  windows::install();
}

pub fn uninstall() {
  #[cfg(target_family = "unix")]
  unix::uninstall();

  #[cfg(target_family = "windows")]
  windows::uninstall();
}

pub fn set_default_browser() {
  #[cfg(target_family = "unix")]
  unix::set_default_browser();

  #[cfg(target_family = "windows")]
  windows::set_default_browser();
}

pub fn list_default_applications() {
  #[cfg(target_family = "unix")]
  unix::list_default_applications();

  #[cfg(target_family = "windows")]
  windows::list_default_applications();
}

pub fn load_icon() {
  #[cfg(target_family = "unix")]
  unix::load_icon();

  #[cfg(target_family = "windows")]
  windows::load_icon();
}