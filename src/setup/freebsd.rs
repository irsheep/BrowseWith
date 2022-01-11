use nix::unistd::{ Uid, getuid };

pub fn set_default_browser(system_wide:bool) {
  println!("This feature isn't supported in FreeBSD, to set browsewith as the default browser you need to use the tools provided by the display manager installed on your system\nNo changes were made.");
}

pub fn is_privileged_user() -> bool {
  let uid:Uid;
  uid = getuid();
  return uid.is_root();
}

pub fn load_icon() {

}