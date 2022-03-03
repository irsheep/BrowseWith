use std::{ include_bytes, thread, time };
use std::path::{ PathBuf };
use std::fs::{ write };
use std::process::{ exit };
use std::io::{ Error as IoError };
use std::ffi::CString;

use core::slice::Iter;

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

use winreg::enums::{ HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_ALL_ACCESS };
use winreg::enums::{ RegType, RegDisposition };
use winreg::{ RegKey, RegValue };

use is_elevated::is_elevated;

use crate::config;
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

pub fn install() {
  save_browsewith();
  save_icon();
}

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
  return is_elevated();
}

pub fn load_icon() {
  let mut icon_file:PathBuf;
  let icon_pixbuf:Pixbuf;

  // Get the icon file, preferring the icon in 'Program Files'
  icon_file = PathBuf::from(config::get_program_dir());
  icon_file.push(config::ICON_FILE);
  if !icon_file.is_file() {
    icon_file = config::get_config_dir();
    icon_file.push(config::ICON_FILE);
  }

  // Set the application Icon if browsewith.ico is found.
  if icon_file.is_file() {
    // Assign the icon to the main window
    icon_pixbuf = Pixbuf::from_file(icon_file).unwrap();
    gtk::Window::set_default_icon(&icon_pixbuf);
  }
}

fn set_registry_settings(system_wide:bool) -> std::io::Result<()> {
  let hkey_root:RegKey;
  let mut sub_key:RegKey;
  let mut reg_capabilities:RegKey;
  let mut _disposition:RegDisposition;
  let mut destination_path:PathBuf;
  // This should be replaced by adding as '%USER_PROFILE%\\.browsewith\\browsewith.ico' to the registry
  // as REG_EXPAND_SZ, but 'RegKey::set_raw_value' is writing mangled characters.
  let icon_path:PathBuf;
  let icon_full_path:&str;

  if is_privileged_user() {
    hkey_root = RegKey::predef(HKEY_LOCAL_MACHINE);
    destination_path = config::get_program_dir();
    icon_path = config::get_program_dir();
  } else {
    hkey_root = RegKey::predef(HKEY_CURRENT_USER);
    destination_path = config::get_config_dir();
    destination_path.push("bin");
    icon_path = config::get_config_dir();
  }

  destination_path.push("browsewith.exe");
  icon_full_path = icon_path.to_str().unwrap();

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
  update_reg(&reg_capabilities, "", &format!("{} \"%1\"", &destination_path.to_str().unwrap()));

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

fn read_reg_string(path:&RegKey, key:&str) -> String {
  let value:Result<String, IoError>;

  value = path.get_value(key);
  match value {
    Ok(key_value) => {
      return key_value;
    },
    Err(..) => {
      return String::new();
    }
  }
}

fn update_reg(path:&RegKey, key:&str, value:&str) {
  let v:Result<String, IoError>;

  v = path.get_value(key);
  match v {
    Ok(r) => {
      if r != value {
        match path.set_value(key, &value) {
          Ok(..) => {
          },
          Err(error) => {
            println!("Failed update registry\n{:?}", error);
          }
        }
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

fn save_browsewith() {
  let mut destination_path:PathBuf;
  let mut destination_file:PathBuf;

  if is_privileged_user() {
    destination_path = config::get_program_dir();
  } else {
    destination_path = config::get_config_dir();
    destination_path.push("bin");
  }

  if !destination_path.is_dir() {
    std::fs::create_dir_all(&destination_path).unwrap();
  }

  destination_file = PathBuf::from(destination_path.to_str().unwrap());
  destination_file.push("browsewith.exe");
  // Always copy in case its an update
  std::fs::copy(std::env::current_exe().unwrap(), &destination_file).unwrap();

  if destination_file.exists() {
    copy_dlls();
    add_env_path(destination_path.to_str().unwrap().to_string());
  }
}

fn save_icon() {
  let mut icon_file:PathBuf;
  let icon_raw:&[u8];
  let icon_bytes:Bytes;

  icon_raw = include_bytes!("../../resources/browsewith.ico");
  icon_bytes = Bytes::from(&icon_raw[..]);

  if is_privileged_user() {
    icon_file = config::get_program_dir();
  } else {
    icon_file = config::get_config_dir();
  }

  if !icon_file.is_dir() {
    std::fs::create_dir_all(&icon_file).unwrap();
  }

  icon_file.push(config::ICON_FILE);
  if !icon_file.is_file() {
    match write(&icon_file, icon_bytes) {
      Ok(..) => {},
      Err(..) => { println!("Failed to create '{:?}'", icon_file.to_str()); }
    }
  }
}

fn add_env_path(browsewith_path:String) {
  let hkey_root:RegKey;
  let sub_key:RegKey;
  let env_path:String;

  if is_privileged_user() {
    hkey_root = RegKey::predef(HKEY_LOCAL_MACHINE);
    sub_key = hkey_root.open_subkey_with_flags("SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment", KEY_ALL_ACCESS).unwrap();
  } else {
    hkey_root = RegKey::predef(HKEY_CURRENT_USER);
    sub_key = hkey_root.open_subkey_with_flags("Environment", KEY_ALL_ACCESS).unwrap();
  }

  env_path = read_reg_string(&sub_key, "Path");

  if !env_path.contains(&browsewith_path) {
    if env_path.ends_with(";") {
      update_reg(&sub_key, "Path", (format!("{}{}", env_path, browsewith_path)).as_str()) ;
    } else {
      update_reg(&sub_key, "Path", (format!("{};{}", env_path, browsewith_path)).as_str()) ;
    }
  }
}

fn _remove_env_path() {
  let hkey_root:RegKey;
  let sub_key:RegKey;
  let env_path:String;
  let mut path_list:Vec<&str>;
  let mut i:usize;
  let mut iterator:Iter<&str>;
  
  if is_privileged_user() {
    hkey_root = RegKey::predef(HKEY_LOCAL_MACHINE);
  } else {
    hkey_root = RegKey::predef(HKEY_CURRENT_USER);
  }

  sub_key = hkey_root.open_subkey("Environment").unwrap();
  env_path = read_reg_string(&sub_key, &"Path");

  if env_path.contains("browsewith.exe") {
    path_list = env_path.split(";").collect();

    i = 0;
    iterator = path_list.iter();
    loop {
      match iterator.next() {
        Some(value) => {
          if value.contains("browsewith.exe") {
            break;
          }
        },
        None => {
          break;
        }
      }
      i = i + 1;
    }
    path_list.remove(i);
    update_reg(&sub_key, &"Path", &path_list.join(";"));
  }

}

fn copy_dlls() {
  let current_path:PathBuf = PathBuf::from(std::env::current_dir().unwrap());
  let dll_list:Vec<&str> = [
    "iconv.dll",
    "icudata67.dll",
    "icui18n67.dll",
    "icuio67.dll",
    "icutest67.dll",
    "icutu67.dll",
    "icuuc67.dll",
    "libasprintf-0.dll",
    "libatk-1.0-0.dll",
    "libatomic-1.dll",
    "libbz2-1.dll",
    "libcairo-2.dll",
    "libcairo-gobject-2.dll",
    "libcairo-script-interpreter-2.dll",
    "libepoxy-0.dll",
    "libexpat-1.dll",
    "libffi-6.dll",
    "libfontconfig-1.dll",
    "libfreetype-6.dll",
    "libfribidi-0.dll",
    "libgailutil-3-0.dll",
    "libgcc_s_seh-1.dll",
    "libgdk-3-0.dll",
    "libgdk_pixbuf-2.0-0.dll",
    "libgettextlib-0-21.dll",
    "libgettextpo-0.dll",
    "libgettextsrc-0-21.dll",
    "libgio-2.0-0.dll",
    "libglib-2.0-0.dll",
    "libgmodule-2.0-0.dll",
    "libgobject-2.0-0.dll",
    "libgthread-2.0-0.dll",
    "libgtk-3-0.dll",
    "libharfbuzz-0.dll",
    "libharfbuzz-icu-0.dll",
    "libharfbuzz-subset-0.dll",
    "libintl-8.dll",
    "libjpeg-62.dll",
    "liblcms2-2.dll",
    "libopenjp2.dll",
    "libpango-1.0-0.dll",
    "libpangocairo-1.0-0.dll",
    "libpangoft2-1.0-0.dll",
    "libpangowin32-1.0-0.dll",
    "libpcre-1.dll",
    "libpcre16-0.dll",
    "libpcre32-0.dll",
    "libpcrecpp-0.dll",
    "libpcreposix-0.dll",
    "libpixman-1-0.dll",
    "libpng16-16.dll",
    "libpoppler-106.dll",
    "libssp-0.dll",
    "libstdc++-6.dll",
    "libtermcap-0.dll",
    "libtextstyle-0.dll",
    "libtiff-5.dll",
    "libtiffxx-5.dll",
    "libturbojpeg.dll",
    "libwinpthread-1.dll",
    "zlib1.dll"
   ].to_vec(); 



  let mut iter:Iter<&str>;
  let mut destination:PathBuf;
  let mut source:PathBuf;

  iter = dll_list.iter();
  loop {
    match iter.next() {
      Some(dll) => {
        if is_privileged_user() {
          destination = PathBuf::from(config::get_program_dir());
        } else {
          destination = config::get_config_dir();
          destination.push("bin");
        }
        destination.push(dll);

        source = (&current_path).to_path_buf();
        source.push(dll);
        if source.is_file() {
          std::fs::copy(source, destination).unwrap();
        }
      },
      None => {
        break;
      }
    }
  }
}