use gtk::prelude::*;
use gtk::{ Application, ApplicationWindow, Button, Image, Box, Orientation, Align, PositionType, Label, WindowPosition };
use gtk::gdk_pixbuf::{ Pixbuf, InterpType };
use std::process::{ Command };

// Configuration module
mod config;

#[derive(Clone, Copy)]
struct ButtonMargins {
  left: i32,
  top: i32,
  right: i32,
  bottom: i32
}

fn main() {
  let configuration:config::Configuration;

  configuration = config::get_configuration();
  show_application_window(configuration);
}

fn show_application_window(configuration:config::Configuration) {
  let application = Application::builder()
  .application_id("com.sheep.browsewith")
  .build();

  application.connect_activate(move |app| {
    let window_box:Box = Box::new(Orientation::Vertical, 0);
    let icons_box:Box = Box::new(Orientation::Vertical, 0);
    let hostinfo_box:Box;
    let mut icons_row:Box = Box::new(Orientation::Horizontal, 0);
    let mut icon_counter:i32 = 1;
    let icons_per_row:i32 = configuration.settings.icons_per_row;
    let icon_spacing:i32 = configuration.settings.icon_spacing;
    let mut icon_spacing_top:i32 = configuration.settings.icon_spacing;

    let button_margin_default:ButtonMargins = ButtonMargins { left: icon_spacing, top: icon_spacing_top, right: 0, bottom: icon_spacing };
    let button_margin_last:ButtonMargins = ButtonMargins { left: icon_spacing, top: icon_spacing, right: icon_spacing, bottom: icon_spacing };

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
        icon_spacing_top = 0;
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
  let button_box:Box;
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
    .build();
  button.connect_clicked(move |_| {button_clicked(&application_clone, &browser_settings_clone)});

  //
  button_box = Box::builder()
    .orientation(Orientation::Horizontal)
    .spacing(0)
    .margin_start(margins.left)
    .margin_top(margins.top)
    .margin_end(margins.right)
    .margin_bottom(margins.bottom)
    .build();
  button_box.add(&button);

  // Add to the main window
  box_object.add(&button_box);
}

fn button_clicked<'a>(application:&Application, browser_settings:&'a config::BrowserSettings ) {
  eprintln!("{:?}", &browser_settings.executable);
  Command::new(&browser_settings.executable)
    .arg(&browser_settings.arguments)
    .spawn()
    .expect("failed to execute process");
  application.quit();
}

fn diplay_host_info() -> Box {
  let box_object:Box;
  let label_url:Label;
  let label_page_title:Label;
  let label_ssl_status:Label;
  let label_response:Label;

  label_url = Label::builder().label("URL:").halign(Align::Start).build();
  label_page_title = Label::builder().label("Page title:").halign(Align::Start).build();
  label_ssl_status = Label::builder().label("SSL Statue:").halign(Align::Start).build();
  label_response = Label::builder().label("Response:").halign(Align::Start).build();

  box_object = Box::builder()
    .orientation(Orientation::Vertical)
    .spacing(0)
    .margin_start(5)
    .margin_bottom(5)
    .halign(Align::Start)
    .build();
  box_object.add(&label_url);
  box_object.add(&label_page_title);
  box_object.add(&label_ssl_status);
  box_object.add(&label_response);

  return box_object;
}