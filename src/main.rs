use gtk::prelude::*;
use gtk::{ HeaderBar, Application, ApplicationWindow, Button, Image, Box, Orientation, Align, PositionType, Label, WindowPosition };
use gtk::gdk_pixbuf::{ Pixbuf, InterpType };
use gtk::gio::{ ApplicationFlags };
use gtk::glib::{ Bytes };

use std::{ include_bytes };
use std::process::{ Command, Stdio, exit };
use std::cell::{ RefCell };
use std::path::{ Path, PathBuf };
use std::fs::{ write };

// Configuration module
mod config;
mod webclient;
mod setup;

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
  let configuration:config::Configuration;
  let mut error_code:i32;
  let argument_list:Vec<String>;
  let argument_count:usize;
  let argument_name:String;
  let argument_value:String;

  // Read configuration and store settings in 'thread_local'
  configuration = config::get_configuration();
  URL.with(|v| { *v.borrow_mut() = configuration.settings.homepage.clone(); });
  ICON_SPACING.with(|v| { *v.borrow_mut() = configuration.settings.icon_spacing.clone(); });

  argument_list = std::env::args().collect();
  argument_count = argument_list.len();
  error_code = -1;

  if argument_count > 1 {
    argument_name = argument_list[1].clone();
    if argument_count == 3 {
      argument_value = argument_list[2].clone();
    } else {
      argument_value = "".to_string();
    }

    if argument_count == 3 && argument_name == "--set-default" && argument_value != "" {
      match argument_value.as_str() {
        "system" => {
          if setup::is_privileged_user() {
            setup::set_default_browser(true);
            error_code = 0;
          } else {
            println!("This application required elevated privileges to change the default browser systemwide.");
            error_code = 1;
          }
        },
        "user" => {
          setup::set_default_browser(false);
          error_code = 0;
        },
        _ => {
          println!("Invalid option for --set-default ");
          error_code = 2;
        }
      }
    } else if argument_count == 2 {
      if webclient::validate_url(&argument_name) {
        URL.with( |v| { *v.borrow_mut() = argument_name });
        error_code = -1;
      } else {
        println!("Invalid URL");
        error_code = 3;
      }
    } else {
      // FIX: I think that this code is unreachable
      println!("Use: browsewith ARGUMENTS\n\tArguments:\n\t\tURL\t\t\t\tA valid http or https url\n\t\t--set-default [system|user]\tSets browsewith as the default browser system wide or for the current user.\n\t\t\t\t\t\tNote: BrowseWith needs to be executed with elevated privileges to set as the default browser systemwide.");
      error_code = 4;
    }
  }

  match error_code {
    -1 => { show_application_window(configuration); }
    0 => { exit(0); },
    _ => { exit(error_code); }
  }
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
    let icons_per_row:i32 = configuration.settings.icons_per_row;
    let icon_spacing:i32 = configuration.settings.icon_spacing;
    let icon_spacing_top:i32 = configuration.settings.icon_spacing;

    let button_margin_default:ButtonMargins = ButtonMargins { left: icon_spacing, top: icon_spacing_top, right: 0, bottom: 0 };
    let button_margin_last:ButtonMargins = ButtonMargins { left: icon_spacing, top: icon_spacing, right: icon_spacing, bottom: 0 };

    let window = ApplicationWindow::builder()
      .application(app)
      .title("BrowseWith")
      .default_width(180+icon_spacing*2)
      .default_height(70)
      .window_position(WindowPosition::Center)
      .build();

    // Add all browsers as icons to a Box widget, creating a new child Box widget
    // for every 'icons_per_row' browsers
    icons_box.add(&icons_row);
    for browser in configuration.browsers_list.clone() {
      if icon_counter % icons_per_row == 0 {
        button_with_image(&app, &browser, &icons_row, button_margin_last);
        icons_row = Box::new(Orientation::Horizontal, 0);
        icons_box.add(&icons_row);
      } else {
        button_with_image(&app, &browser, &icons_row, button_margin_default);
      }
      icon_counter = icon_counter + 1;
    }
    window_box.add(&icons_box);

    // Check if we need to add taget URL host information
    if configuration.settings.host_info {
      hostinfo_box = diplay_host_info();
      window_box.add(&hostinfo_box);
    }

    // Build a title bar
    header_bar = HeaderBar::builder()
      .title("BrowseWith")
      .decoration_layout("menu:close")
      .show_close_button(true)
      .build();

    // Traits from GtkWindowExt
    window.set_keep_above(true);
    window.set_resizable(false);
    window.set_titlebar(Some(&header_bar));

    load_icon();

    // Display main windows with all the components
    window.add(&window_box);
    window.show_all();
  
  });

  application.run();
}

fn button_with_image(application:&Application, browser_settings:&config::BrowserSettings, box_object:&Box, margins:ButtonMargins) {
  let browser_settings_clone:config::BrowserSettings;
  let application_clone:Application;
  let image:Image;
  let button:Button;
  let mut image_pixbuf:Pixbuf;

  // Clone application and browser_settings so we can pass them to 
  // the closure in button connect_clicked
  application_clone = application.clone();
  browser_settings_clone = browser_settings.clone();

  // Create a button with image and label, assigning a function for when clicked
  image_pixbuf = Pixbuf::from_file(browser_settings.icon.clone()).unwrap();
  if image_pixbuf.width() != 32 && image_pixbuf.height() != 32 {
    image_pixbuf = image_pixbuf.scale_simple(32, 32, InterpType::Bilinear ).unwrap();
  }
  image = Image::from_pixbuf(Some(&image_pixbuf));
  button = Button::builder()
    .width_request(180).height_request(60)
    .image(&image).always_show_image(true).image_position(PositionType::Left)
    .label(&browser_settings.title)  
    .margin_start(margins.left)
    .margin_top(margins.top)
    .margin_end(margins.right)
    .margin_bottom(margins.bottom)
    .build();
  button.connect_clicked(move |_| {button_clicked(&application_clone, &browser_settings_clone)});

  // Add to the main window
  box_object.add(&button);
}

fn button_clicked<'a>(application:&Application, browser_settings:&'a config::BrowserSettings ) {
  let mut url:String = String::from("");
  URL.with(|v| {url = v.borrow().to_string();});
  Command::new(&browser_settings.executable)
    .arg(&browser_settings.arguments)
    .arg(url)
    .stderr(Stdio::null())
    .stdout(Stdio::null())
    .spawn()
    .expect("failed to execute process");
  application.quit();
}

fn diplay_host_info() -> Box {
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
  label_url = Label::builder().label(&url).halign(Align::Start).build();
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

fn load_icon() {
  let mut home_dir_buf:PathBuf;
  let icon_file_path:&Path;
  let icon_file:Pixbuf;
  let icon_raw:&[u8];
  let icon_bytes:Bytes;

  // Load the icon file as '[u8]' at compile time
  icon_raw = include_bytes!("../resources/browsewith.ico");
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