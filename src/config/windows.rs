
use std::path::{ PathBuf };

use crate::config::{ BrowserSettings };

pub fn get_browser_list() -> Vec<BrowserSettings> {
  let program_files_list:[String; 2];

  let mut browsers_found:Vec<BrowserSettings> = [].to_vec();
  let mut browser_settings:BrowserSettings;

  let mut path:PathBuf;
  let mut executable:String;
  let mut icon_index:i32;

  let mut browser_list:Vec<BrowserSettings> = [
    BrowserSettings { title: "Brave".to_string(), executable: "BraveSoftware\\Brave-Browser\\Application\\brave.exe,0".to_string(), arguments: "".to_string(), icon: "".to_string() },
    BrowserSettings { title: "Brave Incognito".to_string(), executable: "BraveSoftware\\Brave-Browser\\Application\\brave.exe,0".to_string(), arguments: "--incognito".to_string(), icon: "".to_string() },
    BrowserSettings { title: "Brave TOR".to_string(), executable: "BraveSoftware\\Brave-Browser\\Application\\brave.exe,0".to_string(), arguments: "--tor".to_string(), icon: "".to_string() },
    BrowserSettings { title: "Edge".to_string(), executable: "Microsoft\\Edge\\Application\\msedge.exe,0".to_string(), arguments: "--tor".to_string(), icon: "".to_string() },
    BrowserSettings { title: "Edge InPrivate".to_string(), executable: "Microsoft\\Edge\\Application\\msedge.exe,0".to_string(), arguments: "--tor".to_string(), icon: "".to_string() },
    BrowserSettings { title: "Firefox".to_string(), executable: "Mozilla Firefox\\firefox.exe,0".to_string(), arguments: "-new-tab".to_string(), icon: "".to_string() },
    BrowserSettings { title: "Firefox Private".to_string(), executable: "Mozilla Firefox\\firefox.exe,4".to_string(), arguments: "-private-window".to_string(), icon: "".to_string() },
    BrowserSettings { title: "Google".to_string(), executable: "Google\\Chrome\\Application\\chrome.exe,0".to_string(), arguments: "".to_string(), icon: "".to_string() },
    BrowserSettings { title: "Google Incognito".to_string(), executable: "Google\\Chrome\\Application\\chrome.exe,7".to_string(), arguments: "--incognito".to_string(), icon: "".to_string() },
    // { title: "".to_string(), executable: "".to_string(), arguments: "--tor".to_string(), icon: "".to_string() },
  ].to_vec();

  program_files_list = [
    "C:\\Program Files".to_string(),
    "C:\\Program Files (x86)".to_string()
  ];

  for browser in &mut browser_list {
    for program_files in & program_files_list {
      (executable, icon_index) = get_file_and_index(&browser.executable);
      path = PathBuf::from(program_files).join(&executable);

      if path.is_file() {
        browser_settings = browser.clone();
        browser_settings.executable = path.to_str().unwrap().to_string();
        browser_settings.icon = format!("{},{}", path.to_str().unwrap().to_string(), icon_index);
        browsers_found.push(browser_settings.clone());
        break;
      }

    }
  }

  return browsers_found;
}

fn get_file_and_index(file_path:&String) -> (String, i32) {
  let parts:Vec<&str>;
  let source:String;
  let icon_index:i32;

  parts = file_path.split(",").collect();
  source = parts[0].to_string();
  if file_path.contains(",") {
    icon_index = parts[1].parse().unwrap();
  } else {
    icon_index = 0;
  }
  return (source, icon_index);
}