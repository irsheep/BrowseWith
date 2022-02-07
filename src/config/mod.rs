use dirs;

use std::path::{ PathBuf };

use std::fs;
use std::fs::{ File };
use std::io::{ BufReader, BufWriter };

use serde::{Deserialize, Serialize};

#[cfg(target_family = "unix")] mod unix;
#[cfg(target_family = "windows")] mod windows;

#[derive(Serialize, Deserialize)]
pub struct Settings {
  pub set_default: i32,
  pub homepage: String,
  pub host_info: bool,
  pub icons_per_row: i32,
  pub icon_spacing: i32
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BrowserSettings {
  pub title: String,
  pub executable: String,
  pub arguments: String,
  pub icon: String
}

#[derive(Serialize, Deserialize)]
pub struct Configuration {
  pub settings: Settings,
  pub browsers_list: Vec<BrowserSettings>
}

pub fn get_configuration() -> Configuration {
  let configuration:Configuration;
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
    println!("Configuration directory not found");
    match fs::create_dir(config_directory_buf.as_path()) {
      e => println!("{:?}", e)
    };
  }
  if !config_file_buf.is_file() {
    println!("Configuration file not found");
    create_configuration_file(&config_file_buf);
  }

  configuration = load_configuration(&config_file_buf);
  return configuration;
}

pub fn get_config_dir() -> PathBuf {
  let mut home_dir:PathBuf;

  home_dir = dirs::home_dir().unwrap();
  home_dir.push(".browsewith");

  return home_dir.to_path_buf();
}
pub fn get_config_file() -> PathBuf {
  let mut config_file:PathBuf;

  config_file = get_config_dir().to_path_buf();
  config_file.push("config.json");

  return config_file.to_path_buf();
}

fn create_configuration_file(config_full_path:&PathBuf) {
  let default_configuration:Configuration;
  default_configuration = get_default_settings();
  save_configuration(config_full_path, &default_configuration);
}

fn get_default_settings() -> Configuration {
  let installed_browsers:Vec<BrowserSettings>;

  #[cfg(target_family = "unix")] {
    installed_browsers = unix::get_browser_list();
  }
  #[cfg(target_family = "windows")] {
    installed_browsers = windows::get_browser_list();
  }

  let default_settings = Configuration {
    settings: Settings {
      set_default: 1,
      homepage: "about:blank".to_string(),
      host_info: true,
      icons_per_row: 3,
      icon_spacing: 5
    },
    browsers_list: installed_browsers
  };
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
    Ok(..) => { println!("Created {}", file_path.to_str().unwrap()); },
    Err(..) => { println!("Failed to create {}", file_path.to_str().unwrap()); }
  };
}
