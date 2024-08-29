// use dirs;
use std::{ include_bytes };
use std::path::{ Path, PathBuf };
use std::fs::{ write, create_dir_all, copy };
use std::process::{ Command, Output };
use std::io::{ Error };
use std::slice::Iter;
use std::ops::Range;

use nix::unistd::{ Uid, getuid };

use gtk::glib::{ Bytes };
use gtk::gdk_pixbuf::{ Pixbuf };
use bitflags::bitflags;
use ini::{ Ini, Properties };

use crate::config;

bitflags! {
  struct InstalledStatus:u8 {
    const HAS_SYSTEM_EXECUTABLE = 0x01;
    const HAS_USER_EXECUTABLE = 0x02;
    const HAS_SYSTEM_DOTDESKTOP = 0x04;
    const HAS_USER_DOTDESKTOP = 0x08;
    const HAS_SYSTEM_ICON = 0x10;
    const HAS_USER_ICON = 0x20;
    const HAS_CONFIG = 0x40;
    const EXECUTABLE_OK = Self::HAS_SYSTEM_EXECUTABLE.bits() | Self::HAS_USER_EXECUTABLE.bits();
    const DOTDESKTOP_OK = Self::HAS_SYSTEM_DOTDESKTOP.bits() | Self::HAS_USER_DOTDESKTOP.bits();
    const ICON_OK = Self::HAS_SYSTEM_ICON.bits() | Self::HAS_USER_ICON.bits();
  }
}

struct DefaultApplications {
  pub browser: String,
  pub http: String,
  pub https: String
}

enum DefaultApplicationType {
  Web,
  Http,
  Https
}

pub fn install() {
  save_browsewith();
  // save_icon();
  save_icons();
  save_dotdesktop();
}

pub fn uninstall() {
  remove_browsewith();
  unregister_browsewith();
}

pub fn set_default_browser() {
  let apps:DefaultApplications = get_default_applications();
  let install_status:InstalledStatus = check_installation();

  if check_desktop_configuration_tool() &&
     install_status.intersects(InstalledStatus::EXECUTABLE_OK) &&
     install_status.intersects(InstalledStatus::DOTDESKTOP_OK) &&
     install_status.intersects(InstalledStatus::ICON_OK)
  {
    // Web and HTTP seem to be linked, but we change both just in case
    if apps.browser != config::BW_DOTDESKTOP { change_default_app(DefaultApplicationType::Web); }
    if apps.http != config::BW_DOTDESKTOP { change_default_app(DefaultApplicationType::Http); }
    if apps.https != config::BW_DOTDESKTOP { change_default_app(DefaultApplicationType::Https); }
  } else if !check_desktop_configuration_tool() {
    println!("This application requries '{}' to be installed on this system, in order to make the required changes", config::OS_CONFIG_TOOL);
  } else {
    println!("One or more required files are missing, please run 'browsewith --install' to copy the required files")
  }
}

pub fn list_default_applications() {
  let apps:DefaultApplications = get_default_applications();

  println!("Defaults:\n Web Browser: {}\n Protocol handlers:\n   http: {}\n   https: {}\n",
    apps.browser, apps.http, apps.https
  );

  check_application_files();
}

pub fn check_application_files() {
  let install_status:InstalledStatus;
  let install_status_system:&str;
  let install_status_user:&str;

  install_status = check_installation();
  install_status_system = if
      install_status.intersects(InstalledStatus::HAS_SYSTEM_EXECUTABLE) &&
      install_status.intersects(InstalledStatus::HAS_SYSTEM_DOTDESKTOP) &&
      install_status.intersects(InstalledStatus::HAS_SYSTEM_ICON)
      { "Ok" }
    else if
      install_status.intersects(InstalledStatus::HAS_SYSTEM_EXECUTABLE) ||
      install_status.intersects(InstalledStatus::HAS_SYSTEM_DOTDESKTOP) ||
      install_status.intersects(InstalledStatus::HAS_SYSTEM_ICON)
      { "Incomplete" }
    else { "Missing" };
  install_status_user = if
      install_status.intersects(InstalledStatus::HAS_USER_EXECUTABLE) &&
      install_status.intersects(InstalledStatus::HAS_USER_DOTDESKTOP) &&
      install_status.intersects(InstalledStatus::HAS_USER_ICON)
      { "Ok" }
    else if
      install_status.intersects(InstalledStatus::HAS_USER_EXECUTABLE) ||
      install_status.intersects(InstalledStatus::HAS_USER_DOTDESKTOP) ||
      install_status.intersects(InstalledStatus::HAS_USER_ICON)
      { "Incomplete" }
    else { "Missing" };

  //Executable, Icon, .desktop
  println!("System configuration [{}]:", install_status_system);
  print_status(config::get_executable_path(true), config::BW_EXECUTABLE, "Executable\t");
  print_status(config::get_dotdesktop_path(true), config::BW_DOTDESKTOP, ".desktop\t");
  print_status(config::get_icon_path(true), config::BW_ICON_APPLICATION, "Icon\t\t");

  if !is_privileged_user() {
    println!("User configuration [{}]:", install_status_user);
    print_status(config::get_executable_path(false), config::BW_EXECUTABLE, "Executable\t");
    print_status(config::get_dotdesktop_path(false), config::BW_DOTDESKTOP, ".desktop\t");
    print_status(config::get_icon_path(false), config::BW_ICON_APPLICATION, "Icon\t\t");

    print_status(config::get_configuration_path(), config::BW_CONFIG, "Configuration\t");
  }
}

fn print_status(path:PathBuf, filename:&str, text:&str) {
  let mut file:PathBuf;
  let mut status:&str;

  file = PathBuf::from(path.to_str().unwrap());
  file.push(filename);
  status = "NOT_FOUND";

  if file.is_file() {
    status = "OK"
  }
  println!("  {} [{}] {}", text, status, file.to_str().unwrap());
}

pub fn is_privileged_user() -> bool {
  let uid:Uid;
  uid = getuid();
  return uid.is_root();
}

pub fn load_icon() {
  let icon_path:PathBuf;
  let icon_file_path:PathBuf;
  let icon_file:Pixbuf;
  let icon_raw:&[u8];
  let icon_bytes:Bytes;

  // Load the icon file as '[u8]' at compile time
  icon_raw = include_bytes!("../../resources/browsewith.ico");
  icon_bytes = Bytes::from(&icon_raw[..]);

  // Create directory for icons if it doesn't exist
  icon_path = config::get_icon_path(false);
  if !icon_path.exists() {
    create_dir_all(icon_path).unwrap();
  }

  // Create the icon file in the configuration directory, if it doesn't exist
  icon_file_path = config::get_icon_file(false);
  if !icon_file_path.is_file() {
    match write(&icon_file_path, icon_bytes) {
      Ok(..) => {},
      Err(..) => println!("Failed to create icon file {}", &icon_file_path.display())
    }
  }

  // Confirm that the icon was successfully created before loading
  if icon_file_path.is_file() {
    // Assign the icon to the main window
    icon_file = Pixbuf::from_file(icon_file_path).unwrap();
    gtk::Window::set_default_icon(&icon_file);
  }
}

fn get_default_applications() -> DefaultApplications {
  let mut result:Result<Output, Error>;
  let mut default_applications: DefaultApplications;

  default_applications = DefaultApplications { browser: "".to_string(), http: "".to_string(), https: "".to_string() };

  result = Command::new(config::OS_CONFIG_TOOL).args(["get", "default-web-browser"]).output();
  match result {
    Ok(output) => { default_applications.browser = String::from_utf8(output.stdout).unwrap().trim_end_matches("\n").to_string(); },
    Err(..) => { }
  }

  result = Command::new(config::OS_CONFIG_TOOL).args(["get", "default-url-scheme-handler", "http"]).output();
  match result {
    Ok(output) => { default_applications.http = String::from_utf8(output.stdout).unwrap().trim_end_matches("\n").to_string(); },
    Err(..) => { }
  }

  result = Command::new(config::OS_CONFIG_TOOL).args(["get", "default-url-scheme-handler", "https"]).output();
  match result {
    Ok(output) => { default_applications.https = String::from_utf8(output.stdout).unwrap().trim_end_matches("\n").to_string(); },
    Err(..) => { }
  }

  return default_applications;
}

fn change_default_app(application_type: DefaultApplicationType) {
  let mut cmd: Command;
  let result: Result<Output, Error>;
  let args:Vec<&str>;

  cmd = Command::new(config::OS_CONFIG_TOOL);

  match application_type {
    DefaultApplicationType::Web => {
      args = vec!["set", "default-web-browser", config::BW_DOTDESKTOP];
    },
    DefaultApplicationType::Http => {
      args = vec!["set", "default-url-scheme-handler", "http", config::BW_DOTDESKTOP];
    },
    DefaultApplicationType::Https => {
      args = vec!["set", "default-url-scheme-handler", "https", config::BW_DOTDESKTOP];
    }
  }

  result = cmd.args(args).output();
  match result {
    Ok(_output) => {
      // println!("Ok->change_default_app: {}", String::from_utf8(output.stdout).unwrap());
    },
    Err(..) => {
      // println!("Err->change_default_app: ");
    }
  }
}

fn check_desktop_configuration_tool() -> bool {
  let result: Result<Output, Error>;

  result = Command::new(config::OS_CONFIG_TOOL).output();
  match result {
    Ok(..) => { return true; },
    Err(..) => { return false; }
  }
}

fn check_installation() -> InstalledStatus {
  let mut status:InstalledStatus = InstalledStatus::empty();
  let system_executable:PathBuf;
  let user_executable:PathBuf;
  let system_dotdesktop:PathBuf;
  let user_dotdesktop:PathBuf;
  let system_icon:PathBuf;
  let user_icon:PathBuf;
  let config_file:PathBuf;

  system_executable = config::get_executable_file(true);
  system_dotdesktop = config::get_dotdesktop_file(true);
  system_icon = config::get_icon_file(true);

  user_executable = config::get_executable_file(false);
  user_dotdesktop = config::get_dotdesktop_file(false);
  user_icon = config::get_icon_file(false);

  config_file = config::get_configuration_file();

  // Set the appropriate bitmask.
  if system_executable.is_file() { status = status | InstalledStatus::HAS_SYSTEM_EXECUTABLE; }
  if user_executable.is_file() { status = status | InstalledStatus::HAS_USER_EXECUTABLE; }
  if system_dotdesktop.is_file() { status = status | InstalledStatus::HAS_SYSTEM_DOTDESKTOP; }
  if user_dotdesktop.is_file() { status = status | InstalledStatus::HAS_USER_DOTDESKTOP; }
  if system_icon.is_file() { status = status | InstalledStatus::HAS_SYSTEM_ICON; }
  if user_icon.is_file() { status = status | InstalledStatus::HAS_USER_ICON; }
  if config_file.is_file() { status = status | InstalledStatus::HAS_CONFIG; }

  return status;
}

fn save_browsewith() {
  let mut destination:PathBuf;
  let is_admin:bool;

  is_admin = is_privileged_user();
  destination = config::get_executable_path(is_admin);

  if !destination.is_dir() {
    std::fs::create_dir_all(&destination).unwrap();
  }

  destination = config::get_executable_file(is_admin);
  if !destination.exists() {
    std::fs::copy(std::env::current_exe().unwrap(), destination).unwrap();
  }
}

#[allow(dead_code)]
fn save_icon() {
  let mut icon_file:PathBuf;
  let is_admin:bool;
  let icon_raw:&[u8];
  let icon_bytes:Bytes;

  icon_raw = include_bytes!("../../resources/browsewith.ico");
  icon_bytes = Bytes::from(&icon_raw[..]);

  is_admin = is_privileged_user();
  icon_file = config::get_icon_path(is_admin);

  if !icon_file.is_dir() {
    std::fs::create_dir_all(&icon_file).unwrap();
  }

  icon_file = config::get_icon_file(is_admin);
  if !icon_file.is_file() {
    match write(&icon_file, icon_bytes) {
      Ok(..) => {},
      Err(..) => { println!("Failed to create '{:?}'", icon_file.to_str()); }
    }
  }
}

fn save_icons() {
  let is_admin:bool;
  let icon_raw:&[u8];
  let dlls_file:String;
  let mut lines:std::str::Lines<>;
  let icons_path:PathBuf;
  let mut src_file:PathBuf;
  let mut dst_file:PathBuf;

  icon_raw = include_bytes!("../../resources/icons.txt");
  dlls_file = String::from_utf8_lossy(icon_raw).to_string();
  lines = dlls_file.lines();

  is_admin = is_privileged_user();
  icons_path = config::get_icon_path(is_admin);

  if !icons_path.exists() {
    create_dir_all(icons_path.clone()).unwrap();
  }

  loop {
    match lines.next() {
      Some(line) => {
        src_file = PathBuf::from("../icons");
        src_file.push(line);
        dst_file = icons_path.clone();
        dst_file.push(line);
        // println!("src:{} dst:{}", src_file.display(), dst_file.display());
        copy(src_file.as_path(), dst_file.as_path()).unwrap();
      },
      None => {
        break;
      }
    }
  }
}

fn save_dotdesktop() {
  let dotdesktop_file:PathBuf;
  let mut mimeapps_file:PathBuf;
  let dotdesktop_raw:&[u8];
  let mut dotdesktop_data:String;
  let system_path:String;
  let icon_file_path:String;

  dotdesktop_raw = include_bytes!("../../resources/browsewith.desktop");

  let is_admin:bool;
  is_admin = is_privileged_user();
  dotdesktop_file = config::get_dotdesktop_file(is_admin);
  // The mimeapps.list is user ~.config/ not in the BrowseWith configuration directory
  mimeapps_file = config::get_home_dir();
  mimeapps_file.push(".config");
  mimeapps_file.push("mimeapps.list");
  system_path = config::get_executable_path(is_admin).to_str().unwrap().to_string();
  icon_file_path = config::get_icon_file(is_admin).to_str().unwrap().to_string();

  if !dotdesktop_file.is_file() {
    dotdesktop_data = String::from_utf8_lossy(dotdesktop_raw).to_string();
    dotdesktop_data = dotdesktop_data.replace("_SYSTEM_PATH_", &system_path);
    dotdesktop_data = dotdesktop_data.replace("_ICON_FILE_", &icon_file_path);

    match write(&dotdesktop_file, dotdesktop_data) {
      Ok(..) => {
        if is_privileged_user() {
          Command::new("update-desktop-database").output().expect("Failed to execute process");
        } else {
          modify_default_list(&mimeapps_file, true);
        }
      },
      Err(..) => { println!("Failed to create '{:?}'", dotdesktop_file.to_str()); }
    }
  }

}

pub fn modify_default_list(file_path:&Path, install:bool) {
  let mut ini:Ini;
  let mut iter:Iter<&str>;
  let mut new_value:String;
  let section:&mut Properties;

  let mime_keys:Vec<&str> = [
    "text/html",
    "x-scheme-handler/http",
    "x-scheme-handler/https"
  ].to_vec();
  let mut ini_changed:bool = false;

  ini = Ini::load_from_file(file_path).unwrap();
  section = ini.section_mut(Some("Added Associations")).unwrap();

  iter = mime_keys.iter();
  loop {
    match iter.next() {
      Some(key) => {
        if section.contains_key(key) {
          match section.get(key) {
            Some(val) => {
              if !val.contains(config::BW_DOTDESKTOP) && install {
                new_value = String::from(val);
                section.insert(key.to_string(), format!("{}{};", new_value, config::BW_DOTDESKTOP) );
                ini_changed = true;
              } else if val.contains(config::BW_DOTDESKTOP) && !install {
                new_value = String::from(val);
                new_value = remove_from_string(new_value, format!("{};", config::BW_DOTDESKTOP));
                section.insert(key.to_string(), new_value);
                ini_changed = true;
              }
            },
            None => { }
          }
        }
      },
      None => {
        break;
      }
    }
  }

  if ini_changed {
    ini.write_to_file(file_path).unwrap();
  }

}

fn remove_browsewith() {
  let executable_file:PathBuf;
  let is_admin:bool;

  is_admin = is_privileged_user();
  executable_file = config::get_executable_file(is_admin);
  if executable_file.is_file() {
    std::fs::remove_file(executable_file).unwrap();
  }
}

fn unregister_browsewith() {
  let config_dir:PathBuf;
  let mut desktop_file:PathBuf;
  let mut mimeapps_file:PathBuf;

  if is_privileged_user() {
    desktop_file = config::get_dotdesktop_file(true);
    if desktop_file.is_file() { std::fs::remove_file(desktop_file).unwrap(); }
      Command::new("update-desktop-database").output().expect("Failed to execute process");
    remove_icon();
  }

  // ~/.loca/share/applications/browsewith.desktop
  desktop_file = config::get_dotdesktop_file(false);
  if desktop_file.is_file() { std::fs::remove_file(desktop_file).unwrap(); }

  // ~/.config/browsewith
  config_dir = config::get_configuration_path();
  if config_dir.is_dir() { std::fs::remove_dir_all(config_dir).unwrap(); }

  // ~/.config/mimeapps.list
  mimeapps_file = config::get_home_dir();
  mimeapps_file.push(".config");
  mimeapps_file.push("mimeapps.list");
  if mimeapps_file.is_file() { modify_default_list(&mimeapps_file, false); }
}

fn remove_icon() {
  let icon_file:PathBuf;
  let is_admin:bool;

  is_admin = is_privileged_user();
  icon_file = config::get_icon_file(is_admin);

  if icon_file.is_file() {
    std::fs::remove_file(icon_file).unwrap();
  }
}

fn remove_from_string(mut string:String, remove:String) -> String {
  let offset:usize;
  let range; //:Range<Idx>;

  offset = string.find(&remove).unwrap_or(string.len());
  range = Range { start: offset, end: offset + remove.len() };

  string.replace_range(range, "");
  return string;
}