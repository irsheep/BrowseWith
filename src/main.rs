#![windows_subsystem = "windows"]
#![deny(unused_crate_dependencies)]

use gtk::{
  prelude::*,
  ButtonsType, MessageType, HeaderBar, Application, ApplicationWindow, Button, Image, Box, Orientation, Align, PositionType, Label, WindowPosition, MessageDialog,
  gio::{ ApplicationFlags },
  pango::{ EllipsizeMode },
  builders::{ ImageBuilder }
};
use glib::clone;

use std::process::{ Command, Stdio, exit };
use std::cell::{ RefCell };
use std::path::{ PathBuf, Path };

#[cfg(target_os = "windows")]
use winapi::um::{
  wincon::{ FreeConsole, AttachConsole, ATTACH_PARENT_PROCESS },
  winuser::{ SendInput, INPUT, KEYBDINPUT, INPUT_u, INPUT_KEYBOARD, VK_RETURN }
};

#[allow(unused_imports)]
use std::{ file, line };

// Add application modules
mod constants;
mod config;
mod webclient;
mod setup;
mod update;
// Windows specific modules
#[cfg(target_family = "windows")] use std::fs::{ create_dir };
#[cfg(target_family = "windows")] mod portable_executable;
#[cfg(target_family = "windows")] extern crate base64;

#[derive(Clone, Copy)]
struct ButtonMargins {
  left: i32,
  top: i32,
  right: i32,
  bottom: i32
}

thread_local!(
  static URL:RefCell<String> = RefCell::new(String::new());
  static ICON_SPACING:RefCell<i32> = RefCell::new(0);
  static GIT_RELEASE:RefCell<update::Releases> = RefCell::new(update::Releases::initialize());
);

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
              if gtk::init().is_err() {
                println!("Failed to initialize GTK.");
                exit(1);
              };
              let dialog:MessageDialog = MessageDialog::builder()
                .buttons(ButtonsType::Ok)
                .message_type(MessageType::Error)
                .title("Invalid URL")
                .text("URL is blocked due\nto invalid characters")
                .build();
              dialog.run();
              dialog.emit_close();
              gtk::main_iteration();
            } else if
              check_url(&u, x.utf16, config::CharsetList::Utf16) == config::CharsetPolicyAction::Warn ||
              check_url(&u, x.utf32, config::CharsetList::Utf32) == config::CharsetPolicyAction::Warn
            {
              if gtk::init().is_err() {
                println!("Failed to initialize GTK.");
                exit(1);
              }
              if show_dialog(&u) {
                valid_urls.push(u.to_string());
              }
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

      if user_launch_urls.len() == 0 {
        exit(0);
      }

      URL.with( |v| { *v.borrow_mut() = user_launch_urls.join(",") });

      // Check for upates
      std::thread::spawn( move || {
        let mut updates_check_file:PathBuf = config::get_config_dir();
        let mut saved_check:update::Releases = update::Releases::initialize();

        updates_check_file.push(constants::UPDATES_CHECK_FILENAME);

        // Prevent from checking for updates if the last check was done recently
        match std::fs::metadata(&updates_check_file) {
          Ok(metadata) => {
            let delta = std::time::Duration::from_secs(constants::UPDATES_CHECK_FILE_DELAY);
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

      show_application_window(configuration);
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
  let mut input_u: INPUT_u = unsafe { std::mem::zeroed() };
  unsafe {
    *input_u.ki_mut() = KEYBDINPUT {
      wVk: VK_RETURN as u16,
      wScan: 0,
      dwFlags: 0,
      time: 0,
      dwExtraInfo: 0
    };

    let mut input = INPUT {
      type_: INPUT_KEYBOARD,
      u: input_u
    };
    FreeConsole();
    SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
  } ;
}

fn show_application_window(configuration:config::Configuration) {
  let application = Application::builder()
    .application_id("com.sheep.browsewith")
    .flags(ApplicationFlags::HANDLES_COMMAND_LINE)
    .build();

  // Application ::command-line signal handler
  /* NOTE:
    This acts as a dummy hanler, all the process of CLI arguments is done in 'fn main'
    but Gtk requires this handler if arguments are being passed to browsewith.
    We can't proccess the arguments here also because of the 'Gtk-WARNING **: cannot open display' error
    when running browsewith with elevated priviledges.
  */
  application.connect_command_line( move |app, _cli_arguments| {
    app.activate();
    return 0;
  });

  // Application ::active signal handler
  application.connect_activate(move |app| {
    let header_bar:HeaderBar;
    let window_box:Box = Box::new(Orientation::Vertical, 0);
    let icons_box:Box = Box::new(Orientation::Vertical, 0);
    let hostinfo_box:Box;
    let mut icons_row:Box = Box::new(Orientation::Horizontal, 0);
    let mut icon_counter:i32 = 1;
    let icons_per_row:i32 = configuration.settings.buttons.per_row;
    let icon_spacing:i32 = configuration.settings.buttons.spacing;
    let icon_spacing_top:i32 = configuration.settings.buttons.spacing;
    let button_width:i32 = configuration.settings.buttons.width;
    let button_height:i32 = configuration.settings.buttons.height;
    let window_always_ontop:bool = configuration.settings.window.always_ontop;
    let window_position:WindowPosition;
    let button_margin_default:ButtonMargins = ButtonMargins { left: icon_spacing, top: icon_spacing_top, right: 0, bottom: 0 };
    let button_margin_last:ButtonMargins = ButtonMargins { left: icon_spacing, top: icon_spacing, right: icon_spacing, bottom: 0 };
    let header_title:String = String::from("Browsewith"); //format!("Browsewith v{}", env!("CARGO_PKG_VERSION"));

    window_position = match configuration.settings.window.position.as_str() {
      "none" => WindowPosition::None,
      "mouse" => WindowPosition::Mouse,
      _ => WindowPosition::Center
    };

    let window = ApplicationWindow::builder()
      .application(app)
      .title("BrowseWith")
      .default_width(button_width + icon_spacing * 2)
      .default_height(button_height)
      .window_position(window_position)
      .build();

    // Add all browsers as icons to a Box widget, creating a new child Box widget
    // for every 'icons_per_row' browsers
    icons_box.add(&icons_row);
    for browser in configuration.browsers_list.clone() {
      if icon_counter % icons_per_row == 0 {
        button_with_image(&app, &icons_row, &configuration.settings.buttons, &browser,  button_margin_last);
        icons_row = Box::new(Orientation::Horizontal, 0);
        icons_box.add(&icons_row);
      } else {
        button_with_image(&app, &icons_row, &configuration.settings.buttons, &browser, button_margin_default);
      }
      icon_counter = icon_counter + 1;
    }
    window_box.add(&icons_box);

    // Check if we need to add taget URL host information
    if configuration.settings.host_info {
      hostinfo_box = diplay_host_info(button_width * icons_per_row + icon_spacing * icons_per_row - icon_spacing);
      window_box.add(&hostinfo_box);
    } else {
      window_box.add(
        &Box::builder()
          .margin_bottom(configuration.settings.buttons.spacing)
          .build()
      );
    }

    #[cfg(target_family = "unix")] {
      // Build a title bar
      header_bar = HeaderBar::builder()
        .title(header_title.as_str())
        .decoration_layout("menu:close")
        .show_close_button(true)
        .build();
    }
    #[cfg(target_family = "windows")] {
      let app_clone:Application;
      let close_box:Box;
      let close_button:Button;
      let close_image:Image;
      let mut icon_file:PathBuf;

      header_bar = HeaderBar::builder()
        .title(header_title.as_str())
        .build();

      icon_file = config::get_icon_path(true);
      if !icon_file.is_dir() {
        icon_file = config::get_icon_path(false);
      }
      icon_file.push(config::BW_ICON_CLOSE);

      close_image = Image::from_file(icon_file);
      app_clone = app.clone();
      close_box = Box::new(Orientation::Horizontal, 1);
      close_button = Button::builder()
        .image(&close_image)
        .border_width(0).relief(gtk::ReliefStyle::None)
        .build();
      close_button.connect_clicked(move |_| {close_app(&app_clone);});
      close_box.add(&close_button);
      header_bar.pack_end(&close_box);
    }

    // Traits from GtkWindowExt
    window.set_keep_above(window_always_ontop);
    window.set_resizable(false);
    window.set_titlebar(Some(&header_bar));

    setup::load_icon();

    // Display main windows with all the components
    window.add(&window_box);
    window.show_all();

  });

  application.run();
}

fn button_with_image(application:&Application, box_object:&Box, button_properties:&config::ButtonProperties, browser_settings:&config::BrowserSettings, margins:ButtonMargins) {
  let browser_settings_clone:config::BrowserSettings;
  let application_clone:Application;
  let image:Image;
  let image_position:PositionType;
  let button:Button;

  // Clone application and browser_settings so we can pass them to
  // the closure in button connect_clicked
  application_clone = application.clone();
  browser_settings_clone = browser_settings.clone();
  image_position = match button_properties.image_position.as_str() {
    "top" => PositionType::Top,
    "bottom" => PositionType::Bottom,
    "right" => PositionType::Right,
    _ => PositionType::Left
  };

  image = get_icon_image(&browser_settings.icon);

  button = Button::builder()
    .width_request(button_properties.width).height_request(button_properties.height)
    .image(&image).always_show_image(button_properties.show_image).image_position(image_position)
    .margin_start(margins.left)
    .margin_top(margins.top)
    .margin_end(margins.right)
    .margin_bottom(margins.bottom)
    .build();
  if button_properties.show_label || !button_properties.show_image {
    button.set_label(&browser_settings.title);
    button.set_use_underline(true);
  }
  button.connect_clicked(move |_| {button_clicked(&application_clone, &browser_settings_clone)});

  // Add to the main window
  box_object.add(&button);
}

fn button_clicked<'a>(application:&Application, browser_settings:&'a config::BrowserSettings ) {
  let mut url_list:String = String::new();
  URL.with(|v| {url_list = v.borrow().to_string();});
  url_list.split(",").for_each( |u| {
    // println!("{}:{} button_clicked url: {}", file!(), line!(), &u);
    start_browser(browser_settings.clone(), &u, Some(application));
  });
}

fn close_app<'a>(application:&'a Application) {
  application.quit();
}

fn diplay_host_info(max_width:i32) -> Box {
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
    .halign(Align::Start)
    .expand(false)
    .width_request(max_width - icon_spacing - download_icon_size)
    .margin_start(download_icon_size / 2)
    .max_width_chars(30)
    .label(&url_label)
    .tooltip_text(&url_tooltip)
    .ellipsize(EllipsizeMode::End)
    .build();

  button = Button::builder()
    .halign(Align::End)
    .expand(false)
    .margin_start(icon_spacing + 13)
    .can_focus(false)
    .sensitive(false)
    .tooltip_text("Checking for updates")
    .build();

  if pathbuf.exists() {
    button.set_image(Some(&image));
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

  box_object.add(&label_url);
  box_object.add(&button);

  button.connect_clicked(move |_| {

    let mut git_release:update::Releases = update::Releases::initialize();
    GIT_RELEASE.with(|v| { git_release = v.clone().into_inner() });

    let release_dialog = MessageDialog::new(
      None::<&gtk::Window>,
      gtk::DialogFlags::MODAL,
      MessageType::Info,
      ButtonsType::YesNo,
      format!(
        "A new release of BrowseWith is available:\nCurrent: {}\nNew: {}\nDo you want to set the URL to the new release page?",
        env!("CARGO_PKG_VERSION"),
        git_release.version
      ).as_str()
    );

    match release_dialog.run() {
      gtk::ResponseType::Ok => {
        label_url.set_label(format!("Url: {}", git_release.html_url).as_str());
        URL.with(|v| {*v.borrow_mut() = git_release.html_url});
      },
      _ => { }
    };
    release_dialog.close();

  });

  // Start a thread to check for updates
  glib::source::timeout_add_local(std::time::Duration::new(1, 0), clone!(@strong button as btn_widget => move || {
    let mut updates_check_file = config::get_config_dir();
    updates_check_file.push(constants::UPDATES_CHECK_FILENAME);
    match std::fs::metadata(&updates_check_file) {
      Ok(_) => { },
      _ => { return glib::Continue(true); }
    }

    // Compare versions
    let git_release:update::Releases = update::read_check_file(&updates_check_file);
    let mut update_message:String = String::new();
    let mut glib_continue:bool = true;
    if git_release.version == "" {
      update_message = "Unable to check for updates".to_string();
    } else if !git_release.is_newer {
      update_message = "No update is available".to_string();
    } else if git_release.is_newer {
      update_message = format!("New version available\n{}", git_release.version);
      btn_widget.set_sensitive(true);
      GIT_RELEASE.with(|v| { *v.borrow_mut() = git_release});
    }

    // println!("{}:{} update_message: '{}'", file!(), line!(), update_message);
    if update_message != String::new() {
      btn_widget.set_tooltip_text(Some(format!("Version: v{}\n{}", env!("CARGO_PKG_VERSION"), update_message.as_str()).as_str()));
      glib_continue = false;
    }

    return glib::Continue(glib_continue);
  }));

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

    if file_path.contains(".exe") {
      icon_file = PathBuf::from(&config_dir);
      icon_file.push("cache");
      icon_file.push(base64::encode(file_path));
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
    image = ImageBuilder::new()
      .file(icon_file.to_str().unwrap())
      .width_request(width_height)
      .height_request(width_height)
      .build();
  } else {
    image = ImageBuilder::new()
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

fn show_dialog(url:&str) -> bool {
  let message_dialog:MessageDialog = MessageDialog::builder()
    .buttons(ButtonsType::YesNo)
    .message_type(MessageType::Warning)
    .title("Invalid URL")
    .text(format!("The URL '{}' might contain invalid characters\nAre you sure that you want to proceed?", url))
    .build();
    // println!("{}:{} show_dialog: built", file!(), line!());

  match message_dialog.run() {
    gtk::ResponseType::Yes => {
      message_dialog.emit_close();
      gtk::main_iteration();
      return true;
    }
    _ => {
      println!("Aborting due to invalid characters in URL");
      message_dialog.emit_close();
      gtk::main_iteration();
      return false;
    }
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
