use dirs;

use std::path::{ Path, PathBuf };

use std::fs;
use std::fs::{ File };
use std::io::{ BufReader, BufWriter };

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
  let mut config_directory_buf:PathBuf;
  let mut config_file_buf:PathBuf;

  let config_directory_path:&Path;
  let config_file_path:&Path;

  // Check user home directory, this should always exist.
  home_dir_path = dirs::home_dir();
  if home_dir_path.is_none() {
    //TODO: Abort with error
  }

  // Create PathBuf objects for configuration directory and file.
  config_directory_buf = home_dir_path.unwrap();
  config_directory_buf.push(".browsewith/");
  config_file_buf = config_directory_buf.clone();
  config_file_buf.push(".brosewith/"); //BUG: config_directory_buf already has '.browsewith' but its lost when clonned to config_file_buf
  config_file_buf.set_file_name("config.json");

  // Transform the PathBuf objects to Path, so we can do directory/file checks and other operations
  config_directory_path = config_directory_buf.as_path();
  config_file_path = config_file_buf.as_path();

  // Create configuration directory and file if required
  if !config_directory_path.is_dir() {
    println!("Configuration directory not found");
    match fs::create_dir(config_directory_path) {
      e => println!("{:?}", e)
    };
  }
  if !config_file_path.is_file() {
    println!("Configuration file not found");
    create_configuration_file(config_file_path);
  }

  configuration = load_configuration(&config_file_path);
  return configuration;
}

fn create_configuration_file(config_full_path:&Path) {
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
  let mut executable_buf:PathBuf;
  let mut executable_path:&Path;
  let mut icon_buf:PathBuf;
  let mut icon_path:&Path;

  let mut browser_list:Vec<BrowserSettings> = [
    BrowserSettings { title: "Brave".to_string(), executable: "brave-browser".to_string(), arguments: "".to_string(), icon: "brave-browser.png".to_string() },
    BrowserSettings { title: "Brave Incognito".to_string(), executable: "brave-browser".to_string(), arguments: "--incognito".to_string(), icon: "brave-browser.png".to_string() },
    BrowserSettings { title: "Brave TOR".to_string(), executable: "brave-browser".to_string(), arguments: "".to_string(), icon: "brave-browser.png".to_string() },
    BrowserSettings { title: "Edge".to_string(), executable: "Microsoft\\Edge\\Application\\msedge.exe".to_string(), arguments: "--tor".to_string(), icon: "".to_string() },
    BrowserSettings { title: "Edge InPrivate".to_string(), executable: "Microsoft\\Edge\\Application\\msedge.exe".to_string(), arguments: "--tor".to_string(), icon: "".to_string() },
    BrowserSettings { title: "Firefox".to_string(), executable: "firefox".to_string(), arguments: "-new-tab".to_string(), icon: "firefox.png".to_string() },
    BrowserSettings { title: "Firefox Private".to_string(), executable: "firefox".to_string(), arguments: "-private-window".to_string(), icon: "firefox.png".to_string() },
    BrowserSettings { title: "Google".to_string(), executable: "chrome".to_string(), arguments: "--tor".to_string(), icon: "google-chrome.png".to_string() },
    BrowserSettings { title: "Google Incognito".to_string(), executable: "chrome".to_string(), arguments: "--incognito".to_string(), icon: "google-chrome.png".to_string() },
    BrowserSettings { title: "Chromium".to_string(), executable: "chromium-browser".to_string(), arguments: "--tor".to_string(), icon: "chromium-browser.png".to_string() },
    BrowserSettings { title: "Chromium Incognito".to_string(), executable: "chromium-browser".to_string(), arguments: "--incognito".to_string(), icon: "chromium-browser.png".to_string() },
    // { title: "".to_string(), executable: "".to_string(), arguments: "--tor".to_string(), icon: "".to_string() },
  ].to_vec();
  
  let mut browsers_found:Vec<BrowserSettings> = [].to_vec();
  let mut browsers_settings:BrowserSettings;

  let well_known_bin = [
    "/usr/bin",
    "/usr/local/bin",
    "/bin"
  ];
  let well_known_icon = [
    "/usr/share/icons/hicolor/32x32/apps",
    "/usr/share/icons/hicolor/64x64/apps",
    "/usr/share/icons/Papirus/32x32/apps",
    "/usr/share/icons/Papirus/64x64/apps",
  ];

  // let metadata:Result<Metadata>;
  for browser in &mut browser_list {
    for bin in well_known_bin {
      executable_buf = Path::new(bin).join(&browser.executable);
      executable_path = executable_buf.as_path();

      if is_file_executable(&executable_path) {
        browsers_settings = browser.clone();
        
        match executable_path.to_str() {
          None => browsers_settings.executable = "".to_string(),
          Some(f) => {
            browsers_settings.executable = f.to_string();
          }
        }

        for icon in well_known_icon {
          icon_buf = Path::new(icon).join(&browser.icon);
          icon_path = icon_buf.as_path();
          if icon_path.exists() {
            match icon_path.to_str() {
              None => browsers_settings.icon = "".to_string(),
              Some(i) => {
                browsers_settings.icon = i.to_string();
              }
            }
            break;
          }
        }

        browsers_found.push(browsers_settings.clone());
        break;
      }
    }
  }

  return browsers_found;
}

fn load_configuration(file_path:&Path) -> Configuration {
  let reader:BufReader<File>;
  let file_handle:File;
  let configuration:Configuration;

  file_handle = File::open(file_path).unwrap();
  reader = BufReader::new(file_handle);
  configuration = serde_json::from_reader(reader).unwrap();
  return configuration;
}

fn save_configuration(file_path:&Path, data:&Configuration) {
  let writer:BufWriter<File>;
  let file_handle:File;

  file_handle = File::create(file_path).unwrap();
  writer = BufWriter::new(file_handle);
  match serde_json::to_writer_pretty(writer, &data) {
    Ok(..) => { println!("Created {}", file_path.to_str().unwrap()); },
    Err(..) => { println!("Failed to create {}", file_path.to_str().unwrap()); }
  };
}

fn is_file_executable(path:&Path) -> bool {
  if ! path.exists() { return false; }

  if cfg!(unix) {
    // TODO: Check if user has exec permissions on the file,
    // for now just assume that he has
    return true;
  }

  return false;
}

