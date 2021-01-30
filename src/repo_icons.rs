use super::Readme;
use crate::blacklist::{is_badge, is_blacklisted_homepage};
use site_icons::{IconInfo, IconKind, Icons};
use std::{
  error::Error,
  fmt::{self, Display},
};
use url::Url;

#[derive(Debug, Serialize, PartialOrd, PartialEq, Ord, Eq)]
pub enum RepoIconKind {
  ReadmeImage,
  Site(IconKind),
}

impl Display for RepoIconKind {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match self {
      RepoIconKind::ReadmeImage => write!(f, "readme_image"),
      RepoIconKind::Site(kind) => write!(f, "{}", kind),
    }
  }
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct RepoIcon {
  pub url: Url,
  #[serde(with = "serde_with::rust::display_fromstr")]
  pub kind: RepoIconKind,
  #[serde(flatten)]
  pub info: IconInfo,
}

/// Fetch all the icons. Ordered from highest to lowest resolution
///
/// ```
/// # async fn run() {
/// let icons = get_repo_icons("facebook", "react").await?;
///
/// for icon in icons {
///   println("{:?}", icon)
/// }
/// ```
pub async fn get_repo_icons(user: &str, repo: &str) -> Result<Vec<RepoIcon>, Box<dyn Error>> {
  let readme = Readme::load(user, repo).await?;
  let mut icons = Icons::new();

  let readme_image = readme.images().await.into_iter().find(|image| {
    if image.in_primary_heading {
      icons.add_icon(image.src.clone(), IconKind::SiteLogo, None);
      true
    } else {
      false
    }
  });

  if let Some(homepage) = &readme.homepage {
    if !is_blacklisted_homepage(homepage) {
      warn_err!(
        icons.load_website(homepage.clone()).await,
        "failed to load website {}",
        homepage
      );
    }
  }

  let entries = icons.entries().await;

  let repo_icons: Vec<_> = entries
    .into_iter()
    .filter(|icon| !is_badge(&icon.url))
    .map(|entry| {
      let is_readme = readme_image
        .as_ref()
        .map(|image| image.src == entry.url)
        .unwrap_or(false);

      RepoIcon {
        url: entry.url,
        info: entry.info,
        kind: if is_readme {
          RepoIconKind::ReadmeImage
        } else {
          RepoIconKind::Site(entry.kind)
        },
      }
    })
    .collect();

  Ok(repo_icons)
}
