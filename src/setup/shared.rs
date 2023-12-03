use std::{ include_bytes };
use std::path::{ PathBuf };
use std::fs::{ write };

use gtk::gdk_pixbuf::{ Pixbuf };
use gtk::glib::{ Bytes };

use crate::config;

// This function can be removed from unix and windows files as they are identical
pub fn _load_icon() {
  let icon_file_path:PathBuf;
  let icon_file:Pixbuf;
  let icon_raw:&[u8];
  let icon_bytes:Bytes;

  // Load the icon file as '[u8]' at compile time
  icon_raw = include_bytes!("../../resources/browsewith.ico");
  icon_bytes = Bytes::from(&icon_raw[..]);

  // Create the icon file in the configuration directory, if it doesn't exist
  icon_file_path = config::get_icon_file(false);
  if !icon_file_path.is_file() {
    match write(&icon_file_path, icon_bytes) {
      Ok(..) => {},
      Err(..) => println!("Failed to create icon file {}", &icon_file_path)
    }
  }

  // Confirm that the icon was successfully created before loading
  if icon_file_path.is_file() {
    // Assign the icon to the main window
    icon_file = Pixbuf::from_file(icon_file_path).unwrap();
    gtk::Window::set_default_icon(&icon_file);
  }
}