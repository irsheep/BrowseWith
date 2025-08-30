use gtk::{
  prelude::*,
  Application,
  ApplicationWindow,
  Align,
  Orientation,
  Grid,
  Box,
  Button,
  Image,
  Label,
  MessageType,
  ButtonsType,
  Window,
  AccessibleRole,
  pango::WrapMode,
  IconSize,
  HeaderBar,
};

type Callback = fn();

pub struct MessageWindow<'a> {
  pub callback_ok: Option<Callback>,
  pub callback_close: Option<Callback>,
  pub callback_cancel: Option<Callback>,
  pub callback_yes: Option<Callback>,
  pub callback_no: Option<Callback>,
  pub icon_name: Option<&'a str>,
  pub lbl_ok: &'a str,
  pub lbl_close: &'a str,
  pub lbl_cancel: &'a str,
  pub lbl_yes: &'a str,
  pub lbl_no: &'a str,
}

impl Default for MessageWindow<'_> {
  fn default() -> Self {
    Self {
      callback_ok: None,
      callback_close: None,
      callback_cancel: None,
      callback_yes: None,
      callback_no: None,
      icon_name: None,
      lbl_ok: "_Ok",
      lbl_close: "Clo_se",
      lbl_cancel: "_Cancel",
      lbl_yes: "_Yes",
      lbl_no: "_No",
    }
  }
}

impl MessageWindow<'_> {

  pub fn show(&self, application:&Application, parent:Option<&ApplicationWindow>, title:&str, message:&str, message_type:MessageType, buttons:ButtonsType) {

    let spacing:i32 = 0;
    let container_width:i32 = 350;
    let width_height:i32 = 64;

    let window = Window::builder()
      .application(application)
      .accessible_role(AccessibleRole::AlertDialog)
      .modal(true)
    .build();
    let title_bar = HeaderBar::builder()
      .show_title_buttons(false)
    .build();
    
    let mut icon_name:&str = match message_type {
      MessageType::Info => "dialog-information",
      MessageType::Warning => "dialog-warning",
      MessageType::Question => "dialog-question",
      MessageType::Error => "dialog-error",
      _ => "dialog-info"
    };

    match self.icon_name {
      Some(n) => { icon_name = n; },
      None => {}
    }

    let grid = Grid::builder()
      .margin_start(spacing)
      .margin_end(spacing)
      .margin_top(spacing)
      .margin_bottom(spacing)
      .halign(Align::Center)
      .hexpand(true).hexpand_set(true)
      .valign(Align::Center)
      .row_spacing(spacing)
      .column_spacing(spacing)
    .build();
    let container_top = Box::builder()
      .orientation(Orientation::Horizontal)
      .width_request(container_width)
      .margin_top(5).margin_bottom(5)
      .spacing(5)
    .build();
    let container_bottom = Box::builder()
      .orientation(Orientation::Horizontal)
      .width_request(container_width)
      .homogeneous(true)
    .build();
    let icon = Image::builder()
      .icon_name(icon_name)
      .icon_size(IconSize::Large)
      .width_request(width_height)
      .height_request(width_height)
    .build();
    let label = Label::builder()
      .label(message)
      .wrap(true).wrap_mode(WrapMode::Word)
      .width_request(container_width-width_height-5)
      .margin_end(5)
    .build();

    macro_rules! add_button_with_callback {
      ($callback:expr, $label:expr) => {
        let button = Button::builder()
          .label($label)
          .use_underline(true)
          .hexpand(true).hexpand_set(true)
        .build();
        match $callback {
          Some(cb) => {
            let cb_clone = cb.clone();
            let window_clone = window.clone();
            container_bottom.append(&button);
            button.connect_clicked(move |_| {
              cb_clone();
              window_clone.close();
            });
          },
          None => {}
        }
      }
    }

    match buttons {
      ButtonsType::Ok => { add_button_with_callback!(self.callback_ok, self.lbl_ok); }
      ButtonsType::Close => { add_button_with_callback!(self.callback_close, self.lbl_close); }
      ButtonsType::Cancel => { add_button_with_callback!(self.callback_cancel, self.lbl_cancel); }
      ButtonsType::YesNo => {
        add_button_with_callback!(self.callback_yes, self.lbl_yes);
        add_button_with_callback!(self.callback_no, self.lbl_no);
      }
      ButtonsType::OkCancel => {
        add_button_with_callback!(self.callback_ok, self.lbl_ok);
        add_button_with_callback!(self.callback_cancel, self.lbl_cancel);
      }
      _ => {}
    }

    container_top.append(&icon);
    container_top.append(&label);

    grid.attach(&container_top,0, 0, 1, 1);
    grid.attach(&container_bottom, 0, 1, 1, 1);

    window.set_transient_for(parent);
    window.set_title(Some(title));
    window.set_default_size(container_width, 0);
    window.set_child(Some(&grid));
    window.set_titlebar(Some(&title_bar));
    window.present();
  }

}

