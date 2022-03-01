use url::{ Url };

pub fn validate_url(request_url:&str) -> bool {
  let supported_schemes:Vec<String> = vec!["http".to_string(), "https".to_string(), "ftp".to_string(), "file".to_string()];

  match Url::parse(request_url) {
    Ok(url) => {
      if supported_schemes.contains(&url.scheme().to_lowercase()) {
        return true;
      }
      return false;
    },
    Err(..) => { return false }
  };
}
