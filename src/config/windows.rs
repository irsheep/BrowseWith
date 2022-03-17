use std::path::{ PathBuf };

use crate::config::{ BrowserSettings };
use crate::config::{ get_home_dir };
use crate::config::{ BW_EXECUTABLE, BW_CONFIG, BW_ICON_APPLICATION };

pub fn get_browser_list() -> Vec<BrowserSettings> {
  let program_files_list:[String; 2];

  let mut browsers_found:Vec<BrowserSettings> = [].to_vec();
  let mut browser_settings:BrowserSettings;

  let mut path:PathBuf;
  let mut executable:String;
  let mut icon_index:i32;

  let mut browser_list:Vec<BrowserSettings> = [
    BrowserSettings { title: "_Brave".to_string(), executable: "BraveSoftware\\Brave-Browser\\Application\\brave.exe,0".to_string(), arguments: "".to_string(), icon: "".to_string() },
    BrowserSettings { title: "Brave Incog_nito".to_string(), executable: "BraveSoftware\\Brave-Browser\\Application\\brave.exe,0".to_string(), arguments: "--incognito".to_string(), icon: "".to_string() },
    BrowserSettings { title: "Brave _TOR".to_string(), executable: "BraveSoftware\\Brave-Browser\\Application\\brave.exe,0".to_string(), arguments: "--tor".to_string(), icon: "".to_string() },
    BrowserSettings { title: "_Edge".to_string(), executable: "Microsoft\\Edge\\Application\\msedge.exe,0".to_string(), arguments: "".to_string(), icon: "".to_string() },
    BrowserSettings { title: "Edge In_Private".to_string(), executable: "Microsoft\\Edge\\Application\\msedge.exe,0".to_string(), arguments: "-inprivate".to_string(), icon: "".to_string() },
    BrowserSettings { title: "_Firefox".to_string(), executable: "Mozilla Firefox\\firefox.exe,0".to_string(), arguments: "-new-tab".to_string(), icon: "".to_string() },
    BrowserSettings { title: "Firefox Pri_vate".to_string(), executable: "Mozilla Firefox\\firefox.exe,4".to_string(), arguments: "-private-window".to_string(), icon: "".to_string() },
    BrowserSettings { title: "_Google".to_string(), executable: "Google\\Chrome\\Application\\chrome.exe,0".to_string(), arguments: "".to_string(), icon: "".to_string() },
    BrowserSettings { title: "Google _Incognito".to_string(), executable: "Google\\Chrome\\Application\\chrome.exe,7".to_string(), arguments: "--incognito".to_string(), icon: "".to_string() },
    BrowserSettings { title: "Internet E_xplorer".to_string(), executable: "Internet Explorer\\iexplore.exe,0".to_string(), arguments: "".to_string(), icon: "".to_string() },
    BrowserSettings { title: "Internet Explorer InPrivate".to_string(), executable: "Internet Explorer\\iexplore.exe,0".to_string(), arguments: "-private".to_string(), icon: "".to_string() },
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

pub fn get_programfiles_path() -> PathBuf {
  let mut path:String;

  match std::env::var_os("ProgramFiles") {
    Some(var) => {
      path = var.into_string().unwrap();
      path = format!("{}\\BrowseWith", path);
    },
    None => {
      path = String::new();
    }
  };

  return PathBuf::from(path);
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

pub fn get_executable_path(is_admin:bool) -> PathBuf {
  let mut path:PathBuf;
  if is_admin {
    path = get_programfiles_path();
  } else {
    path = get_configuration_path();
    path.push("bin");
  }
  return path;
}
pub fn get_executable_file(is_admin:bool) -> PathBuf {
  let mut path:PathBuf;
  path = get_executable_path(is_admin);
  path.push(BW_EXECUTABLE);
  return path;
}

pub fn get_icon_path(is_admin:bool) -> PathBuf {
  let mut path:PathBuf;
  if is_admin {
    path = get_executable_path(is_admin);
  } else {
    path = get_configuration_path();
    path.push("icons");
  }
  return path;
}
pub fn get_icon_file(is_admin:bool) -> PathBuf {
  let mut path:PathBuf;
  path = get_icon_path(is_admin);
  path.push(BW_ICON_APPLICATION);
  return path;
}

pub fn get_configuration_path() -> PathBuf {
  let mut path:PathBuf;
  path = get_home_dir();
  path.push(".browsewith");
  return path;
}
pub fn get_configuration_file() -> PathBuf {
  let mut path:PathBuf;
  path = get_configuration_path();
  path.push(BW_CONFIG);
  return path;
}
