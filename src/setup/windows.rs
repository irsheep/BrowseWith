use std::{ include_bytes, thread, time };
use std::path::{ PathBuf };
use std::fs::{ write };
use std::process::{ exit };
use std::io::{ Error as IoError };
use std::ffi::CString;
use std::convert::TryInto;

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
use winreg::enums::{ HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_ALL_ACCESS, REG_EXPAND_SZ };
use winreg::enums::{ RegType, RegDisposition };
use winreg::{ HKEY, RegKey, RegValue };
use winapi::um::winreg as winreg_api;
use winapi::um::winuser::SendMessageTimeoutW;
use winapi::um::winuser::{ HWND_BROADCAST, WM_SETTINGCHANGE, SMTO_ABORTIFHUNG };

use bitflags::bitflags;
use is_elevated::is_elevated;

use crate::config;

bitflags! {
  struct InstalledStatus:u8 {
    const HAS_SYSTEM_EXECUTABLE = 0x01;
    const HAS_USER_EXECUTABLE = 0x02;
    const HAS_SYSTEM_DLLS = 0x04;
    const HAS_USER_DLLS = 0x08;
    const HAS_SYSTEM_ICON = 0x10;
    const HAS_USER_ICON = 0x20;
    const HAS_CONFIG = 0x40;
    const EXECUTABLE_OK = Self::HAS_SYSTEM_EXECUTABLE.bits | Self::HAS_USER_EXECUTABLE.bits;
    const DOTDESKTOP_OK = Self::HAS_SYSTEM_DLLS.bits | Self::HAS_USER_DLLS.bits;
    const ICON_OK = Self::HAS_SYSTEM_ICON.bits | Self::HAS_USER_ICON.bits;
  }
}

pub fn install() {
  let is_admin:bool;

  is_admin = is_privileged_user();
  save_browsewith(is_admin);
  save_icon(is_admin);
  save_close_icon(is_admin);
  set_registry_settings().unwrap();
}

pub fn uninstall() {
  remove_browsewith();
  unregister_browsewith();
}

// Attempts to configure BrowseWith as the default web browser
pub fn set_default_browser() {
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

  if is_privileged_user() {
    association_level = AL_MACHINE;
  } else {
    association_level = AL_EFFECTIVE;
  }

  match set_registry_settings() {
    Ok(..) => { },
    Err(..) => {
      println!("Unable to make changes to the registry");
      exit(100);
    }
  }

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

pub fn list_default_applications() {
  let default_apps_system:Result<[String; 2], Error>;
  let default_apps_user:Result<[String; 2], Error>;
  let apps_system:[String; 2];
  let apps_user:[String; 2];

  default_apps_system = get_registered_application(AL_MACHINE);
  default_apps_user = get_registered_application(AL_EFFECTIVE);

  apps_system = default_apps_system.unwrap();
  apps_user = default_apps_user.unwrap();

  println!("Default browser:\n System:\n   http: {}\n   https: {}\n Effective:\n   http: {}\n   https: {}\n",
    apps_system[0],
    apps_system[1],
    apps_user[0],
    apps_user[1]
  );

  check_application_files();
}

pub fn check_application_files() {
  let install_status:InstalledStatus;
  let install_status_system:&str;
  let install_status_user:&str;

  install_status = check_installation();
  install_status_system = if 
      install_status.intersects(InstalledStatus::HAS_SYSTEM_EXECUTABLE) &&
      install_status.intersects(InstalledStatus::HAS_SYSTEM_DLLS) &&
      install_status.intersects(InstalledStatus::HAS_SYSTEM_ICON)
      { "Ok" }
    else if
      install_status.intersects(InstalledStatus::HAS_SYSTEM_EXECUTABLE) ||
      install_status.intersects(InstalledStatus::HAS_SYSTEM_DLLS) ||
      install_status.intersects(InstalledStatus::HAS_SYSTEM_ICON)
      { "Incomplete" }
    else { "Missing" };
  install_status_user = if 
      install_status.intersects(InstalledStatus::HAS_USER_EXECUTABLE) &&
      install_status.intersects(InstalledStatus::HAS_USER_DLLS) &&
      install_status.intersects(InstalledStatus::HAS_USER_ICON)
      { "Ok" }
    else if
      install_status.intersects(InstalledStatus::HAS_USER_EXECUTABLE) ||
      install_status.intersects(InstalledStatus::HAS_USER_DLLS) ||
      install_status.intersects(InstalledStatus::HAS_USER_ICON)
      { "Incomplete" }
    else { "Missing" };


  //Executable, Icon, .desktop
  println!("System configuration [{}]:", install_status_system);
  print_status(config::get_executable_path(true), config::BW_EXECUTABLE, "Executable\t");
  print_status(config::get_icon_path(true), config::BW_ICON_APPLICATION, "Icon\t\t");
  print_dll_status(config::get_executable_path(true), install_status, InstalledStatus::HAS_SYSTEM_DLLS);

  println!("User configuration [{}]:", install_status_user);
  print_status(config::get_executable_path(false), config::BW_EXECUTABLE, "Executable\t");
  print_status(config::get_icon_path(false), config::BW_ICON_APPLICATION, "Icon\t\t");
  print_dll_status(config::get_executable_path(false), install_status, InstalledStatus::HAS_USER_DLLS);
  
  print_status(config::get_configuration_path(), config::BW_CONFIG, "Configuration\t");
}

fn print_status(path:PathBuf, filename:&str, text:&str) {
  let mut file:PathBuf;
  let mut status:&str;

  file = PathBuf::from(path.to_str().unwrap());
  file.push(filename);
  status = "NOT_FOUND";

  if file.is_file() {
    status = "OK"
  }
  println!("  {} [{}] {}", text, status, file.to_str().unwrap());
}
fn print_dll_status(path:PathBuf, install_status:InstalledStatus, status_check:InstalledStatus) {
  let mut status:&str;

  status = "MISSING_DLLS";
  if install_status.intersects(status_check) {
    status = "OK";
  }
  println!("  {} [{}]\t{}\\bin\\*.dll", "DLLs\t\t", status, path.to_str().unwrap());
}

pub fn is_privileged_user() -> bool {
  return is_elevated();
}

pub fn load_icon() {
  let mut icon_file:PathBuf;
  let icon_pixbuf:Pixbuf;

  // Get the icon file, preferring the icon in 'Program Files'
  icon_file = config::get_icon_file(true);
  if !icon_file.is_file() {
    icon_file = config::get_icon_file(false);
  }

  // Set the application Icon if browsewith.ico is found.
  if icon_file.is_file() {
    // Assign the icon to the main window
    icon_pixbuf = Pixbuf::from_file(icon_file).unwrap();
    gtk::Window::set_default_icon(&icon_pixbuf);
  }
}

fn check_installation() -> InstalledStatus {
  let mut status:InstalledStatus = InstalledStatus::empty();
  let system_executable:PathBuf;
  let user_executable:PathBuf;
  let system_dlls:PathBuf;
  let user_dlls:PathBuf;
  let system_icon:PathBuf;
  let user_icon:PathBuf;
  let config_file:PathBuf;
  let system_dll_status;
  let user_dll_status;

  system_executable = config::get_executable_file(true);
  system_icon = config::get_icon_file(true);

  system_dlls = config::get_executable_path(true);
  user_dlls =  config::get_executable_path(false);
  system_dll_status = check_dlls(system_dlls);
  user_dll_status = check_dlls(user_dlls);

  user_executable = config::get_executable_file(false);
  user_icon = config::get_icon_file(false);

  config_file = config::get_configuration_file();

  // Set the appropriate bitmask.
  if system_executable.is_file() { status = status | InstalledStatus::HAS_SYSTEM_EXECUTABLE; }
  if user_executable.is_file() { status = status | InstalledStatus::HAS_USER_EXECUTABLE; }
  if system_dll_status.installed { status = status | InstalledStatus::HAS_SYSTEM_DLLS; }
  if user_dll_status.installed { status = status | InstalledStatus::HAS_USER_DLLS; }
  if system_icon.is_file() { status = status | InstalledStatus::HAS_SYSTEM_ICON; }
  if user_icon.is_file() { status = status | InstalledStatus::HAS_USER_ICON; }
  if config_file.is_file() { status = status | InstalledStatus::HAS_CONFIG; }

  return status;
}

fn set_registry_settings() -> std::io::Result<()> {
  let hkey_root:RegKey;
  let mut sub_key:RegKey;
  let mut reg_capabilities:RegKey;
  let mut _disposition:RegDisposition;
  let executable_path:PathBuf;
  let icon_path:PathBuf;
  let icon_full_path:&str;
  let is_admin:bool;

  if is_privileged_user() {
    hkey_root = RegKey::predef(HKEY_LOCAL_MACHINE);
    is_admin = true;
  } else {
    hkey_root = RegKey::predef(HKEY_CURRENT_USER);
    is_admin = false;
  }

  executable_path = config::get_executable_file(is_admin);
  icon_path = config::get_icon_file(is_admin);

  icon_full_path = icon_path.to_str().unwrap();

  // Default program
  (sub_key, _disposition) = hkey_root.create_subkey("Software\\BrowseWith.1")?;

  (reg_capabilities, _disposition) = sub_key.create_subkey("Capabilities")?;
  registry_add_value(&reg_capabilities, "ApplicationDescription", "Select browser to open URL");

  (reg_capabilities, _disposition) = sub_key.create_subkey("Capabilities\\FileAssociations")?;
  registry_add_value(&reg_capabilities, ".html", "BrowseWith.Assoc.1");

  (reg_capabilities, _disposition) = sub_key.create_subkey("Capabilities\\MIMEAssociations")?;
  registry_add_value(&reg_capabilities, "application/http", "BrowseWith.Assoc.1");
  registry_add_value(&reg_capabilities, "application/https", "BrowseWith.Assoc.1");

  (reg_capabilities, _disposition) = sub_key.create_subkey("Capabilities\\UrlAssociations")?;
  registry_add_value(&reg_capabilities, "http", "BrowseWith.Assoc.1");
  registry_add_value(&reg_capabilities, "https", "BrowseWith.Assoc.1");

  // ProgID associations
  (sub_key, _disposition) = hkey_root.create_subkey("SOFTWARE\\Classes\\BrowseWith.Assoc.1")?;
  registry_add_value(&sub_key, "", "Local web page files");
  registry_add_value(&sub_key, "AppUserModelId", "BrowseWith");

  (reg_capabilities, _disposition) = sub_key.create_subkey("CLSID")?;
  registry_add_value(&reg_capabilities, "", "{39DCD515-7CD5-4B79-B076-44996FB9D899}");

  (reg_capabilities, _disposition) = sub_key.create_subkey("Application")?;
  registry_add_value(&reg_capabilities, "ApplicationCompany", "irsheep");
  registry_add_value(&reg_capabilities, "ApplicationDescription", "Select browser to open URL");
  registry_add_value(&reg_capabilities, "ApplicationIcon", icon_full_path );
  registry_add_value(&reg_capabilities, "ApplicationName", "BrowseWith");
  registry_add_value(&reg_capabilities, "AppUserModelId", "BrowseWith");

  (reg_capabilities, _disposition) = sub_key.create_subkey("DefaultIcon")?;
  registry_add_value(&reg_capabilities, "", icon_full_path);

  (reg_capabilities, _disposition) = sub_key.create_subkey("shell\\open\\command")?;
  registry_add_value(&reg_capabilities, "", &format!("{} \"%1\"", &executable_path.to_str().unwrap()));

  // Registered applications
  (sub_key, _disposition) = hkey_root.create_subkey("SOFTWARE\\RegisteredApplications")?;
  registry_add_value(&sub_key, "BrowseWith", "Software\\BrowseWith.1\\Capabilities");

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

fn save_browsewith(is_admin:bool) {
  let mut file_location:PathBuf;

  file_location = config::get_executable_path(is_admin);
  if !file_location.is_dir() {
    std::fs::create_dir_all(&file_location).unwrap();
  }

  file_location = config::get_executable_file(is_admin);
  // Always copy in case its an update
  std::fs::copy(std::env::current_exe().unwrap(), &file_location).unwrap();

  if file_location.exists() {
    copy_dlls();
    add_env_path(file_location.to_str().unwrap().to_string());
  }
}

fn save_icon(is_admin:bool) {
  let mut file_location:PathBuf;
  let icon_raw:&[u8];
  let icon_bytes:Bytes;

  icon_raw = include_bytes!("../../resources/browsewith.ico");
  icon_bytes = Bytes::from(&icon_raw[..]);

  file_location = config::get_icon_path(is_admin);
  if !file_location.is_dir() {
    std::fs::create_dir_all(&file_location).unwrap();
  }

  file_location = config::get_icon_file(is_admin);
  if !file_location.is_file() {
    match write(&file_location, icon_bytes) {
      Ok(..) => {},
      Err(..) => { println!("Failed to create '{:?}'", file_location.to_str()); }
    }
  }
}

fn save_close_icon(is_admin:bool) {
  let mut file_location:PathBuf;
  let icon_raw:&[u8];
  let icon_bytes:Bytes;

  icon_raw = include_bytes!("../../resources/window-close-symbolic.png");
  icon_bytes = Bytes::from(&icon_raw[..]);

  file_location = config::get_icon_path(is_admin);

  if !file_location.is_dir() {
    std::fs::create_dir_all(&file_location).unwrap();
  }

  file_location.push(config::BW_ICON_CLOSE);
  if !file_location.is_file() {
    match write(&file_location, icon_bytes) {
      Ok(..) => {},
      Err(..) => { println!("Failed to create '{:?}'", file_location.to_str()); }
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

  env_path = registry_read_string(&sub_key, "Path");

  if !env_path.contains(&browsewith_path) {
    if env_path.ends_with(";") {
      registry_add_value_raw(&sub_key, "Path", (format!("{}{}", env_path, browsewith_path)).as_str(), REG_EXPAND_SZ) ;
    } else {
      registry_add_value_raw(&sub_key, "Path", (format!("{};{}", env_path, browsewith_path)).as_str(), REG_EXPAND_SZ) ;
    }
    notify_env_change();
  }
}

fn remove_env_path() {
  let hkey_root:RegKey;
  let sub_key:RegKey;
  let env_path:String;
  let mut path_list:Vec<&str>;
  let mut i:usize;
  let mut iterator:Iter<&str>;
  let mut update_registry:bool;
  
  if is_privileged_user() {
    hkey_root = RegKey::predef(HKEY_LOCAL_MACHINE);
    sub_key = hkey_root.open_subkey_with_flags("SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment", KEY_ALL_ACCESS).unwrap();
  } else {
    hkey_root = RegKey::predef(HKEY_CURRENT_USER);
    sub_key = hkey_root.open_subkey_with_flags("Environment", KEY_ALL_ACCESS).unwrap();
  }

  env_path = registry_read_string(&sub_key, "Path");
  update_registry = false;

  if env_path.contains(".browsewith") || env_path.contains("BrowseWith") {
    path_list = env_path.split(";").collect();

    i = 0;
    iterator = path_list.iter();
    loop {
      match iterator.next() {
        Some(value) => {
          if value.contains(".browsewith") || value.contains("BrowseWith") {
            update_registry = true;
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
    if update_registry {
      registry_add_value_raw(&sub_key, &"Path", &path_list.join(";"), REG_EXPAND_SZ);
      notify_env_change();
    }
  }
}

fn notify_env_change() {
  unsafe {
    SendMessageTimeoutW(
      HWND_BROADCAST,
      WM_SETTINGCHANGE,
      0,
      ("Environment".as_ptr() as i64).try_into().unwrap(),
      SMTO_ABORTIFHUNG,
      5000,
      std::ptr::null_mut()
    );
  }
}

struct DllInstallStatus<'a> {
  pub installed: bool,
  pub missing_list: Vec<&'a str>
}
impl DllInstallStatus<'_> {
  pub fn new() -> Self {
    return Self {
      installed: false,
      missing_list: Vec::new()
    }
  }
}
pub fn required_dlls() -> Vec<&'static str> {
  return [
    "iconv.dll",
    "libatk-1.0-0.dll",
    "libbz2-1.dll",
    "libcairo-2.dll",
    "libcairo-gobject-2.dll",
    "libepoxy-0.dll",
    "libexpat-1.dll",
    "libffi-6.dll",
    "libfontconfig-1.dll",
    "libfreetype-6.dll",
    "libfribidi-0.dll",
    "libgcc_s_seh-1.dll",
    "libgdk_pixbuf-2.0-0.dll",
    "libgdk-3-0.dll",
    "libgio-2.0-0.dll",
    "libglib-2.0-0.dll",
    "libgmodule-2.0-0.dll",
    "libgobject-2.0-0.dll",
    "libgtk-3-0.dll",
    "libharfbuzz-0.dll",
    "libintl-8.dll",
    "libjpeg-62.dll",
    "libpango-1.0-0.dll",
    "libpangocairo-1.0-0.dll",
    "libpangoft2-1.0-0.dll",
    "libpangowin32-1.0-0.dll",
    "libpcre-1.dll",
    "libpixman-1-0.dll",
    "libpng16-16.dll",
    "libssp-0.dll",
    "libtiff-5.dll",
    "libwinpthread-1.dll",
    "zlib1.dll"
   ].to_vec(); 
}
fn check_dlls(current_path:PathBuf) -> DllInstallStatus<'static> {
  let dll_list:Vec<&str>;
  let mut iter:Iter<&str>;
  let mut check_dll:PathBuf;
  let mut dll_install_status:DllInstallStatus;

  dll_install_status = DllInstallStatus::new();
  dll_install_status.installed = true;

  dll_list = required_dlls();
  iter = dll_list.iter();
  loop {
    match iter.next() {
      Some(dll) => {
        check_dll = (&current_path).to_path_buf();
        check_dll.push(dll);
        if !check_dll.is_file() {
          dll_install_status.installed = false;
          dll_install_status.missing_list.push(dll);
        }
      },
      None => {
        break;
      }
    }
  }

  return dll_install_status;
}
fn copy_dlls() {
  let current_path:PathBuf = PathBuf::from(std::env::current_dir().unwrap());
  let dll_list:Vec<&str>;
  let mut iter:Iter<&str>;
  let mut destination:PathBuf;
  let mut source:PathBuf;

  dll_list = required_dlls();
  iter = dll_list.iter();
  loop {
    match iter.next() {
      Some(dll) => {
        destination = config::get_executable_path(is_privileged_user());
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

fn remove_browsewith() {
  let system_direcotry:PathBuf;
  let user_directory:PathBuf;

  system_direcotry = config::get_executable_path(true);
  user_directory = config::get_configuration_path();

  if is_privileged_user() && system_direcotry.is_dir() {
    println!("Removing directory: {}", system_direcotry.to_str().unwrap());
    std::fs::remove_dir_all(system_direcotry).unwrap();
  }

  if user_directory.is_dir() {
    println!("Removing config dir: {}", user_directory.to_str().unwrap());
    std::fs::remove_dir_all(user_directory).unwrap();
  }

  remove_env_path();
}

fn unregister_browsewith() {
  let hkey_root:HKEY;

  if is_privileged_user() {
    hkey_root = HKEY_LOCAL_MACHINE;
  } else {
    hkey_root = HKEY_CURRENT_USER;
  }

  registry_remove_key(hkey_root, "Software\\BrowseWith.1");
  registry_remove_key(hkey_root, "SOFTWARE\\Classes\\BrowseWith.Assoc.1");

  registry_remove_value(hkey_root, "SOFTWARE\\RegisteredApplications", "browsewith");

}

fn registry_add_value(path:&RegKey, key:&str, value:&str) {
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

// See 'registry_add_value_raw' at the end of this module.
fn _registry_add_value_raw(path:&RegKey, key:&str, value:&str, reg_type:RegType) {
  let v:Result<String, IoError>;
  let data:RegValue;
  let bytes:Vec<u8>;

  bytes = Vec::<u8>::from(format!("{}\0", value));

  v = path.get_value(key);
  match v {
    Ok(_r) => {
      data = RegValue { vtype: reg_type, bytes: bytes };
      match path.set_raw_value(key, &data) {
        Ok(()) => { println!("winreg::set_raw_value successful"); },
        Err(e) => { println!("Error: {:?}", e);}
      }
    },
    Err(..) => {
      println!("Error updating registry");
    }
  }
}

fn registry_read_string(path:&RegKey, key:&str) -> String {
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

fn registry_remove_key(hkey:HKEY, path:&str) {
  let hkey_root:RegKey;

  hkey_root = RegKey::predef(hkey);
  match hkey_root.open_subkey_with_flags(path, KEY_ALL_ACCESS) {
    Ok(_key) => {
      hkey_root.delete_subkey_all(path).unwrap();
    },
    Err(..) => {
    }
  }
}

fn registry_remove_value(hkey:HKEY, path:&str, value:&str) {
  let hkey_root:RegKey;

  hkey_root = RegKey::predef(hkey);
  match hkey_root.open_subkey_with_flags(path, KEY_ALL_ACCESS) {
    Ok(key) => {
      match key.delete_value(value) { Ok(..) => {}, Err(..) => {} }
    },
    Err(..) => {
    }
  }
}

/*
  There seems to be a mismatch with RegValue.bytes, 
  which is a Vec<u8> but in winreg::RegSetValueExW the 
  length of lpData (cbData) is expected to be calculated 
  from a double word Vec<u32>
*/
use winapi::shared::minwindef::{ BYTE, DWORD };
use std::ffi::OsStr;
use std::os::windows::ffi::{ OsStrExt };

fn registry_add_value_raw(hkey:&RegKey, path_and_key:&str, value:&str, reg_type:RegType) {
  let key:Vec<u16>;
  let data:Vec<u16>;

  key = to_utf16(path_and_key);
  data = to_utf16(value);

  unsafe {
    winreg_api::RegSetValueExW(
      hkey.raw_handle(),
      key.as_ptr(),
      0,
      reg_type as DWORD,
      data.as_ptr() as *const BYTE,
      (data.len()*2) as u32 // It does look beautiful ....
    );
  }
}

fn to_utf16<P: AsRef<OsStr>>(s: P) -> Vec<u16> {
  s.as_ref()
    .encode_wide()
    .chain(Some(0).into_iter())
    .collect()
}