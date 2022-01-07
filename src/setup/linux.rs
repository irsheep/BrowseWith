use std::{ include_bytes };
use std::path::{ Path, PathBuf };
use std::fs::{ write };

use nix::unistd::{ Uid, getuid };

use gtk::glib::{ Bytes };
use gtk::gdk_pixbuf::{ Pixbuf };

use super::shared;

pub fn set_default_browser(system_wide:bool) {
  let mut default_applications_buf:PathBuf;

  let mut list_buf:PathBuf;
  let list_path:&Path;

  let mut desktop_buf:PathBuf;
  let desktop_path:&Path;

  let default_list_raw:&[u8];
  let desktop_file_raw:&[u8];

  // Load and store resource files
  default_list_raw = include_bytes!("../../resources/defaults.list");
  desktop_file_raw = include_bytes!("../../resources/browsewith.desktop");

  if system_wide {
    default_applications_buf = PathBuf::from("/usr/share/applications");
  } else {
    default_applications_buf = dirs::home_dir().unwrap();
    default_applications_buf.push(".local");
    default_applications_buf.push("share");
    default_applications_buf.push("applications");
  }

  // Check for the defaults.list file, modify if exists or create a new one
  list_buf = default_applications_buf.clone();
  list_buf.push("defaults.list");
  list_path = list_buf.as_path();
  if list_path.is_file() {
    println!("Checking {}", list_path.to_str().unwrap());
    shared::modify_default_list(list_path);
  } else {
    match write(list_path, Bytes::from(&default_list_raw[..])) {
      Ok(..) => { println!("Created {}", list_path.to_str().unwrap()); },
      Err(..) => { println!("Failed to create {}", list_path.to_str().unwrap()); }
    }
  }

  // Create browsewith.desktop if does not exist
  desktop_buf = default_applications_buf.clone();
  desktop_buf.push("browsewith.desktop");
  desktop_path = desktop_buf.as_path();
  if !desktop_path.is_file() {
    match write(desktop_path, Bytes::from(&desktop_file_raw[..])) {
      Ok(..) => { println!("Created {}", desktop_path.to_str().unwrap()); },
      Err(..) => { println!("Failed to create {}", desktop_path.to_str().unwrap()); }
    }
  }

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