use super::Readme;
use crate::{
  blacklist::{is_badge, is_blacklisted_homepage},
  get_token, RepoIcon, RepoIconKind,
};
use reqwest::{
  header::{HeaderMap, HeaderValue, AUTHORIZATION},
  Client, IntoUrl,
};
use site_icons::{IconKind, Icons};
use std::{
  cmp::{max, min, Ordering},
  collections::HashMap,
  convert::TryInto,
  error::Error,
};
use vec1::Vec1;

#[derive(Debug, Serialize, Deserialize)]
pub struct RepoIcons(Vec1<RepoIcon>);

impl RepoIcons {
  /// Fetch all the icons. Ordered from highest to lowest resolution
  ///
  /// ```
  /// # async fn run() {
  /// let icons = RepoIcons::load("facebook", "react").await?;
  ///
  /// for icon in icons {
  ///   println("{:?}", icon)
  /// }
  /// ```
  pub async fn load(user: &str, repo: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
    let mut icons = Icons::new();

    let readme_image = {
      let readme = Readme::load(user, repo).await?;

      if let Some(homepage) = &readme.homepage {
        if !is_blacklisted_homepage(homepage) {
          warn_err!(
            icons.load_website(homepage.clone()).await,
            "failed to load website {}",
            homepage
          );
        }
      }

      readme.images().await.into_iter().find(|image| {
        if image.in_primary_heading {
          icons.add_icon(image.src.clone(), IconKind::SiteLogo, None);
          true
        } else {
          false
        }
      })
    };

    let entries = icons.entries().await;

    let mut repo_icons: Vec1<RepoIcon> = entries
      .into_iter()
      .filter(|icon| !is_badge(&icon.url))
      .map(|entry| {
        let is_readme = readme_image
          .as_ref()
          .map(|image| image.src == entry.url)
          .unwrap_or(false);

        RepoIcon::new(
          entry.url,
          if is_readme {
            RepoIconKind::ReadmeImage
          } else {
            RepoIconKind::Site(entry.kind)
          },
          entry.info,
        )
      })
      .collect::<Vec<_>>()
      .try_into()
      .map_err(|_| "no icons found for repo")?;

    repo_icons.sort_by(|a, b| {
      if a.kind == RepoIconKind::ReadmeImage {
        Ordering::Less
      } else if b.kind == RepoIconKind::ReadmeImage {
        Ordering::Greater
      } else {
        Ordering::Equal
      }
    });

    Ok(RepoIcons(repo_icons))
  }

  /// Fetch all icons using an API endpoint. Ordered from highest to lowest resolution
  ///
  /// ```
  /// # async fn run() {
  /// let icons = RepoIcons::fetch("https://repo-icons.api.com", "facebook", "react").await?;
  ///
  /// for icon in icons {
  ///   println("{:?}", icon)
  /// }
  /// ```
  pub async fn fetch<U: IntoUrl>(
    endpoint: U,
    user: &str,
    repo: &str,
  ) -> Result<Self, Box<dyn Error>> {
    let endpoint = endpoint
      .into_url()?
      .join(&format!("{}/{}/icons", user, repo))?;

    let mut headers = HeaderMap::new();
    if let Some(token) = get_token() {
      headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("token {}", token))?,
      );
    }

    let repo_icons = Client::new()
      .get(endpoint)
      .headers(headers)
      .send()
      .await?
      .error_for_status()?
      .json()
      .await?;

    Ok(repo_icons)
  }

  pub fn get_thumbnail_sizes(&self, resolutions: &[u32]) -> Vec<(u32, &RepoIcon)> {
    let mut resolutions = resolutions.to_vec();
    resolutions.sort_by(|a, b| b.cmp(a));

    let mut thumbnails = HashMap::new();

    for icon in self.0.iter() {
      if let Some(sizes) = icon.info.sizes() {
        for icon_size in sizes.iter().map(|size| size.max_rect()) {
          let mut resolutions = resolutions.iter().peekable();

          while let Some(&resolution) = resolutions.next() {
            let existing_thumbnail = thumbnails.get(&resolution);

            let is_better_fit = |existing_size| {
              let existing_fit =
                min(existing_size, resolution) as f32 / max(existing_size, resolution) as f32;
              let fit = min(icon_size, resolution) as f32 / max(icon_size, resolution) as f32;
              fit > existing_fit
            };

            if {
              // if the existing thumbnail is already the perfect
              // resolution then don't replace it
              if existing_thumbnail
                .map(|(_, existing_size)| *existing_size == resolution)
                .unwrap_or(false)
              {
                false
              }
              // if the icon is the perfect resolution
              else if icon_size == resolution {
                true
              }
              // if the icon is oversized
              else if icon_size > resolution {
                existing_thumbnail
                  .map(|(_, existing_size)| is_better_fit(*existing_size))
                  .unwrap_or(true)
              }
              // if the icon is undersized
              else if resolutions
                .peek()
                .map(|&&next_resolution| {
                  let pixel_loss =
                    1.0 - (resolution - icon_size) as f32 / (resolution - next_resolution) as f32;
                  pixel_loss >= 0.3
                })
                .unwrap_or(false)
              {
                existing_thumbnail
                  .map(|(_, existing_size)| is_better_fit(*existing_size))
                  .unwrap_or(true)
              } else {
                false
              }
            } {
              thumbnails.insert(resolution, (icon, icon_size));
            };
          }
        }
      }
    }

    let mut sizes = Vec::new();
    let mut seen_thumbnails = Vec::new();
    for resolution in resolutions {
      if let Some(thumbnail) = thumbnails.get(&resolution) {
        if seen_thumbnails.contains(&thumbnail) {
          // info!("seen {:#?} {:#?}", seen_thumbnails, thumbnail);
          continue;
        }

        seen_thumbnails.insert(0, thumbnail);
        sizes.push((resolution, thumbnail.0));
      }
    }

    sizes
  }

  pub fn get_size(&self, width: u32, height: u32) -> &RepoIcon {
    for icon in self.0.iter().rev() {
      if let Some(size) = icon.info.size() {
        if size.width >= width || size.height >= height {
          return icon;
        }
      }
    }

    self.largest()
  }

  pub fn largest(&self) -> &RepoIcon {
    self.0.first()
  }

  pub fn smallest(&self) -> &RepoIcon {
    self.0.last()
  }
}

// impl Deref for RepoIcons {
//   type Target = Vec1<RepoIcon>;
//   fn deref(&self) -> &Vec1<RepoIcon> {
//     &self.0
//   }
// }

impl IntoIterator for RepoIcons {
  type Item = RepoIcon;
  type IntoIter = std::vec::IntoIter<Self::Item>;

  fn into_iter(self) -> Self::IntoIter {
    self.0.into_iter()
  }
}
