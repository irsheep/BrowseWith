use std::fs::{ write, read_to_string };
use std::path::{ Path };
use std::str::{ Lines };
use std::slice::{ Iter };

pub fn modify_default_list(file_path:&Path) {
  let defaults_list_data:String;
  let mut lines:Lines;
  let mut defaults_list_iterator:Iter<&str>;
  
  // Data to be written to 'defaults.file'
  let mut new_data:String;
  let mut new_data_line:String;

  // Name of the '*.desktop' file to be added to the keys in the 'defaults.list' file
  let desktop_filename:&str = "browsewith.desktop";
  // Array of keys that need to be replaced in the 'defaults.list' file
  let defaults_list_keys:Vec<&str> = [
    "text/html=",
    "x-scheme-handler/http=",
    "x-scheme-handler/https="
  ].to_vec();

  defaults_list_data = read_to_string(file_path).expect("Could not read file defaults.list");
  lines = defaults_list_data.lines();
  new_data = String::new();
  loop {
    match lines.next() {
      Some(line) => {
        
        defaults_list_iterator = defaults_list_keys.iter();
        new_data_line = line.clone().to_string();
        loop {
          match defaults_list_iterator.next() {
            Some(key) => {
              if line.contains(key) && !line.contains(desktop_filename) {
                new_data_line = format!("{}{};{}", key, desktop_filename, line.strip_prefix(key).unwrap() ).as_str().to_owned();
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
      None => { break; }
    }
  }

  match write(file_path, new_data.as_bytes()) {
    Ok(..) => {},
    Err(..) => {}
  }
}