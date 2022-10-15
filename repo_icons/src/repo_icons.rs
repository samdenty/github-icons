use crate::{
  get_token,
  github_api::{self, owner_name_lowercase},
  RepoIcon, RepoIconKind,
};
use async_recursion::async_recursion;
use futures::{
  future::{join_all, select_all},
  Future, FutureExt,
};
use itertools::Itertools;
use reqwest::{
  header::{HeaderMap, HeaderValue, AUTHORIZATION},
  Client, IntoUrl, Url,
};
use site_icons::Icons;
use std::{
  cmp::{max, min},
  collections::HashMap,
  convert::TryInto,
  error::Error,
  pin::Pin,
};
use vec1::Vec1;

#[derive(Debug, Serialize, Deserialize)]
pub struct RepoIcons(Vec1<RepoIcon>);

#[derive(Clone)]
enum LoadedKind {
  UserAvatar(Option<RepoIcon>),
  Blob(Option<RepoIcon>),
  ReadmeImage(Option<RepoIcon>),
  Homepage(Vec<RepoIcon>),
  PrefixedRepo(Vec<RepoIcon>),
}

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
  #[async_recursion(?Send)]
  pub async fn load(
    owner: &str,
    repo: &str,
    best_matches_only: bool,
  ) -> Result<Self, Box<dyn Error>> {
    let mut repo_icons = Vec::new();

    let readme = github_api::Readme::load(owner, repo).shared();

    let mut futures: Vec<Pin<Box<dyn Future<Output = Result<LoadedKind, Box<dyn Error>>>>>> = vec![
      async {
        let user_avatar_url: Url = format!("https://github.com/{}.png", owner).parse().unwrap();

        // Check if the repo contains the owner's username, and load the user's avatar
        let icon = if repo.to_lowercase().contains(&owner_name_lowercase(owner)) {
          Some(RepoIcon::load(user_avatar_url.clone(), RepoIconKind::UserAvatar).await?)
        } else {
          None
        };

        Ok(LoadedKind::UserAvatar(icon))
      }
      .boxed_local(),
      // Try and find prefixed repos, and load icons for them on GitHub
      async {
        let repos = github_api::get_user_repos(owner).await?;

        Ok(LoadedKind::PrefixedRepo(
          join_all(
            repos
              .into_iter()
              .filter(|possibly_prefixed_repo| {
                possibly_prefixed_repo != &repo.to_lowercase()
                  && repo.to_lowercase().contains(possibly_prefixed_repo)
              })
              .map(async move |repo| {
                RepoIcons::load(owner, &repo, best_matches_only)
                  .await
                  .map(|icons| icons.0.into_vec())
                  .unwrap_or(Vec::new())
              }),
          )
          .await
          .into_iter()
          .flatten()
          .collect(),
        ))
      }
      .boxed_local(),
      async {
        let blob_icon = match github_api::get_blob(owner, repo).await? {
          Some((is_icon_field, blob)) => Some(RepoIcon::load_blob(blob, is_icon_field).await?),
          None => None,
        };

        Ok(LoadedKind::Blob(blob_icon))
      }
      .boxed_local(),
      async {
        let mut icons = Icons::new();

        if let Some(homepage) = readme.clone().await?.homepage {
          warn_err!(
            icons.load_website(homepage.clone()).await,
            "failed to load website {}",
            homepage
          );
        }

        Ok(LoadedKind::Homepage(
          icons
            .entries()
            .await
            .into_iter()
            .map(|icon| RepoIcon::new(icon.url, RepoIconKind::Site(icon.kind), icon.info))
            .collect(),
        ))
      }
      .boxed_local(),
      // Try and extract images from the readme website, or directly in it
      async {
        let image = readme
          .clone()
          .await?
          .load_body()
          .await?
          .into_iter()
          .find(|image| image.in_primary_heading);

        let icon = match image {
          Some(image) => Some(
            RepoIcon::load_with_headers(image.src, image.headers, RepoIconKind::ReadmeImage)
              .await?,
          ),
          None => None,
        };

        Ok(LoadedKind::ReadmeImage(icon))
      }
      .boxed_local(),
    ];

    let mut previous_loads = Vec::new();
    let mut found_best_match = false;

    while !futures.is_empty() {
      let (loaded, index, _) = select_all(&mut futures).await;
      futures.remove(index);
      let loaded = loaded?;

      match &loaded {
        LoadedKind::Blob(blob_icon) => {
          if let Some(mut blob_icon) = blob_icon.clone() {
            blob_icon.set_repo_private(readme.clone().await?.private);

            if matches!(blob_icon.kind, RepoIconKind::IconField(_)) {
              found_best_match = true;
            }

            repo_icons.push(blob_icon);
          }
        }

        LoadedKind::UserAvatar(user_avatar) => {
          if previous_loads
            .iter()
            .any(|loaded| matches!(loaded, LoadedKind::Blob(_)))
          {
            found_best_match = true;
          }

          if let Some(user_avatar) = user_avatar {
            found_best_match = true;

            repo_icons.push(user_avatar.clone());
          }
        }

        LoadedKind::ReadmeImage(readme_image) => {
          if let Some(readme_image) = readme_image {
            if previous_loads
              .iter()
              .any(|loaded| matches!(loaded, LoadedKind::UserAvatar(_)))
              && previous_loads
                .iter()
                .any(|loaded| matches!(loaded, LoadedKind::Blob(_)))
            {
              found_best_match = true;
            }

            repo_icons.push(readme_image.clone());
          }
        }

        LoadedKind::PrefixedRepo(icons) => {
          repo_icons.extend(icons.clone());
        }

        LoadedKind::Homepage(_site_icons) => {}
      }

      previous_loads.push(loaded);

      repo_icons.sort_by(|a, b| a.info.cmp(&b.info));
      repo_icons.sort_by(|a, b| a.kind.cmp(&b.kind));

      if best_matches_only && found_best_match {
        break;
      }
    }

    let repo_icons = repo_icons
      .into_iter()
      .unique_by(|icon| icon.url.clone())
      .collect::<Vec<_>>();

    let repo_icons: Vec1<RepoIcon> = repo_icons
      .try_into()
      .map_err(|_| "no icons found for repo")?;

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
    owner: &str,
    repo: &str,
  ) -> Result<Self, Box<dyn Error>> {
    let endpoint = endpoint
      .into_url()?
      .join(&format!("{}/{}/all", owner, repo))?;

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

    self.closest_match()
  }

  pub fn closest_match(&self) -> &RepoIcon {
    self.0.first()
  }
}

impl IntoIterator for RepoIcons {
  type Item = RepoIcon;
  type IntoIter = std::vec::IntoIter<Self::Item>;

  fn into_iter(self) -> Self::IntoIter {
    self.0.into_iter()
  }
}
