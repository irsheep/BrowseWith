use dirs;
use std::{ include_bytes };

use std::path::{ PathBuf };
use std::fs;
use std::fs::{ File };
use std::io::{ BufReader, BufWriter };

use gtk::glib::{ Bytes };

use serde::{Deserialize, Serialize};

use regex::Regex;

#[cfg(target_family = "unix")] mod unix;
#[cfg(target_family = "windows")] mod windows;

#[cfg(target_family = "unix")] pub static OS_CONFIG_TOOL:&str = "xdg-settings";

#[cfg(target_family = "windows")] pub static BW_EXECUTABLE:&str = "browsewith.exe";
#[cfg(target_family = "unix")] pub static BW_EXECUTABLE:&str = "browsewith";
#[cfg(target_family = "unix")] pub static BW_DOTDESKTOP:&str = "browsewith.desktop";
pub static BW_CONFIG:&str = "config.json";
pub static BW_ICON_APPLICATION:&str = "browsewith.ico";
#[cfg(target_family = "windows")] pub static BW_ICON_CLOSE:&str = "close.png";

#[cfg(target_family = "unix")] pub static PATH_EXECUTABLE:&str = "/usr/local/bin";

#[cfg(target_os = "linux")] pub static PATH_DESKTOP:&str = "/usr/share/applications";
#[cfg(target_os = "linux")] pub static PATH_ICON:&str = "/usr/share/icons/hicolor/scalable/apps";

#[cfg(target_os = "freebsd")] pub static PATH_DESKTOP:&str = "/usr/local/share/applications";
#[cfg(target_os = "freebsd")] pub static PATH_ICON:&str = "/usr/local/share/icons/hicolor/scalable/apps";

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum CharsetPolicyAction {
  Allow,
  Warn,
  Block
}

#[derive(Debug, PartialEq)]
pub enum CharsetList {
  Unknown,
  Utf16,
  Utf32
}

#[derive(Serialize, Deserialize)]
pub struct ButtonProperties {
  pub width: i32,
  pub height: i32,
  pub spacing: i32,
  pub per_row:i32,
  pub show_label: bool,
  pub show_image: bool,
  pub image_position: String
}

#[derive(Serialize, Deserialize)]
pub struct WindowProperties {
  pub always_ontop: bool,
  pub position: String,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct CharsetPolicy {
  pub utf8:CharsetPolicyAction,
  pub utf16:CharsetPolicyAction,
  pub utf32:CharsetPolicyAction
}

#[derive(Serialize, Deserialize)]
pub struct Settings {
  pub homepage: String,
  pub host_info: bool,
  pub buttons: ButtonProperties,
  pub window: WindowProperties,
  pub charset_policy: Option<CharsetPolicy>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BrowserSettings {
  pub title: String,
  pub executable: String,
  pub arguments: String,
  pub icon: String,
  pub auto_launch: Option<Vec<String>>
}

#[derive(Serialize, Deserialize)]
pub struct Configuration {
  pub settings: Settings,
  pub browsers_list: Vec<BrowserSettings>
}

pub fn get_configuration() -> Configuration {
  let mut configuration:Configuration;
  let home_dir_path:Option<PathBuf>;
  let config_directory_buf:PathBuf;
  let config_file_buf:PathBuf;

  // Check user home directory, this should always exist.
  home_dir_path = dirs::home_dir();
  if home_dir_path.is_none() {
    //TODO: Abort with error
  }

  config_directory_buf = get_config_dir();
  config_file_buf = get_config_file();

  // Create configuration directory and file if required
  if !config_directory_buf.is_dir() {
    match fs::create_dir(config_directory_buf.as_path()) {
      Ok(..) => { },
      Err(e) => { println!("{:?}", e); }
    };
  }
  if !config_file_buf.is_file() {
    create_configuration_file(&config_file_buf);
  }

  configuration = load_configuration(&config_file_buf);
  configuration = upgrade_configuration(configuration);

  return configuration;
}

pub fn get_home_dir() -> PathBuf {
  return dirs::home_dir().unwrap();
}

pub fn get_config_dir() -> PathBuf {
  let mut home_dir:PathBuf;

  home_dir = dirs::home_dir().unwrap();
  #[cfg(target_family = "unix")] {
    #[cfg(not(target_os = "macos"))] {
      home_dir.push(".config");
      home_dir.push("browsewith");
    }
    #[cfg(target_os = "macos")] {
      home_dir.push("Applications");
      home_dir.push("BrowseWith.app");
    }
  }
  #[cfg(target_family = "windows")] {
    home_dir.push(".browsewith");
  }

  return home_dir.to_path_buf();
}
pub fn get_config_file() -> PathBuf {
  let mut config_file:PathBuf;

  config_file = get_config_dir().to_path_buf();
  config_file.push("config.json");

  return config_file.to_path_buf();
}

pub fn get_resource_path(dir:&str, file:&str) -> PathBuf {
  let mut path:PathBuf;
  path = get_config_dir().to_path_buf();
  path.push(dir);
  path.push(file);
  return path.to_path_buf();
}

fn create_configuration_file(config_full_path:&PathBuf) {
  let default_configuration:Configuration;
  default_configuration = get_default_settings();
  save_configuration(config_full_path, &default_configuration);
}

fn get_default_settings() -> Configuration {
  let mut default_settings:Configuration;
  let installed_browsers:Vec<BrowserSettings>;
  let config_raw:&[u8];
  let config_bytes:Bytes;

  #[cfg(target_family = "unix")] {
    installed_browsers = unix::get_browser_list();
  }
  #[cfg(target_family = "windows")] {
    installed_browsers = windows::get_browser_list();
  }

  config_raw = include_bytes!("../../resources/config.json");
  config_bytes = Bytes::from(config_raw);
  default_settings = serde_json::from_slice(&config_bytes).unwrap();
  default_settings.browsers_list = installed_browsers;

  return default_settings;
}

fn load_configuration(file_path:&PathBuf) -> Configuration {
  let reader:BufReader<File>;
  let file_handle:File;
  let configuration:Configuration;

  file_handle = File::open(file_path).unwrap();
  reader = BufReader::new(file_handle);
  configuration = serde_json::from_reader(reader).unwrap();
  return configuration;
}

fn save_configuration(file_path:&PathBuf, data:&Configuration) {
  let writer:BufWriter<File>;
  let file_handle:File;

  file_handle = File::create(file_path).unwrap();
  writer = BufWriter::new(file_handle);
  match serde_json::to_writer_pretty(writer, &data) {
    Ok(..) => { println!("Created default configuration: {}", file_path.to_str().unwrap()); },
    Err(..) => { println!("Failed to create {}", file_path.to_str().unwrap()); }
  };
}

pub fn upgrade_configuration(mut data:Configuration) -> Configuration {
  let config_file_buf:PathBuf = get_config_file();
  let default_settings:Configuration = get_default_settings();
  let mut config_upgraded = false;

  match data.settings.charset_policy {
    Some(_) => { },
    None => {
      data.settings.charset_policy = default_settings.settings.charset_policy;
      config_upgraded = true;
    }
  }

  if config_upgraded {
    save_configuration(&config_file_buf, &data);
  }

  return data;
}

#[cfg(target_family = "windows")]
pub fn get_programfiles_path() -> PathBuf {
  return windows::get_programfiles_path();
}

pub fn get_executable_path(is_admin:bool) -> PathBuf {
  #[cfg(target_family = "unix")] return unix::get_executable_path(is_admin);
  #[cfg(target_family = "windows")] return windows::get_executable_path(is_admin);
}
pub fn get_executable_file(is_admin:bool) -> PathBuf {
  #[cfg(target_family = "unix")] return unix::get_executable_file(is_admin);
  #[cfg(target_family = "windows")] return windows::get_executable_file(is_admin);
}

pub fn get_icon_path(is_admin:bool) -> PathBuf {
  #[cfg(target_family = "unix")] return unix::get_icon_path(is_admin);
  #[cfg(target_family = "windows")] return windows::get_icon_path(is_admin);
}
pub fn get_icon_file(is_admin:bool) -> PathBuf {
  #[cfg(target_family = "unix")] return unix::get_icon_file(is_admin);
  #[cfg(target_family = "windows")] return windows::get_icon_file(is_admin);
}

#[cfg(target_family = "unix")]
pub fn get_dotdesktop_path(is_admin:bool) -> PathBuf {
  return unix::get_dotdesktop_path(is_admin);
}
#[cfg(target_family = "unix")]
pub fn get_dotdesktop_file(is_admin:bool) -> PathBuf {
   return unix::get_dotdesktop_file(is_admin);
}

pub fn get_configuration_path() -> PathBuf {
  #[cfg(target_family = "unix")] return unix::get_configuration_path();
  #[cfg(target_family = "windows")] return windows::get_configuration_path();
}
pub fn get_configuration_file() -> PathBuf {
  #[cfg(target_family = "unix")] return unix::get_configuration_file();
  #[cfg(target_family = "windows")] return windows::get_configuration_file();
}

#[cfg(target_family = "windows")]
pub fn get_lib_path(is_admin:bool) -> PathBuf {
  return windows::get_lib_path(is_admin);
}

pub fn auto_launch_browser(url: String, browser_settings: Vec<BrowserSettings>) -> Option<BrowserSettings> {
  let mut re;
  for browser in browser_settings {
    match browser.auto_launch {
      Some(ref auto_launch_url) => {
        for config_url in auto_launch_url {
          re = Regex::new(config_url).unwrap();
          if re.is_match(&url) {
            return Some(browser);
          }
        }
      },
      None => { }
    }
  }
  return std::option::Option::None;
}
