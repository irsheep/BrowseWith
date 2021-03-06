// #![windows_subsystem = "windows"]
use gtk::prelude::*;
use gtk::{ HeaderBar, Application, ApplicationWindow, Button, Image, Box, Orientation, Align, PositionType, Label, ImageBuilder, WindowPosition };
use gtk::gio::{ ApplicationFlags };
use gtk::pango::{ EllipsizeMode };

use std::process::{ Command, Stdio, exit };
use std::cell::{ RefCell };
use std::path::{ PathBuf };

// Configuration module
mod config;
mod webclient;
mod setup;

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
  static URL:RefCell<String> = RefCell::new(String::from(""));
  static ICON_SPACING:RefCell<i32> = RefCell::new(0);
);

fn main() {
  let help_message:String = String::from_utf8(include_bytes!("../resources/help.txt").to_vec()).unwrap();

  let configuration:config::Configuration;
  let mut valid_url:bool;
  let mut error_code:i32;
  let argument_list:Vec<String>;
  let argument_count:usize;
  let argument_name:String;

  argument_list = std::env::args().collect();
  argument_count = argument_list.len();
  error_code = -1;
  valid_url = false;

  if argument_count > 1 {
    argument_name = argument_list[1].clone();

    if argument_count == 2 {
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
      } else if webclient::validate_url(&argument_name) {
        URL.with( |v| { *v.borrow_mut() = argument_name });
        valid_url = true;
        error_code = -1;
      } else {
        println!("ERROR: Invalid URL or argument");
        error_code = 1;
      }
    } else {
      error_code = 2;
    }
  }

  match error_code {
    -1 => {
      // Read configuration and store settings in 'thread_local'
      configuration = config::get_configuration();
      if !valid_url { URL.with(|v| { *v.borrow_mut() = configuration.settings.homepage.clone(); }); }
      ICON_SPACING.with(|v| { *v.borrow_mut() = configuration.settings.buttons.spacing.clone(); });

      #[cfg(target_family = "windows")] hide_console_window();
      show_application_window(configuration);
      exit(0);
    },
    0 => { exit(0); },
    _ => {
      println!("{}", help_message);
      exit(error_code);
    }
  }
}

#[cfg(target_family = "windows")]
fn hide_console_window() {
  unsafe {
    winapi::um::wincon::FreeConsole();
  };
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
        .title("BrowseWith")
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
        .title("BrowseWith")
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
  let mut url:String = String::from("");
  let mut args:Vec<&str> = Vec::new();

  URL.with(|v| {url = v.borrow().to_string();});

  if browser_settings.arguments != "" {
    args.push(&browser_settings.arguments);
  }
  args.push(&url);

  Command::new(&browser_settings.executable)
    .args(args.iter())
    .stderr(Stdio::null())
    .stdout(Stdio::null())
    .spawn()
    .expect("failed to execute process");
  application.quit();
}
#[cfg(target_family = "windows")]
fn close_app<'a>(application:&'a Application) {
  application.quit();
}

fn diplay_host_info(max_width:i32) -> Box {
  let box_object:Box;
  let label_url:Label;
  // let label_page_title:Label;
  // let label_ssl_status:Label;
  // let label_response:Label;
  let mut icon_spacing:i32 = 0;
  let mut url:String = String::new();

  // Get variables stored in 'thread_local'
  ICON_SPACING.with(|v| {icon_spacing = *v.borrow();});
  URL.with(|v| {url = v.borrow().to_string();});
  url = format!("Url: {}", url);

  // Create the Label objects
  label_url = Label::builder().label(&url).halign(Align::Start)
    .ellipsize(EllipsizeMode::End)
    .expand(false)
    .width_request(max_width)
    .max_width_chars(30)
    .build();
  // label_page_title = Label::builder().label("Page title:").halign(Align::Start).build();
  // label_ssl_status = Label::builder().label("SSL Statue:").halign(Align::Start).build();
  // label_response = Label::builder().label("Response:").halign(Align::Start).build();

  // Create a Box and add all the labels inside
  box_object = Box::builder()
    .orientation(Orientation::Vertical)
    .spacing(0)
    .margin_top(icon_spacing)
    .margin_start(icon_spacing)
    .margin_bottom(icon_spacing)
    .halign(Align::Start)
    .build();
  box_object.add(&label_url);
  // box_object.add(&label_page_title);
  // box_object.add(&label_ssl_status);
  // box_object.add(&label_response);

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
      .icon_size(gtk::IconSize::LargeToolbar)
      .width_request(width_height)
      .height_request(width_height)
      .build();
  }

  return image;
}
