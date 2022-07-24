#![feature(async_closure)]

//! # repo_icons
//! Get icons for a GitHub repository.
//!
//! ## Usage
//! ```rust
//! use repo_icons::RepoIcons;
//!
//! let icons = RepoIcons::load("facebook", "react").await?;
//!
//! for icon in icons {
//!   println("{:?}", icon)
//! }
//! ```

#[macro_use]
extern crate log;
#[macro_use]
extern crate gh_api;
#[macro_use]
extern crate derivative;
#[macro_use]
extern crate serde_with;
#[macro_use]
extern crate futures;

#[macro_use]
mod macros;
mod blacklist;
mod readme;
mod repo_icon;
mod repo_icons;

pub use gh_api::*;
pub use readme::*;
pub use repo_icon::*;
pub use repo_icons::*;
