use std::path::{ Path, PathBuf };
use std::fs::{ read_to_string };
use std::str::{ Lines };

use crate::config::{ BrowserSettings };
use crate::config::{ get_home_dir };
use crate::config::{
  PATH_EXECUTABLE, PATH_DESKTOP, PATH_ICON,
  BW_EXECUTABLE, BW_CONFIG, BW_ICON_APPLICATION, BW_DOTDESKTOP
};

pub fn get_browser_list() -> Vec<BrowserSettings> {
  let mut file_data:String;
  let mut lines:Lines;
  let mut dot_desktop_config:PathBuf;

  let mut browsers_found:Vec<BrowserSettings> = [].to_vec();
  let mut browsers_settings:BrowserSettings;

  let mut browser_list:Vec<BrowserSettings> = [
    BrowserSettings { title: "_Brave".to_string(), executable: "brave-browser".to_string(), arguments: "".to_string(), icon: "brave-browser.png".to_string(), auto_launch: None },
    BrowserSettings { title: "Brave Incog_nito".to_string(), executable: "brave-browser".to_string(), arguments: "--incognito".to_string(), icon: "brave-browser.png".to_string(), auto_launch: None },
    BrowserSettings { title: "Brave _TOR".to_string(), executable: "brave-browser".to_string(), arguments: "--tor".to_string(), icon: "brave-browser.png".to_string(), auto_launch: None },
    BrowserSettings { title: "_Edge".to_string(), executable: "Microsoft\\Edge\\Application\\msedge.exe".to_string(), arguments: "--tor".to_string(), icon: "".to_string(), auto_launch: None },
    BrowserSettings { title: "Edge In_Private".to_string(), executable: "Microsoft\\Edge\\Application\\msedge.exe".to_string(), arguments: "--tor".to_string(), icon: "".to_string(), auto_launch: None },
    BrowserSettings { title: "_Firefox".to_string(), executable: "firefox".to_string(), arguments: "-new-tab".to_string(), icon: "firefox.png".to_string(), auto_launch: None },
    BrowserSettings { title: "Firefox Pri_vate".to_string(), executable: "firefox".to_string(), arguments: "-private-window".to_string(), icon: "firefox.png".to_string(), auto_launch: None },
    BrowserSettings { title: "_Google".to_string(), executable: "google-chrome".to_string(), arguments: "".to_string(), icon: "google-chrome.png".to_string(), auto_launch: None },
    BrowserSettings { title: "Google _Incognito".to_string(), executable: "google-chrome".to_string(), arguments: "--incognito".to_string(), icon: "google-chrome.png".to_string(), auto_launch: None },
    BrowserSettings { title: "_Chromium".to_string(), executable: "chromium-browser".to_string(), arguments: "".to_string(), icon: "chromium-browser.png".to_string(), auto_launch: None },
    BrowserSettings { title: "Chromium Incognit_o".to_string(), executable: "chromium-browser".to_string(), arguments: "--incognito".to_string(), icon: "chromium-browser.png".to_string(), auto_launch: None },
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

pub fn get_executable_path(is_admin:bool) -> PathBuf {
  let mut path:PathBuf;
  if is_admin {
    path = PathBuf::from(PATH_EXECUTABLE);
  } else {
    path = get_home_dir();
    #[cfg(target_os = "linux")] path.push(".local/bin");
    #[cfg(target_os = "freebsd")] path.push("bin");
  }
  return path;
}
pub fn get_executable_file(is_admin:bool) -> PathBuf {
  let mut file:PathBuf;
  file = get_executable_path(is_admin);
  file.push(BW_EXECUTABLE);
  return file;
}

pub fn get_icon_path(is_admin:bool) -> PathBuf {
  let mut path:PathBuf;
  if is_admin {
    path = PathBuf::from(PATH_ICON);
  } else {
    path = get_configuration_path();
    path.push("icons");
  }
  return path;
}
pub fn get_icon_file(is_admin:bool) -> PathBuf {
  let mut file:PathBuf;
  file = get_icon_path(is_admin);
  file.push(BW_ICON_APPLICATION);
  return file;
}

pub fn get_dotdesktop_path(is_admin:bool) -> PathBuf {
  let mut path:PathBuf;
  if is_admin {
    path = PathBuf::from(PATH_DESKTOP);
  } else {
    path = get_home_dir();
    path.push(".local/share/applications");
  }
  return path;
}
pub fn get_dotdesktop_file(is_admin:bool) -> PathBuf {
  let mut file:PathBuf;
  file = get_dotdesktop_path(is_admin);
  file.push(BW_DOTDESKTOP);
  return file;
}

pub fn get_configuration_path() -> PathBuf {
  let mut path:PathBuf;
  path = get_home_dir();
  #[cfg(not(target_os = "macos"))] {
    path.push(".config");
    path.push("browsewith");
  }
  #[cfg(target_os = "macos")] {
    path.push("Applications");
    path.push("BrowseWith.app");
  }
  return path;
}
pub fn get_configuration_file() -> PathBuf {
  let mut file:PathBuf;
  file = get_configuration_path();
  file.push(BW_CONFIG);
  return file;
}
