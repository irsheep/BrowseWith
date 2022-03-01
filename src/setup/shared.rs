use std::fs::{ write, read_to_string };
use std::path::{ Path };
use std::str::{ Lines };
use std::slice::{ Iter };
use ini::{ Ini, Properties };

use crate::config;

#[allow(dead_code)]
pub fn modify_default_list(file_path:&Path) {
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
//test
  iter = mime_keys.iter();
  loop {
    match iter.next() {
      Some(key) => {
        if section.contains_key(key) { 
          match section.get(key) {
            Some(val) => {
              if !val.contains(config::DESKTOP_FILE) {
                new_value = String::from(val);
                section.insert(key.to_string(), format!("{}{};", new_value, config::DESKTOP_FILE) );
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

pub fn _dot_desktop(file_path:&Path) {

  let file_data:String;
  let mut lines:Lines;
  let mut keys_iterator:Iter<&str>;

  let mut new_data:String;
  let mut new_data_line:String;

  let dot_desktop_keys:Vec<&str> = [
    "Icon=",
    "Exec="
  ].to_vec();

  file_data = read_to_string(file_path).expect("Could not read file browsewith.desktop");
  lines = file_data.lines();
  new_data = String::new();

  loop {
    match lines.next() {
      Some(line) => {

        keys_iterator = dot_desktop_keys.iter();
        new_data_line = line.clone().to_string();
        loop {
          match keys_iterator.next() {
            Some(key) => {
              if line.contains(key) {
                // Change here
                new_data_line = format!("{}{}",
                  key,
                  "test"
                ).as_str().to_owned();
              }
            },
            None => {
              break;
            }
          }
        }

        new_data.push_str(new_data_line.as_str());
        new_data.push('\n');

      },
      None => {
        break;
      }
    }
  }

  if true {
    match write(file_path, new_data.as_bytes()) {
      Ok(..) => {},
      Err(..) => {}
    }
  }
}
