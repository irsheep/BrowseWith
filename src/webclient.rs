use reqwest::{ Url };

pub fn validate_url(request_url:&str) -> bool {
  match Url::parse(request_url) {
    Ok(..) => return true,
    Err(..) => return false
  };
}
