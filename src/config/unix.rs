use std::path::{ Path, PathBuf };
use std::fs::{ read_to_string };
use std::str::{ Lines };

use crate::config::{ BrowserSettings };

pub fn get_browser_list() -> Vec<BrowserSettings> {
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