use std::io;

#[cfg(windows)]
use winres::{ WindowsResource };

fn main() -> io::Result<()> {
  #[cfg(windows)] {
    WindowsResource::new()
      .set_icon_with_id("resources/browsewith.ico", "3")
    .compile()?;
  }
  Ok(())
}