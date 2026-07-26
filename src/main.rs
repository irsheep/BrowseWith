#![windows_subsystem = "windows"]

use gtk::{
  prelude::*,
  ButtonsType, MessageType, HeaderBar, Application, ApplicationWindow, Button, Image, Box, Orientation, Align, Label,
  gio::{ ApplicationFlags },
  pango::{ EllipsizeMode }
};

use glib::clone;

use std::process::{ Command, Stdio, exit };
use std::cell::{ RefCell };
use std::path::{ PathBuf, Path };

#[cfg(target_os = "windows")]
use winapi::um::{
  wincon::{ AttachConsole, FreeConsole, ATTACH_PARENT_PROCESS },
  winuser::{ INPUT_u, KEYBDINPUT, VK_RETURN, INPUT_KEYBOARD, SendInput, INPUT }
};

#[cfg(target_family = "windows")]
use base64::{
  Engine as _,
  engine::{ general_purpose }
};

#[cfg(target_family = "windows")]
use std::{
  fs::{ create_dir },
  ffi::CString
};

// Add application modules
mod constants;
mod config;
mod webclient;
mod setup;
mod update;

// Windows specific modules
#[cfg(target_family = "windows")] mod portable_executable;
#[cfg(target_family = "windows")] extern crate base64;

thread_local!(
  static URL:RefCell<String> = RefCell::new(String::new());
  static ICON_SPACING:RefCell<i32> = RefCell::new(0);
  static GIT_RELEASE:RefCell<update::Releases> = RefCell::new(update::Releases::initialize());
);

#[derive(Clone)]
enum UrlAction {
  Allowed,
  Warning,
  Blocked
}
#[derive(Clone)]
struct UrlActionSettings {
  action: UrlAction,
  url: Option<String>
}

#[tokio::main]
async fn main() {
  let help_message:String = String::from_utf8(include_bytes!("../resources/help.txt").to_vec()).unwrap();

  let configuration:config::Configuration;
  let mut valid_url:bool;
  let mut error_code:i32;
  let mut argument_list:Vec<String>;
  let argument_count:usize;
  let argument_appname:String;
  let argument_name:String;
  let mut url_list:String = String::new();

  argument_list = std::env::args().collect();
  argument_count = argument_list.len();
  argument_appname = argument_list[0].clone();
  argument_name = match argument_count {
    0 => argument_list[1].clone(),
    2 => argument_list[1].clone(),
    _ => String::new()
  };
  error_code = -1;
  valid_url = false;

  #[cfg(target_os = "windows")]
  unsafe {
    AttachConsole(ATTACH_PARENT_PROCESS);
  }

  while let Some(u) = argument_list.pop() {
    if webclient::validate_url(&u) || Path::new(&u).is_file() && !u.contains(&argument_appname) {
      url_list = format!("{0},{1}", &u, url_list);
      valid_url = true;
    }
  }

  // println!("{}:{} argument_count: {}, argument_name: {}", file!(), line!(), argument_count, argument_name);
  if argument_count > 1 {

    if argument_count >= 2 {
      if argument_name == "--install" {
        setup::install();
        error_code = 0;
      } else if argument_name == "--uninstall" {
        setup::uninstall();
        error_code = 0;
      } else if argument_name == "--set-as-default-browser" {
        setup::set_default_browser();
        error_code = 0;
      } else if argument_name == "--status" {
        setup::list_default_applications();
        error_code = 0;
      } else if valid_url {
        url_list = url_list.trim_end_matches(",").to_string();
        // println!("{}:{} url_list: {}", file!(), line!(), url_list);
        URL.with( |v| { *v.borrow_mut() = url_list });
        error_code = -1;
      } else {
        println!("ERROR: Invalid URL or argument: '{}'", argument_name.clone());
        error_code = 1;
      }
    } else {
      error_code = 2;
    }
  }

  match error_code {
    -1 => {
      let charset_policy:Option<config::CharsetPolicy>;
      let mut url_list:String = String::new();
      let mut valid_urls:Vec<String> = vec![];
      let mut url_action_settings:UrlActionSettings = UrlActionSettings {
        action: UrlAction::Allowed,
        url: None
      };

      // Read configuration and store settings in 'thread_local'
      configuration = config::get_configuration();
      if !valid_url { URL.with(|v| { *v.borrow_mut() = configuration.settings.homepage.clone(); }); }
      ICON_SPACING.with(|v| { *v.borrow_mut() = configuration.settings.buttons.spacing.clone(); });

      URL.with(|v| {url_list = v.borrow().to_string();});

      charset_policy = configuration.settings.charset_policy;

      url_list.split(",").for_each( |u| {
        // Exit if the URL has 'invalid' characters
        match charset_policy {
          Some(x) => {
            // println!("{}:{} url: {}", file!(), line!(), u);
            if
              check_url(&u, x.utf16, config::CharsetList::Utf16) == config::CharsetPolicyAction::Block ||
              check_url(&u, x.utf32, config::CharsetList::Utf32) == config::CharsetPolicyAction::Block
            {
              valid_urls.push(u.to_string());
              url_action_settings = UrlActionSettings {
                action: UrlAction::Blocked,
                url: None
              };
            } else if
              check_url(&u, x.utf16, config::CharsetList::Utf16) == config::CharsetPolicyAction::Warn ||
              check_url(&u, x.utf32, config::CharsetList::Utf32) == config::CharsetPolicyAction::Warn
            {
              valid_urls.push(u.to_string());
              url_action_settings = UrlActionSettings {
                action: UrlAction::Warning,
                url: Some(u.to_string())
              };
            } else {
              // println!("{}:{} Saving url: {}", file!(), line!(), u);
              valid_urls.push(u.to_string());
            }
          },
          None => { }
        }
      });

      let mut user_launch_urls:Vec<String> = vec![];
      // Open the URL with the pre-defined browser
      valid_urls.iter().for_each( |u| {
        // println!("{}:{} Autolaunch url: {}", file!(), line!(), u);
        match config::auto_launch_browser(u.to_string(), configuration.browsers_list.clone()) {
          Some(browser) => { start_browser(browser, u, None); },
          None => { user_launch_urls.push(u.to_string()); }
        }
      });

      URL.with( |v| { *v.borrow_mut() = user_launch_urls.join(",") });

      // Check for upates
      std::thread::spawn( move || {
        let mut updates_check_file:PathBuf = config::get_config_dir();
        let mut saved_check:update::Releases = update::Releases::initialize();

        updates_check_file.push(constants::UPDATES_CHECK_FILENAME);

        // Prevent from checking for updates if the last check was done recently
        match std::fs::metadata(&updates_check_file) {
          Ok(metadata) => {
            let delta:std::time::Duration = std::time::Duration::from_secs(constants::UPDATES_CHECK_FILE_DELAY);
            if metadata.modified().unwrap().elapsed().unwrap() <= delta {
              return Some(());
            } else {
              saved_check = update::read_check_file(&updates_check_file);
              std::fs::remove_file(&updates_check_file).unwrap();
            }
          },
          _ => { }
        }

        match update::Releases::new() {
          Some(release) => {
            update::write_check_file(&updates_check_file, release);
          },
          None => {
            if saved_check.version == "" {
              update::write_check_file(&updates_check_file, update::Releases::initialize());
            }
          }
        }
        return Some(());
      });

      show_application_window(configuration, url_action_settings);
      exit(0);
    },
    0 => {
      #[cfg(target_family = "windows")] send_return();
      exit(0);
    },
    _ => {
      println!("{}", help_message);
      #[cfg(target_family = "windows")] send_return();
      exit(error_code);
    }
  }
}

#[cfg(target_family = "windows")]
fn send_return() {
  let mut input_u:INPUT_u = unsafe { std::mem::zeroed() };
  unsafe {
    *input_u.ki_mut() = KEYBDINPUT {
      wVk: VK_RETURN as u16,
      wScan: 0,
      dwFlags: 0,
      time: 0,
      dwExtraInfo: 0
    };

    let mut input:INPUT = INPUT {
      type_: INPUT_KEYBOARD,
      u: input_u
    };
    FreeConsole();
    SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
  } ;
}

fn show_application_window(configuration:config::Configuration, url_action_settings:UrlActionSettings) {
  let application:Application = Application::builder()
    .application_id("com.sheep.browsewith")
    .flags(ApplicationFlags::HANDLES_COMMAND_LINE)
    .build();

  if gtk::init().is_err() {
    println!("Failed to initialize GTK.");
    exit(1);
  }

  let url_action_settings_clone:UrlActionSettings = url_action_settings.clone();

  // Application ::command-line signal handler
  /* NOTE:
    This acts as a dummy hanler, all the process of CLI arguments is done in 'fn main'
    but Gtk requires this handler if arguments are being passed to browsewith.
    We can't proccess the arguments here also because of the 'Gtk-WARNING **: cannot open display' error
    when running browsewith with elevated priviledges.
  */
  application.connect_command_line( move |app, _cli_arguments| {
    app.activate();
    return 0.into();
  });

  // Application ::active signal handler
  application.connect_activate(move |app| {
    let header_bar:HeaderBar;
    let hostinfo_box:Box;
    let icons_per_row:i32 = configuration.settings.buttons.per_row;
    let icon_spacing:i32 = configuration.settings.buttons.spacing;
    let button_width:i32 = configuration.settings.buttons.width;
    let button_height:i32 = configuration.settings.buttons.height;

    let spacing:i32 = 5;

    let window:ApplicationWindow = ApplicationWindow::builder()
      .application(app)
      .title("BrowseWith")
      .default_width(button_width + icon_spacing * 2)
      .default_height(button_height)
      .build();

    let grid:gtk::Grid = gtk::Grid::builder()
      .margin_start(spacing)
      .margin_end(spacing)
      .margin_top(spacing)
      .margin_bottom(spacing)
      .halign(gtk::Align::Center)
      .valign(gtk::Align::Center)
      .row_spacing(spacing)
      .column_spacing(spacing)
      .build();

    let mut button:gtk::Button;
    let mut row:i32 = 0;
    let mut col:i32 = 0;
    for browser in configuration.browsers_list.clone() {
      let value:Application = app.clone();

      button = button_with_image(&browser.title, &browser.icon);
      button.connect_clicked(move |_| {button_clicked(&value, &browser.clone())});

      grid.attach(&button, col, row, 1, 1);

      col = col + 1;
      if col % icons_per_row == 0 {
        row = row + 1;
        col = 0;
      }
    }

    // Check if we need to add taget URL host information
    if configuration.settings.host_info {
      hostinfo_box = diplay_host_info(&window, button_width * icons_per_row + icon_spacing * icons_per_row - icon_spacing);
      grid.attach(&hostinfo_box, 0, row+1, icons_per_row, 1);
    }

    window.set_child(Some(&grid));
    header_bar = HeaderBar::builder()
      .decoration_layout("menu:close")
      .build();

    #[cfg(target_family = "windows")] {
      let app_clone:Application;
      let close_box:Box;
      let close_button:Button;
      let mut icon_file:PathBuf;

      icon_file = config::get_icon_path(true);
      if !icon_file.is_dir() {
        icon_file = config::get_icon_path(false);
      }
      icon_file.push(config::BW_ICON_CLOSE);

      app_clone = app.clone();
      close_box = Box::new(Orientation::Horizontal, 1);
      close_button = Button::builder()
        .build();
      close_button.connect_clicked(move |_| {close_app(&app_clone);});
      header_bar.pack_end(&close_box);
    }

    // Traits from GtkWindowExt
    window.set_resizable(false);
    window.set_titlebar(Some(&header_bar));

    setup::load_icon();

    // Display main windows with all the components
    window.show();

    // Center the window on screen and set it as always on top.
    #[cfg(target_family = "windows")] {
      unsafe {
        let mut display_settings:winapi::um::wingdi::DEVMODEA = std::mem::zeroed();
        let mut window_size:winapi::shared::windef::RECT = std::mem::zeroed();
        let x:winapi::ctypes::c_int;
        let y:winapi::ctypes::c_int;

        // https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-enumdisplaysettingsa
        // https://docs.rs/winapi/latest/i686-pc-windows-msvc/winapi/um/winuser/fn.EnumDisplaySettingsA.html
        winapi::um::winuser::EnumDisplaySettingsA(
          std::ptr::null(),
          winapi::um::winuser::ENUM_CURRENT_SETTINGS,
          &mut display_settings as *mut winapi::um::wingdi::DEVMODEA
        );

        // https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-findwindowa
        // https://docs.rs/winapi/latest/i686-pc-windows-msvc/winapi/um/winuser/fn.FindWindowA.html
        let handle:winapi::shared::windef::HWND = winapi::um::winuser::FindWindowA(
          std::ptr::null(),
          CString::new("BrowseWith").unwrap().as_ptr() as *const i8
        );

        // Get the dimensions to the BrowseWith window
        // https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getwindowrect
        // https://docs.rs/winapi/latest/i686-pc-windows-msvc/winapi/um/winuser/fn.GetWindowRect.html
        winapi::um::winuser::GetWindowRect(
          handle,
          &mut window_size as *mut winapi::shared::windef::RECT
        );

        // Center the window on the screen display/2 - window/2
        x = (display_settings.dmPelsWidth as i32 / 2 ) - ( window_size.right - window_size.left ) / 2;
        y = (display_settings.dmPelsHeight as i32 / 2 ) - ( window_size.bottom - window_size.top ) / 2;

        // https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowpos
        // https://docs.rs/winapi/latest/i686-pc-windows-msvc/winapi/um/winuser/fn.SetWindowPos.html
        winapi::um::winuser::SetWindowPos(handle, winapi::um::winuser::HWND_TOPMOST, x, y, 0, 0, winapi::um::winuser::SWP_SHOWWINDOW);
      }
    }

    let app_clone:Application = app.clone();
    match url_action_settings.action {
      UrlAction::Blocked => {
        let dialog:gtk::MessageDialog = gtk::MessageDialog::builder()
          .message_type(MessageType::Error)
          .buttons(ButtonsType::Close)
          .modal(true)
          .transient_for(&window)
          .title("Invalid URL")
          .text("URL is blocked due to invalid characters")
          .build();
        dialog.show();
        dialog.connect_response(move |obj, _| {
          obj.close();
          app_clone.quit();
        });
      },
      UrlAction::Warning => {
        match &url_action_settings_clone.url {
          Some(u) => {
            let dialog:gtk::MessageDialog = gtk::MessageDialog::builder()
              .message_type(MessageType::Warning)
              .buttons(ButtonsType::YesNo)
              .modal(true)
              .transient_for(&window)
              .title("Invalid URL")
              .text(format!("The URL '{}' might contain invalid characters\nAre you sure that you want to proceed?", u).as_str())
              .build();
            dialog.show();
            dialog.connect_response(move |obj, response| {
              match response {
                gtk::ResponseType::No => {
                  obj.close();
                  app_clone.quit();
                },
                _ => { obj.close(); }
              }
            });
          },
          None => {}
        }
      },
      _ => {}
    };
  });

  application.run();
}

fn button_with_image(message:&str, image_file:&str) -> gtk::Button {
  let button:gtk::Button;
  let child:gtk::Box;
  let image:gtk::Image;
  let label:gtk::Label;

  let width:i32 = 180;
  let height:i32 = 70;
  let spacing:i32 = 5;

  // Build the button elements
  button = gtk::Button::builder()
    .width_request(width).height_request(height)
    .build();
  child = gtk::Box::builder()
    .orientation(gtk::Orientation::Horizontal)
    .halign(gtk::Align::Center)
    .spacing(spacing)
    .build();
  image = get_icon_image(&image_file.to_string());
  label = gtk::Label::builder()
    .label(message)
    .use_underline(true)
    .build();

  // Add the image and label to the button,
  // inside a GtkBox
  child.append(&image);
  child.append(&label);

  // button.with_mnemonic(message);
  button.set_child(Some(&child));

  return button;
}

fn button_clicked<'a>(application:&Application, browser_settings:&'a config::BrowserSettings ) {
  let mut url_list:String = String::new();
  URL.with(|v| {url_list = v.borrow().to_string();});
  url_list.split(",").for_each( |u| {
    start_browser(browser_settings.clone(), &u, Some(application));
  });
}

fn close_app<'a>(application:&'a Application) {
  application.quit();
}

fn diplay_host_info(window:&ApplicationWindow, max_width:i32) -> Box {
  let mut icon_spacing:i32 = 0;
  let download_icon_size:i32 = 100;
  let box_object:Box;
  let button:Button;
  let pathbuf:PathBuf = config::get_resource_path("icons", "download.png");
  let image:Image = Image::from_file(pathbuf.clone());
  let label_url:Label;
  let mut url:String = String::new();
  let url_label:String;
  let url_tooltip:String;
  let url_list:Vec<&str>;

  // Get variables stored in 'thread_local'
  ICON_SPACING.with(|v| {icon_spacing = *v.borrow();});
  URL.with(|v| {url = v.borrow().to_string();});

  url_list = url.split(",").collect();
  match url_list.len() {
    1 => {
      url_label = format!("Url: {}", url);
      url_tooltip = format!("Url: {}", url);
    },
    _ => {
      url_label = format!("Open {} urls with", url_list.len());
      url_tooltip = url_list.join("\n");
    }
  }

  // Create the Label objects
  label_url = Label::builder()
    .name("lblurl")
    .halign(Align::Start)
    .hexpand(false)
    .width_request(max_width - icon_spacing - download_icon_size)
    .margin_start(download_icon_size / 2)
    .max_width_chars(30)
    .label(&url_label)
    .tooltip_text(&url_tooltip)
    .ellipsize(EllipsizeMode::End)
    .build();

  button = Button::builder()
    .halign(Align::End)
    .hexpand(false)
    .margin_start(icon_spacing + 13)
    .can_focus(false)
    .sensitive(false)
    .tooltip_text("Checking for updates")
    // .icon_name("epiphany-download")
    .build();

  if pathbuf.exists() {
    button.set_child(Some(&image));
  } else {
    button.set_label("\u{2193}");
    button.set_width_request(24);
    button.set_margin_start(icon_spacing);
  }

  // Create a Box and add all the labels inside
  box_object = Box::builder()
    .orientation(Orientation::Horizontal)
    .spacing(0)
    .margin_top(icon_spacing)
    .margin_start(icon_spacing)
    .margin_bottom(icon_spacing)
    .halign(Align::Start)
    .build();

  box_object.append(&label_url);
  box_object.append(&button);

  let window_clone:ApplicationWindow = window.clone();

  button.connect_clicked(move |_| {

    let mut git_release:update::Releases = update::Releases::initialize();
    GIT_RELEASE.with(|v| { git_release = v.clone().into_inner() });

    let release_dialog = gtk::MessageDialog::builder()
      .message_type(MessageType::Info)
      .buttons(ButtonsType::YesNo)
      .modal(true)
      .transient_for(&window_clone)
      .text(format!(
        "A new release of BrowseWith is available:\nCurrent: {}\nNew: {}\nDo you want to set the URL to the new release page?",
        env!("CARGO_PKG_VERSION"),
        git_release.version
      ).as_str())
      .build();

    let label_url_clone = label_url.clone();
    release_dialog.show();

    release_dialog.connect_response(move |obj, response| {
      match response {
        gtk::ResponseType::Yes => {
          let html_url_clone:String = git_release.html_url.clone();
          label_url_clone.set_label(format!("Url: {}", git_release.html_url).as_str());
          URL.with(|v| {*v.borrow_mut() = html_url_clone});
        },
        _ => {}
      }
      obj.close();
    });

  });

  // Start a thread to check for updates
  glib::source::timeout_add_local(std::time::Duration::new(1, 0), move || {
    let mut updates_check_file:PathBuf = config::get_config_dir();
    updates_check_file.push(constants::UPDATES_CHECK_FILENAME);
    match std::fs::metadata(&updates_check_file) {
      Ok(_) => { },
      _ => { return glib::ControlFlow::Continue; }
    }

    // Compare versions
    let git_release:update::Releases = update::read_check_file(&updates_check_file);
    let mut update_message:String = String::new();
    let mut glib_continue:glib::ControlFlow = glib::ControlFlow::Continue;
    if git_release.version == "" {
      update_message = "Unable to check for updates".to_string();
    } else if !git_release.is_newer {
      update_message = "No update is available".to_string();
    } else if git_release.is_newer {
      update_message = format!("New version available\n{}", git_release.version);
      button.set_sensitive(true);
      GIT_RELEASE.with(|v| { *v.borrow_mut() = git_release});
    }

    // println!("{}:{} update_message: '{}'", file!(), line!(), update_message);
    if update_message != String::new() {
      button.set_tooltip_text(Some(format!("Version: v{}\n{}", env!("CARGO_PKG_VERSION"), update_message.as_str()).as_str()));
      glib_continue = glib::ControlFlow::Break;
    }

    return glib_continue;
  });

  return box_object;
}

fn get_icon_image(file_path:&String) -> Image {
  let image:Image;
  let width_height:i32 = 24;

  #[cfg(target_family = "windows")] let mut icon_file:PathBuf;
  #[cfg(target_family = "unix")] let icon_file:PathBuf;

  #[cfg(target_family = "windows")] {
    let config_dir:String = config::get_config_dir().to_str().unwrap().to_string();
    let parts:Vec<&str>;
    let source:String;
    let index:usize;

    let mut b64_file_path:String = String::new();
    general_purpose::STANDARD.encode_string(file_path, &mut b64_file_path);

    if file_path.contains(".exe") {
      icon_file = PathBuf::from(&config_dir);
      icon_file.push("cache");
      icon_file.push(b64_file_path);
      icon_file.set_extension("ico");

      // Get icon index from an .exe file. Default to 0 if not specified
      parts = file_path.split(",").collect();
      source = parts[0].to_string();
      if file_path.contains(",") {
        index = parts[1].parse().unwrap();
      } else {
        index = 0;
      }

      if !icon_file.parent().unwrap().exists() {
        create_dir(icon_file.parent().unwrap()).unwrap();
      }
      if !icon_file.exists() {
        portable_executable::save_icon(source.as_str(), index, icon_file.to_str().unwrap(), Some(width_height));
      }

    } else {
      icon_file = PathBuf::from(&file_path);
    }
  }
  #[cfg(target_family = "unix")] {
    icon_file = PathBuf::from(&file_path);
  }

  if icon_file.is_file() {
    image = Image::builder()
      .file(icon_file.to_str().unwrap())
      .width_request(width_height)
      .height_request(width_height)
      .build();
  } else {
    image = Image::builder()
      .icon_name(file_path)
      .width_request(width_height)
      .height_request(width_height)
      .build();
  }

  return image;
}

fn start_browser(browser_settings:config::BrowserSettings, url:&str, application:Option<&Application>) {
  let mut args:Vec<&str> = Vec::new();

  if browser_settings.arguments != "" {
    args.push(&browser_settings.arguments);
  }
  url.split(",").for_each(|u| {
    args.push(u);
  });

  Command::new(&browser_settings.executable)
    .args(args.iter())
    .stderr(Stdio::null())
    .stdout(Stdio::null())
    .spawn()
    .expect("failed to execute process");

  match application {
    Some(app) => {
      close_app(&app);
    },
    None => { }
  }
}

fn check_url(url:&str, action:config::CharsetPolicyAction, charset:config::CharsetList) -> config::CharsetPolicyAction {
  let mut test_url:String = url.to_string();
  let mut detected:config::CharsetList = config::CharsetList::Unknown;

  match test_url.pop() {
    Some(c) => {
      let len:usize = c.len_utf8();
      if len == 1 { }
      else if len == 2  && detected == config::CharsetList::Unknown {
        detected = config::CharsetList::Utf16;
      } else if len > 2 && detected == config::CharsetList::Unknown {
        detected = config::CharsetList::Utf32;
      }
    },
    None => { }
  }

  if detected == charset {
    return action;
  }

  return config::CharsetPolicyAction::Allow;
}
