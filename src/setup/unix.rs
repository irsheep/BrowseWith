use std::{ include_bytes };
use std::path::{ Path, PathBuf };
use std::fs::{ write };

use nix::unistd::{ Uid, getuid };

use gtk::glib::{ Bytes };
use gtk::gdk_pixbuf::{ Pixbuf };

#[allow(dead_code)]
pub fn set_default_browser(_system_wide:bool) {
  println!("This feature isn't supported in FreeBSD, to set browsewith as the default browser you need to use the tools provided by the display manager installed on your system\nNo changes were made.");
}

pub fn is_privileged_user() -> bool {
  let uid:Uid;
  uid = getuid();
  return uid.is_root();
}

pub fn load_icon() {
  let mut home_dir_buf:PathBuf;
  let icon_file_path:&Path;
  let icon_file:Pixbuf;
  let icon_raw:&[u8];
  let icon_bytes:Bytes;

  // Load the icon file as '[u8]' at compile time
  icon_raw = include_bytes!("../../resources/browsewith.ico");
  icon_bytes = Bytes::from(&icon_raw[..]);

  // Create the icon file in the configuration directory, if it doesn't exist
  home_dir_buf = dirs::home_dir().unwrap();
  home_dir_buf.push(".browsewith/browsewith.ico");
  icon_file_path = home_dir_buf.as_path();
  if !icon_file_path.is_file() {
    match write(icon_file_path, icon_bytes) {
      Ok(..) => {},
      Err(..) => println!("Failed to create icon file")
    }
  }
  
  // Confirm that the icon was successfully created before loading
  if icon_file_path.is_file() {
    // Assign the icon to the main window
    icon_file = Pixbuf::from_file(icon_file_path).unwrap();
    gtk::Window::set_default_icon(&icon_file);
  }
}