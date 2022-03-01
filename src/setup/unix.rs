// use dirs;
use std::{ include_bytes };
use std::path::{ Path, PathBuf };
use std::fs::{ write };
use std::process::{ Command, Output };
use std::io::{ Error };

use nix::unistd::{ Uid, getuid };

use gtk::glib::{ Bytes };
use gtk::gdk_pixbuf::{ Pixbuf };
use bitflags::bitflags;

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
    const EXECUTABLE_OK = Self::HAS_SYSTEM_EXECUTABLE.bits | Self::HAS_USER_EXECUTABLE.bits;
    const DOTDESKTOP_OK = Self::HAS_SYSTEM_DOTDESKTOP.bits | Self::HAS_USER_DOTDESKTOP.bits;
    const ICON_OK = Self::HAS_SYSTEM_ICON.bits | Self::HAS_USER_ICON.bits;
  }
}

#[allow(dead_code)]
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
  save_icon();
  save_dotdesktop();
}

#[allow(dead_code)]
pub fn set_default_browser(_system_wide:bool) {
  let apps:DefaultApplications = get_default_applications();
  let install_status:InstalledStatus = check_installation();

  if check_desktop_configuration_tool() &&
     install_status.intersects(InstalledStatus::EXECUTABLE_OK) &&
     install_status.intersects(InstalledStatus::DOTDESKTOP_OK) &&
     install_status.intersects(InstalledStatus::ICON_OK) 
  {
    // Web and HTTP seem to be linked, but we change both just in case
    if apps.browser != config::DESKTOP_FILE { change_default_app(DefaultApplicationType::Web); }
    if apps.http != config::DESKTOP_FILE { change_default_app(DefaultApplicationType::Http); }
    if apps.https != config::DESKTOP_FILE { change_default_app(DefaultApplicationType::Https); }
  } else if !check_desktop_configuration_tool() {
    println!("This application requries '{}' to be installed on this system, in order to make the required changes", config::OS_CONFIG_TOOL);
  } else {
    println!("One or more required files are missing, please run 'browsewith --install' to copy the required files")
  }

}

pub fn list_default_applications() {
  let apps:DefaultApplications = get_default_applications();
  
  println!("Web Browser: {}\nProtocol handlers:\n   http: {}\n   https: {}",
    apps.browser, apps.http, apps.https
  );
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
  home_dir_buf = config::get_config_dir();
  home_dir_buf.push(config::ICON_FILE);
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
      args = vec!["set", "default-web-browser", config::DESKTOP_FILE];
    },
    DefaultApplicationType::Http => {
      args = vec!["set", "default-url-scheme-handler", "http", config::DESKTOP_FILE];
    },
    DefaultApplicationType::Https => {
      args = vec!["set", "default-url-scheme-handler", "https", config::DESKTOP_FILE];
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
  let mut system_executable:PathBuf;
  let mut user_executable:PathBuf;
  let mut system_dotdesktop:PathBuf;
  let mut user_dotdesktop:PathBuf;
  let mut system_icon:PathBuf;
  let mut user_icon:PathBuf;
  let mut config_file:PathBuf;

  system_executable = PathBuf::from(config::PATH_EXECUTABLE);
  system_icon = PathBuf::from(config::PATH_ICON);
  system_dotdesktop = PathBuf::from(config::PATH_DESKTOP);

  user_executable = PathBuf::from(config::get_home_dir());
  user_icon = config::get_config_dir();
  user_dotdesktop = PathBuf::from(config::get_home_dir());

  config_file = config::get_config_dir();

  // If browsewith is in the system or local user path
  system_executable.push("browsewith");
  #[cfg(target_os = "linux")] user_executable.push(".local/bin/browsewith");
  #[cfg(target_os = "freebsd")] user_executable.push("bin/browsewith");

  // Dot desktop file
  system_dotdesktop.push(config::DESKTOP_FILE);
  user_dotdesktop.push(".local/share/applications");
  user_dotdesktop.push(config::DESKTOP_FILE);

  // Icon file
  system_icon.push(config::ICON_FILE);
  user_icon.push(config::ICON_FILE);

  // Configuration file
  config_file.push(config::CONFIG_FILE);

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

  if is_privileged_user() {
    destination = PathBuf::from(config::PATH_EXECUTABLE);
  } else {
    destination = config::get_home_dir();
    #[cfg(target_os = "linux")] destination.push(".local/bin");
    #[cfg(target_os = "freebsd")] destination.push("bin");
  }

  if !destination.is_dir() {
    std::fs::create_dir_all(&destination).unwrap();
  }

  destination.push("browsewith");
  if !destination.exists() {
    std::fs::copy(std::env::current_exe().unwrap(), destination).unwrap();
  }
}

fn save_icon() {
  let mut icon_file:PathBuf;
  let icon_raw:&[u8];
  let icon_bytes:Bytes;

  icon_raw = include_bytes!("../../resources/browsewith.ico");
  icon_bytes = Bytes::from(&icon_raw[..]);

  if is_privileged_user() {
    icon_file = PathBuf::from(config::PATH_ICON);
  } else {
    icon_file = config::get_config_dir();
  }

  if !icon_file.is_dir() {
    std::fs::create_dir_all(&icon_file).unwrap();
  }

  icon_file.push(config::ICON_FILE);
  if !icon_file.is_file() {
    match write(&icon_file, icon_bytes) {
      Ok(..) => {},
      Err(..) => { println!("Failed to create '{:?}'", icon_file.to_str()); }
    }
  }
}

fn save_dotdesktop() {
  let mut dotdesktop_file:PathBuf;
  let mut mimeapps_file:PathBuf;
  let dotdesktop_raw:&[u8];
  let mut dotdesktop_data:String;
  let system_path:String;
  let config_path:String;

  dotdesktop_raw = include_bytes!("../../resources/browsewith.desktop");

  if is_privileged_user() {
    dotdesktop_file = PathBuf::from(config::PATH_DESKTOP);
    mimeapps_file = PathBuf::from(config::PATH_DESKTOP);
    system_path = config::PATH_EXECUTABLE.to_string();
    config_path = config::PATH_ICON.to_string();
  } else {
    dotdesktop_file = config::get_home_dir();
    dotdesktop_file.push(".local/share/applications");
    mimeapps_file = config::get_home_dir();
    mimeapps_file.push(".config");
    system_path = (format!("{}/{}", config::get_home_dir().to_str().unwrap(), ".local/bin")).to_string();
    config_path = config::get_config_dir().to_str().unwrap().to_string();
  }
  dotdesktop_file.push(config::DESKTOP_FILE);
  mimeapps_file.push("mimeapps.list");

  if !dotdesktop_file.is_file() {
    dotdesktop_data = String::from_utf8_lossy(dotdesktop_raw).to_string();
    dotdesktop_data = dotdesktop_data.replace("_SYSTEM_PATH_", &system_path);
    dotdesktop_data = dotdesktop_data.replace("_CONFIG_PATH_", &config_path);

    match write(&dotdesktop_file, dotdesktop_data) {
      Ok(..) => {
        if is_privileged_user() {
          Command::new("update-desktop-database").output().expect("Failed to execute process");
        } else {
          crate::setup::shared::modify_default_list(&mimeapps_file);
        }
      },
      Err(..) => { println!("Failed to create '{:?}'", dotdesktop_file.to_str()); }
    }
  }

}
