use dirs;

use std::path::{ Path, PathBuf };

use std::fs;
use std::fs::{ File, read_to_string };
use std::io::{ BufReader, BufWriter };
use std::str::{ Lines };

use serde::{Deserialize, Serialize};

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
  let installed_browsers:Vec<BrowserSettings> = get_browser_list();

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

fn get_browser_list() -> Vec<BrowserSettings> {
  let mut file_data:String;
  let mut lines:Lines;
  let mut dot_desktop_config:PathBuf;

  let mut browsers_found:Vec<BrowserSettings> = [].to_vec();
  let mut browsers_settings:BrowserSettings;

  let mut browser_list:Vec<BrowserSettings> = [
    BrowserSettings { title: "Brave".to_string(), executable: "brave-browser".to_string(), arguments: "".to_string(), icon: "brave-browser.png".to_string() },
    BrowserSettings { title: "Brave Incognito".to_string(), executable: "brave-browser".to_string(), arguments: "--incognito".to_string(), icon: "brave-browser.png".to_string() },
    BrowserSettings { title: "Brave TOR".to_string(), executable: "brave-browser".to_string(), arguments: "--tor".to_string(), icon: "brave-browser.png".to_string() },
    BrowserSettings { title: "Edge".to_string(), executable: "Microsoft\\Edge\\Application\\msedge.exe".to_string(), arguments: "--tor".to_string(), icon: "".to_string() },
    BrowserSettings { title: "Edge InPrivate".to_string(), executable: "Microsoft\\Edge\\Application\\msedge.exe".to_string(), arguments: "--tor".to_string(), icon: "".to_string() },
    BrowserSettings { title: "Firefox".to_string(), executable: "firefox".to_string(), arguments: "-new-tab".to_string(), icon: "firefox.png".to_string() },
    BrowserSettings { title: "Firefox Private".to_string(), executable: "firefox".to_string(), arguments: "-private-window".to_string(), icon: "firefox.png".to_string() },
    BrowserSettings { title: "Google".to_string(), executable: "chrome".to_string(), arguments: "".to_string(), icon: "google-chrome.png".to_string() },
    BrowserSettings { title: "Google Incognito".to_string(), executable: "chrome".to_string(), arguments: "--incognito".to_string(), icon: "google-chrome.png".to_string() },
    BrowserSettings { title: "Chromium".to_string(), executable: "chromium-browser".to_string(), arguments: "".to_string(), icon: "chromium-browser.png".to_string() },
    BrowserSettings { title: "Chromium Incognito".to_string(), executable: "chromium-browser".to_string(), arguments: "--incognito".to_string(), icon: "chromium-browser.png".to_string() },
    // { title: "".to_string(), executable: "".to_string(), arguments: "--tor".to_string(), icon: "".to_string() },
  ].to_vec();

  let dot_desktop_directories = [
    "/usr/share/applications",
    "/usr/local/share/applications"
  ];

  for browser in &mut browser_list {
    for dot_desktop in dot_desktop_directories {
      dot_desktop_config = Path::new(dot_desktop).join(&browser.executable).with_extension("desktop");

      if dot_desktop_config.is_file() {
        file_data = read_to_string(dot_desktop_config).expect("Could not read file defaults.list");
        lines = file_data.lines();
        browsers_settings = browser.clone();
        loop {
          match lines.next() {
            Some(line) => {
              if line.starts_with("Icon=") {
                browsers_settings.icon = line.trim_start_matches("Icon=").to_string();
              } else if line.starts_with("Exec=") {
                browsers_settings.executable = line.trim_start_matches("Exec=").split_whitespace().next().unwrap().to_string();
              }
            },
            None => {
              break;
            }
          }
        }
        browsers_found.push(browsers_settings.clone());
        break;
      }

    }
  }
  return browsers_found;
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
