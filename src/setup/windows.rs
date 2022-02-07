use std::{ include_bytes, thread, time };
use std::path::{ Path, PathBuf };
use std::fs::{ write };
use std::process::{ exit };
use std::io::{ Error as IoError };
use std::ffi::CString;

use gtk::glib::{ Bytes };
use gtk::gdk_pixbuf::{ Pixbuf };

use windows::core::{ Error, HSTRING };
use windows::Win32::Foundation::{ PWSTR, PSTR };
use windows::Win32::UI::Shell::{ AT_URLPROTOCOL, AL_MACHINE, AL_EFFECTIVE, SHCNE_ASSOCCHANGED, SHCNF_DWORD, SHCNF_FLUSH };
use windows::Win32::UI::Shell::{ IApplicationAssociationRegistration, ApplicationAssociationRegistration, ASSOCIATIONLEVEL, SHChangeNotify };
use windows::Win32::System::Com::{ CLSCTX_ALL };
use windows::Win32::System::Com::{ CoCreateInstance, CoInitialize };

use windows::System::{ RemoteLauncher, RemoteLaunchUriStatus };
use windows::System::RemoteSystems::{ RemoteSystemConnectionRequest, RemoteSystem };
use windows::Foundation::{ IAsyncOperation, AsyncStatus, Uri };
use windows::Networking::{ HostName };

use winreg::enums::{ HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE };
use winreg::enums::{ RegType, RegDisposition };
use winreg::{ RegKey, RegValue };

/*
  Microsoft documentation
  https://docs.microsoft.com/en-us/windows/win32/shell/default-programs
  https://docs.microsoft.com/en-us/windows/uwp/launch-resume/launch-settings-app

  Rust documentation to invoke Windows API, using 'windows-rs'
  https://microsoft.github.io/windows-docs-rs/doc/windows/System/struct.RemoteLauncher.html
  https://microsoft.github.io/windows-docs-rs/doc/windows/System/struct.RemoteLaunchUriStatus.html
  https://microsoft.github.io/windows-docs-rs/doc/windows/System/RemoteSystems/struct.RemoteSystemConnectionRequest.html
  https://microsoft.github.io/windows-docs-rs/doc/windows/Foundation/struct.IAsyncOperation.html
  https://microsoft.github.io/windows-docs-rs/doc/windows/Foundation/struct.AsyncStatus.html
  https://microsoft.github.io/windows-docs-rs/doc/windows/Networking/struct.HostName.html
*/

// Attempts to configure BrowseWith as the default web browser
pub fn set_default_browser(system_wide:bool) {
  let browsewith_handlers:[String; 2] = [ "BrowseWith.Assoc.1".to_string(), "BrowseWith.Assoc.1".to_string() ];
  let registered_applications:Result<[String; 2], Error>;
  let association_level:ASSOCIATIONLEVEL;

  let default_applications_url:HSTRING = HSTRING::from("ms-settings:defaultapps");
  let uri:Uri = Uri::CreateUri(default_applications_url).unwrap();
  let hostname:HostName;
  let remote_system_connection_request:RemoteSystemConnectionRequest;
  let remote_system:RemoteSystem;
  let async_op:IAsyncOperation<RemoteSystem>;
  let result:Result<IAsyncOperation<RemoteLaunchUriStatus>, Error>;

  if system_wide {
    association_level = AL_MACHINE;
  } else {
    association_level = AL_EFFECTIVE;
  }

  match set_registry_settings(false) {
    Ok(..) => { },
    Err(..) => {
      println!("Unable to make changes to the registry");
      exit(100);
    }
  }
  // exit(666);

  // Check if we are the default browser
  registered_applications = get_registered_application(association_level);
  match registered_applications {
    Ok(..) => { },
    Err(..) => {
      println!("Failed to get the default application for HTTP and HTTPs protocols");
      exit(101);
    }
  }

  // Invoke MS Windows default app
  if registered_applications.unwrap() != browsewith_handlers {

    // Create a 'RemoteSystemConnectionRequest' to use with 'RemoteLauncher'
    hostname = HostName::CreateHostName(HSTRING::from("localhost")).unwrap();
    async_op = RemoteSystem::FindByHostNameAsync(hostname).unwrap();
    while async_op.Status().unwrap() != AsyncStatus::Completed {
      thread::sleep(time::Duration::from_millis(50));
    }
    remote_system = async_op.GetResults().unwrap();
    remote_system_connection_request = RemoteSystemConnectionRequest::Create(remote_system).unwrap();

    // Run Microsoft 'Default apps'
    result = RemoteLauncher::LaunchUriAsync(remote_system_connection_request, uri);
    match result {
      Ok(launch_status) => {
        while launch_status.Status().unwrap() != AsyncStatus::Completed {
          thread::sleep(time::Duration::from_millis(50));
        }
        if launch_status.GetResults().unwrap() == RemoteLaunchUriStatus::Success {
          println!("Use the Microsoft 'Default apps' application to set 'BrowseWith' as your default 'Web Browser'");
          println!("Alternatively if you want 'BrowseWith' to be associated with 'http' and 'https' links,  ");
          println!("click on 'Choose defaults apps by protocol', scroll down until you see 'HTTP' and 'HTTPS' and set 'BrowseWith' as the default application.");
        } else {
          println!("Error running 'Default apps'.");
        }
      },
      Err(..) => {
        println!("Failed to open 'Default apps'.");
      }
    }

  } else {
    println!("BrowseWith is already configured as the default browser.");
  }
}

pub fn is_privileged_user() -> bool {
  return false;
}

pub fn load_icon() {
  let mut home_dir_buf:PathBuf;
  let icon_file_path:&Path;
  let icon_file:Pixbuf;
  let icon_raw:&[u8];
  let icon_bytes:Bytes;

  // Load the icon file as '[u8]' at compile time
  icon_raw = include_bytes!("../../resources/browsewith.ico");
  icon_bytes = Bytes::from(&icon_raw[..]);

  // Create the icon file in the configuration directory, if it doesn't exist
  home_dir_buf = dirs::home_dir().unwrap();
  home_dir_buf.push(".browsewith/browsewith.ico");
  icon_file_path = home_dir_buf.as_path();
  if !icon_file_path.is_file() {
    match write(icon_file_path, icon_bytes) {
      Ok(..) => {},
      Err(..) => println!("Failed to create icon file")
    }
  }

  // Confirm that the icon was successfully created before loading
  if icon_file_path.is_file() {
    // Assign the icon to the main window
    icon_file = Pixbuf::from_file(icon_file_path).unwrap();
    gtk::Window::set_default_icon(&icon_file);
  }
}

fn set_registry_settings(system_wide:bool) -> std::io::Result<()> {
  let hkey_root:RegKey;
  let mut sub_key:RegKey;
  let mut reg_capabilities:RegKey;
  let mut _disposition:RegDisposition;

  // This should be replaced by adding as '%USER_PROFILE%\\.browsewith\\browsewith.ico' to the registry
  // as REG_EXPAND_SZ, but 'RegKey::set_raw_value' is writing mangled characters.
  let mut icon_path:PathBuf;
  let icon_full_path:&str;
  icon_path = dirs::home_dir().unwrap();
  icon_path.push(".browsewith");
  icon_path.push("browsewith.ico");
  icon_full_path = icon_path.to_str().unwrap();

  if system_wide {
    hkey_root = RegKey::predef(HKEY_LOCAL_MACHINE);
  } else {
    hkey_root = RegKey::predef(HKEY_CURRENT_USER);
  }

  // Default program
  (sub_key, _disposition) = hkey_root.create_subkey("Software\\BrowseWith.1")?;

  (reg_capabilities, _disposition) = sub_key.create_subkey("Capabilities")?;
  update_reg(&reg_capabilities, "ApplicationDescription", "Select browser to open URL");

  (reg_capabilities, _disposition) = sub_key.create_subkey("Capabilities\\FileAssociations")?;
  update_reg(&reg_capabilities, ".html", "BrowseWith.Assoc.1");

  (reg_capabilities, _disposition) = sub_key.create_subkey("Capabilities\\MIMEAssociations")?;
  update_reg(&reg_capabilities, "application/http", "BrowseWith.Assoc.1");
  update_reg(&reg_capabilities, "application/https", "BrowseWith.Assoc.1");

  (reg_capabilities, _disposition) = sub_key.create_subkey("Capabilities\\UrlAssociations")?;
  update_reg(&reg_capabilities, "http", "BrowseWith.Assoc.1");
  update_reg(&reg_capabilities, "https", "BrowseWith.Assoc.1");

  // ProgID associations
  (sub_key, _disposition) = hkey_root.create_subkey("SOFTWARE\\Classes\\BrowseWith.Assoc.1")?;
  update_reg(&sub_key, "", "Local web page files");
  update_reg(&sub_key, "AppUserModelId", "BrowseWith");

  (reg_capabilities, _disposition) = sub_key.create_subkey("CLSID")?;
  update_reg(&reg_capabilities, "", "{39DCD515-7CD5-4B79-B076-44996FB9D899}");

  (reg_capabilities, _disposition) = sub_key.create_subkey("Application")?;
  update_reg(&reg_capabilities, "ApplicationCompany", "irsheep");
  update_reg(&reg_capabilities, "ApplicationDescription", "Select browser to open URL");
  update_reg(&reg_capabilities, "ApplicationIcon", icon_full_path );
  update_reg(&reg_capabilities, "ApplicationName", "BrowseWith");
  update_reg(&reg_capabilities, "AppUserModelId", "BrowseWith");

  (reg_capabilities, _disposition) = sub_key.create_subkey("DefaultIcon")?;
  update_reg(&reg_capabilities, "", icon_full_path);

  (reg_capabilities, _disposition) = sub_key.create_subkey("shell\\open\\command")?;
  update_reg(&reg_capabilities, "", "F:\\Temp\\browsewith\\browsewith.exe \"%1\"");

  // Registered applications
  (sub_key, _disposition) = hkey_root.create_subkey("SOFTWARE\\RegisteredApplications")?;
  update_reg(&sub_key, "BrowseWith", "Software\\BrowseWith.1\\Capabilities");

  return Ok(());
}

fn get_registered_application(association_level:ASSOCIATIONLEVEL) -> Result<[String; 2], Error> {
  let mut http:[u16; 5] = [ 0x68, 0x74, 0x74, 0x70, 0x00 ];
  let mut https:[u16; 6] = [ 0x68, 0x74, 0x74, 0x70, 0x73, 0x00 ];
  let http_handler_name:String;
  let https_handler_name:String;

  unsafe {
    let com_instance: Result<IApplicationAssociationRegistration, Error>;
    let application_association:IApplicationAssociationRegistration;
    let mut browser_association:Result<PWSTR, _>;

    CoInitialize(std::ptr::null_mut())?;

    // Create an instance of ApplicationAssociationRegistration COM component
    com_instance = CoCreateInstance(
      &ApplicationAssociationRegistration,
      None,
      CLSCTX_ALL
    );
    match com_instance {
      Ok(..) => {
      },
      Err(ref e) => {
        println!("Error: {}", e);
        exit(101);
      }
    }
    application_association=com_instance.unwrap();

    // Get associated application for HTTP protocol
    browser_association = application_association.QueryCurrentDefault(
      PWSTR(http.as_mut_ptr()),
      AT_URLPROTOCOL,
      association_level,
    );
    match browser_association {
      Ok(assoc) => {
        http_handler_name = read_to_string(assoc);
      },
      Err(..) => {
        http_handler_name = String::from("undefined");
      }
    }

    // Get associated application for HTTPs protocol
    browser_association = application_association.QueryCurrentDefault(
      PWSTR(https.as_mut_ptr()),
      AT_URLPROTOCOL,
      association_level,
    );
    match browser_association {
      Ok(assoc) => {
        https_handler_name = read_to_string(assoc);
      },
      Err(..) => {
        https_handler_name = String::from("undefined");
      }
    }

    SHChangeNotify( SHCNE_ASSOCCHANGED, SHCNF_DWORD | SHCNF_FLUSH, std::ptr::null_mut(), std::ptr::null_mut() );

  }

  Ok([http_handler_name, https_handler_name])
}

unsafe fn read_to_string(ptr: PWSTR) -> String {
  let mut len = 0usize;
  let mut cursor = ptr;
  loop {
      let val = cursor.0.read();
      if val == 0 {
          break;
      }
      len += 1;
      cursor = PWSTR(cursor.0.add(1));
  }

  let slice = std::slice::from_raw_parts(ptr.0, len);
  String::from_utf16(slice).unwrap()
}

pub trait PxStr {
  fn to_pwstr(&self) -> PWSTR;
  fn to_pstr(&self) -> PSTR;
}
impl PxStr for str {
  fn to_pwstr(&self) -> PWSTR {
    let mut vec:Vec<u16>;
    vec = self.encode_utf16().collect();
    // vec.push(0);
    return PWSTR(vec.as_mut_ptr());
  }

  fn to_pstr(&self) -> PSTR {
    let cstr = CString::new(self);
    let parameter = cstr.unwrap().into_bytes().into_boxed_slice().as_mut_ptr() as *mut u8;
    return PSTR(parameter);
  }
}

fn update_reg(path:&RegKey, key:&str, value:&str) {
  let v:Result<String, IoError>;

  v = path.get_value(key);
  match v {
    Ok(r) => {
      if r != value {
        path.set_value(key, &value).ok();
      }
    },
    Err(..) => {
      path.set_value(key, &value).ok();
    }
  }

}

#[allow(unused)]
fn update_reg_raw(path:&RegKey, key:&str, value:&str, reg_type:RegType) {
  let v:Result<String, IoError>;
  let data:RegValue;
  let byte:Vec<u8>;

  byte = Vec::<u8>::from(value);
  println!("update_reg_raw \nvalue: {}\nbytes:{:?}", value, byte);

  v = path.get_value(key);
  match v {
    Ok(r) => {
      // if r != value {
        data = RegValue { vtype: reg_type, bytes: byte };
        path.set_raw_value(key, &data);
      // }
    },
    Err(..) => {
      data = RegValue { vtype: reg_type, bytes: byte };
      path.set_raw_value(key, &data);
    }
  }
}

// ------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------

// use windows::Win32::System::Registry::{ HKEY, RegOpenKeyW, RegOpenKeyExA, RegGetValueW, RegGetValueA, RegCloseKey };
// use windows::Win32::System::Registry::{
//   HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE,
//   KEY_WOW64_32KEY, KEY_READ, KEY_WRITE,
//   RRF_RT_REG_SZ
// };
// use windows::Win32::Foundation::WIN32_ERROR;
// use core::ffi::c_void;
// use std::ffi::OsStr;
// use std::convert::TryInto;
// use std::slice::{ Iter };

// use core::ptr::drop_in_place;
// use std::alloc::Layout;
// use std::alloc::dealloc;


// struct WindowsRegistry {
//   key_handle: HKEY
// }

// impl WindowsRegistry {
//   pub fn open(&mut self, hkey:HKEY, registry_path:&str) {
//     let win_error:WIN32_ERROR;
//     let mut key_handle: HKEY = 0 as isize;
//     unsafe {
//       win_error = RegOpenKeyW(
//         hkey,
//         registry_path.to_pwstr(),
//         // 0,
//         // KEY_READ,
//         &mut key_handle
//       );
//       if win_error != 0 {
//         println!("Unable to open registry (err:{})", win_error);
//         exit(win_error as i32);
//       }
//       self.key_handle = key_handle;
//       // drop_in_place(&mut key_handle);
//       // dealloc(key_handle as *mut u8, Layout::new::<HKEY>());
//     }
//   }

//   pub fn read_zs(&mut self, path_to_key:&str, key_name:&str) -> String {
//     let mut win_error:WIN32_ERROR;
//     let mut data_len:u32 = 0;
//     let registry_path:PWSTR;
//     let key_to_read:PWSTR;

//     unsafe {

//       if self.key_handle == -1 {
//         println!("Key is not open");
//         exit(666);
//       }

//       if path_to_key == "" {
//         registry_path = PWSTR(0 as *mut u16);
//       } else {
//         registry_path = path_to_key.to_pwstr();
//       }
//       key_to_read = key_name.to_pwstr();

//       // Get the size of the data
//       win_error = RegGetValueW(
//         self.key_handle,
//         registry_path,
//         key_to_read,
//         RRF_RT_REG_SZ, // dwFlags
//         std::ptr::null_mut(), // pdwType
//         std::ptr::null_mut() as *mut c_void,
//         &mut data_len
//       );
//       if win_error != 0 {
//         println!("Unable to get data_len (err:{})", win_error);
//         exit(win_error as i32);
//       }

//       let mut my_size = (data_len as usize / std::mem::size_of::<u16>()) as usize;
//       let mut data: Vec<u16> = vec![0; data_len as usize ];

//       // Read the actual value of the key
//       win_error = RegGetValueW(
//         self.key_handle,
//         registry_path,
//         key_to_read,
//         RRF_RT_REG_SZ, // dwFlags
//         std::ptr::null_mut(), // pdwType
//         data.as_mut_ptr() as *mut c_void,
//         &mut data_len
//       );
//       if win_error != 0 {
//         println!("Unable to read data from key (err:{})", win_error);
//         exit(win_error as i32);
//       }

//       println!("Reg data: {:#?} size: {}", String::from_utf16(&data).unwrap().trim_matches(char::from(0)), my_size);
//       let s:String = String::from_utf16(&data).unwrap().trim_matches(char::from(0)).to_string();

//       drop_in_place(data.as_mut_ptr() as *mut c_void);
//       // dealloc(data.as_mut_ptr() as *mut u8, Layout::new::<Vec<u16>>());

//       drop_in_place(&mut data_len);
//       // dealloc(data_len as *mut u8, Layout::new::<u32>());

//       return s;
//     }
//   }

//   pub fn close(&mut self) {
//     unsafe {
//       if self.key_handle != -1 {
//         RegCloseKey(self.key_handle);
//         self.key_handle = -1;
//       }
//     }
//   }
// }

// impl Drop for WindowsRegistry {
//   fn drop(&mut self) {
//     unsafe {
//       if self.key_handle != -1 {
//         RegCloseKey(self.key_handle);
//         self.key_handle = -1;
//         println!("WindowsRegistry destructor");
//       }
//     }
//   }
// }

// ------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------
