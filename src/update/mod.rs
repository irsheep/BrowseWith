use serde::{ Serialize, Deserialize };
use version_compare::{ Cmp, Version };
use std::{
  path::{ Path, PathBuf },
  fs::{ File },
  io::{ BufReader, BufWriter }
};

#[allow(dead_code)]
#[derive(Deserialize)]
struct GitRelease {
  url:String,
  assets_url:String,
  upload_url:String,
  html_url:String,
  id: i32,
  author:GitAutor,
  node_id:String,
  tag_name:String,
  target_commitish:String,
  name:String,
  draft:bool,
  prerelease:bool,
  created_at:String,
  published_at:String,
  assets:Vec<GitAssets>,
  tarball_url:String,
  zipball_url:String,
  body:String
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct GitAutor {
  login:String,
  id:i32,
  node_id:String,
  avatar_url:String,
  gravatar_id:String,
  url:String,
  html_url:String,
  followers_url:String,
  following_url:String,
  gists_url:String,
  starred_url:String,
  subscriptions_url:String,
  organizations_url:String,
  repos_url:String,
  events_url:String,
  received_events_url:String,
  #[serde(rename = "type")]
  type_:String,
  site_admin:bool
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct GitAssets {
  url:String,
  id:i32,
  node_id:String,
  name:String,
  label:Option<String>,
  uploader:GitAutor,
  content_type:String,
  state:String,
  size:i32,
  download_count:i32,
  created_at:String,
  updated_at:String,
  browser_download_url:String
}

#[derive(Debug,Serialize,Deserialize,Clone)]
pub enum Platform {
  Windows,
  Linux,
  Bsd
}

#[derive(Debug,Serialize,Deserialize,Clone)]
pub struct Releases {
  pub is_newer:bool,
  pub version:String,
  pub download_urls:Vec<PlatformDownloadUrl>,
  pub html_url:String
}
#[derive(Debug,Serialize,Deserialize,Clone)]
pub struct PlatformDownloadUrl {
  pub platform:Platform,
  pub url:String
}
#[derive(Debug,Serialize,Deserialize,Clone)]
pub struct ReleaseDownloadUrls {
  pub html:String,
  pub file:String,
  pub hash:String
}

impl Releases {

  pub fn new() -> Option<Self> {
    // println!("{}:{} Releases::new", file!(), line!());
    std::thread::sleep(std::time::Duration::from_millis(5000));
    return Self::get_latest_version();
  }

  pub fn initialize() -> Releases {
    // println!("{}:{} Releases::initialize", file!(), line!());
    return Releases {
      is_newer:false,
      version:String::from(""),
      download_urls:vec![],
      html_url:String::from("")
    };
  }

  #[allow(dead_code)]
  pub fn get_platform_urls(&self) -> ReleaseDownloadUrls {
    let mut update_download_links:ReleaseDownloadUrls = ReleaseDownloadUrls {html: String::from(""), file: String::from(""), hash: String::from("")};

    for link in &self.download_urls {
      if
        link.url.contains(std::env::consts::OS) ||
        // For BSD OS family OpenBSD, FreeBSD and NetBSD
        ( std::env::consts::OS.contains("bsd") && link.url.contains("bsd") )
      {
        if link.url.contains(".sha256") {
          update_download_links.hash = link.url.clone();
        } else {
          update_download_links.file = link.url.clone();
        }
      }
    }
    update_download_links.html = self.html_url.clone();
    return update_download_links;
  }

  #[allow(dead_code)]
  pub fn download(url:&String, output:&String) {
    match ureq::get(url).call() {
      Ok(response) => {
        let mut output_file:File = Self::create_output_file(output).unwrap();
        std::io::copy(&mut response.into_reader(), &mut output_file).unwrap();
      },
      _ => {}
    }
  }

  fn get_latest_version() -> Option<Releases> {
    // println!("{}:{} Releases::get_latest_version", file!(), line!());

    let releases:Vec<GitRelease> = match Self::download_version_data() {
      Some(data) => data,
      _ => return None
    };

    let release:&GitRelease = &releases[0];

    let cargo_version:String = format!("v{}",env!("CARGO_PKG_VERSION"));
    let current_version = Version::from(&cargo_version).unwrap();

    let latest_version = Version::from(&release.tag_name).unwrap();
    let mut platform_download_urls: Vec<PlatformDownloadUrl> = vec![];

    match current_version.compare(latest_version) {
      // Current version is lesser than latest Git release
      Cmp::Lt => {
        // println!("current is LESS THAN latest");
        for asset in &release.assets {
          if asset.browser_download_url.contains("windows-") {
            platform_download_urls.push(PlatformDownloadUrl{
              platform: Platform::Windows,
              url: asset.browser_download_url.clone()
            });
          }
          if asset.browser_download_url.contains("linux-") {
            platform_download_urls.push(PlatformDownloadUrl{
              platform: Platform::Linux,
              url: asset.browser_download_url.clone()
            });
          }
          if asset.browser_download_url.contains("bsd-") {
            platform_download_urls.push(PlatformDownloadUrl{
              platform: Platform::Bsd,
              url: asset.browser_download_url.clone()
            });
          }
        }
        return Some(Releases {
          is_newer: true,
          version: release.tag_name.clone(),
          download_urls: platform_download_urls,
          html_url: release.html_url.clone()
        });
      },
      Cmp::Eq => {
        // println!("current is EQUAL to latest");
        return Some(Releases {
          is_newer: false,
          version: release.tag_name.clone(),
          download_urls: platform_download_urls,
          html_url: release.html_url.clone()
        });
      },
      Cmp::Gt => {
        // println!("current is GREATER THAN latest");
        return Some(Releases {
          is_newer: false,
          version: release.tag_name.clone(),
          download_urls: platform_download_urls,
          html_url: release.html_url.clone()
        });
      }
      _ => return None
    }
  }

  fn download_version_data() -> Option<Vec<GitRelease>> {
    match ureq::get("https://api.github.com/repos/irsheep/browsewith/releases").call() {
      Ok(response) => { return Self::parse_git_data(response); },
      _ => { return None; }
    }
  }

  fn parse_git_data(git_data:ureq::Response) -> Option<Vec<GitRelease>> {
    match git_data.into_json() {
      Ok(data) => { return Some(data); },
      _ => { return None; }
    }
  }

  fn create_output_file(output_filename: &str) -> Result<File, String> {
    let output_file_path:&Path = Path::new(output_filename);

    match File::create(output_file_path) {
      Ok(file) => Ok(file),
      Err(error) => Err(format!(
        "Failed to open file {} for writing; cause: {}",
        output_filename, error
      )),
    }
  }
}

pub fn write_check_file(file:&PathBuf, data:Releases) {
  let file_handle:File = File::create(file).unwrap();
  let writer:BufWriter<File> = BufWriter::new(file_handle);
  match serde_json::to_writer_pretty(writer, &data) {
    Ok(_) => { },
    _ => { println!("Failed to parse Release data"); }
  }
}

pub fn read_check_file(file:&PathBuf) -> Releases {
  let file_handle:File = File::open(&file).unwrap();
  let reader:BufReader<File> = BufReader::new(file_handle);
  let mut git_release:Releases = serde_json::from_reader(reader).unwrap();

  // Get current and latest version numbers
  let cargo_version:String = format!("v{}",env!("CARGO_PKG_VERSION"));
  let current_version = Version::from(&cargo_version).unwrap();
  let latest_version = Version::from(&git_release.version).unwrap();
  // println!("{}:{} \nCurrent: {}\nLatest: {}", file!(), line!(), current_version, latest_version);

  match current_version.compare(latest_version) {
    Cmp::Lt => {
      git_release.is_newer = true;
    },
    _ => { }
  }

  // println!("{:#?}", git_release);
  return git_release;
}
