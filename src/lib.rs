#![feature(async_closure, map_into_keys_values, bool_to_option)]

//! # repo_icons
//! Get icons for a GitHub repository.
//!
//! ## Usage
//! ```rust
//! use site_icons::get_repo_icons;
//!
//! let icons = get_repo_icons("facebook", "react").await?;
//!
//! for icon in icons {
//!   println("{:?}", icon)
//! }
//! ```

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_with;
#[macro_use]
extern crate futures;

#[macro_use]
mod macros;
mod blacklist;
mod readme;
mod repo_icons;

pub use readme::*;
pub use repo_icons::*;

use once_cell::sync::Lazy;
use reqwest::{header::*, Client, ClientBuilder};

pub fn client_builder() -> ClientBuilder {
  let mut headers = HeaderMap::new();
  headers.insert(USER_AGENT, HeaderValue::from_str("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/88.0.4324.104 Safari/537.36").unwrap());

  if let Some(token) = get_token() {
    headers.insert(
      AUTHORIZATION,
      HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
    );
  }

  Client::builder().default_headers(headers)
}

static mut TOKEN: Option<String> = None;
pub static mut CLIENT: Lazy<Client> = Lazy::new(|| client_builder().build().unwrap());

pub fn get_token() -> Option<&'static String> {
  unsafe { TOKEN.as_ref() }
}

pub fn set_token<T: ToString>(token: T) {
  unsafe {
    TOKEN = Some(token.to_string());
    *CLIENT = client_builder().build().unwrap()
  };
}

#[macro_export]
macro_rules! github_api_get {
  ($client:expr, $fmt:literal, $($arg:tt)*) => {
    $client.get(&format!("https://api.github.com/{}", format!($fmt, $($arg)*)))
  };

  ($fmt:literal, $($arg:tt)*) => {{
    $crate::github_api_get!(unsafe { &*$crate::CLIENT }, $fmt, $($arg)*)
  }}
}
