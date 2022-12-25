use crate::{
  blacklist::{is_badge_url, is_blacklisted_homepage},
  get_token, github_api, RepoIcon, RepoIconKind,
};
use async_recursion::async_recursion;
use futures::{
  future::{join_all, select_all, try_join_all},
  Future, FutureExt,
};
use itertools::Itertools;
use reqwest::IntoUrl;
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
  Avatar(RepoIcon),
  RepoFile(Option<Vec1<RepoIcon>>),
  ReadmeImage(Option<RepoIcon>),
  Homepage(Option<Vec1<RepoIcon>>),
  PrefixedRepo(Option<Vec1<RepoIcon>>),
}

impl RepoIcons {
  /// Fetch all the icons. Ordered from highest to lowest resolution
  ///
  /// ```
  /// # async fn run() {
  /// let icons = RepoIcons::load("facebook", "react", false).await?;
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
        let icon = RepoIcon::load_user_avatar(owner, repo).await?;
        Ok(LoadedKind::Avatar(icon))
      }
      .boxed_local(),
      // Try and find prefixed repos, and load icons for them on GitHub
      async {
        let repos = github_api::get_user_repos(owner, repo).await?;

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
          .collect::<Vec<_>>()
          .try_into()
          .ok(),
        ))
      }
      .boxed_local(),
      async {
        let blob_icons = match github_api::get_repo_icon_files(owner, repo)
          .await
          .ok()
          .flatten()
        {
          Some((is_icon_field, blobs)) => Some(
            try_join_all(
              blobs
                .into_iter()
                .map(|blob| RepoIcon::load_blob(blob, is_icon_field)),
            )
            .await?
            .try_into()
            .unwrap(),
          ),
          None => None,
        };

        Ok(LoadedKind::RepoFile(blob_icons))
      }
      .boxed_local(),
      async {
        let mut icons = Icons::new_with_blacklist(|url| is_blacklisted_homepage(url));

        let homepage = readme.clone().await.ok().and_then(|readme| readme.homepage);

        if let Some(homepage) = &homepage {
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
            .filter(|icon| !is_badge_url(&icon.url))
            .map(|icon| {
              RepoIcon::new(
                icon.url,
                (homepage.clone().unwrap(), icon.kind).into(),
                icon.info,
              )
            })
            .collect::<Vec<_>>()
            .try_into()
            .ok(),
        ))
      }
      .boxed_local(),
      // Try and extract images from the readme website, or directly in it
      async {
        Ok(LoadedKind::ReadmeImage(match readme.clone().await {
          Ok(readme) => {
            let image = readme
              .load_body()
              .await
              .and_then(|images| images.into_iter().find(|image| image.in_primary_heading));

            match image {
              Some(image) => Some(
                RepoIcon::load_with_headers(image.src, image.headers, RepoIconKind::ReadmeImage)
                  .await?,
              ),
              None => None,
            }
          }
          Err(_) => None,
        }))
      }
      .boxed_local(),
    ];

    let mut previous_loads = Vec::new();
    let mut found_best_match = false;

    let mut error = Ok(());

    while !futures.is_empty() {
      let (loaded, index, _) = select_all(&mut futures).await;
      futures.remove(index);

      let loaded = match loaded {
        Err(err) => {
          error = Err(err);
          continue;
        }
        Ok(loaded) => loaded,
      };

      match &loaded {
        LoadedKind::RepoFile(file_icons) => {
          if let Some(mut file_icons) = file_icons.clone() {
            for file_icon in &mut file_icons {
              file_icon.set_repo_private(readme.clone().await?.private);

              if matches!(file_icon.kind, RepoIconKind::IconField(_)) {
                found_best_match = true;
              }
            }

            repo_icons.extend(file_icons);

            if previous_loads
              .iter()
              .any(|loaded| matches!(loaded, LoadedKind::Avatar(_)))
              && previous_loads
                .iter()
                .any(|loaded| matches!(loaded, LoadedKind::Homepage(_)))
            {
              // if we have both the avatar & homepage but haven't
              // found the best match yet, then the repo file is the
              // best match
              found_best_match = true;
            }
          }
        }

        LoadedKind::Avatar(user_avatar) => {
          // found_best_match for RepoFile
          if let Some(blob_kinds) = previous_loads.iter().find_map(|loaded| {
            if let LoadedKind::RepoFile(blob_icons) = loaded {
              Some(blob_icons.as_ref().map(|blob_icons| {
                blob_icons
                  .iter()
                  .map(|blob| blob.kind.clone())
                  .collect::<Vec<_>>()
              }))
            } else {
              None
            }
          }) {
            if let Some(blob_kinds) = blob_kinds {
              for blob_kind in blob_kinds {
                if matches!(blob_kind, RepoIconKind::RepoFile(_)) {
                  found_best_match = true;
                }
              }
            } else {
              found_best_match = true;
            }
          }

          repo_icons.push(user_avatar.clone());
        }

        LoadedKind::ReadmeImage(readme_image) => {
          if let Some(readme_image) = readme_image {
            if previous_loads
              .iter()
              .any(|loaded| matches!(loaded, LoadedKind::Avatar(_)))
              && previous_loads
                .iter()
                .any(|loaded| matches!(loaded, LoadedKind::RepoFile(_)))
              && previous_loads
                .iter()
                .any(|loaded| matches!(loaded, LoadedKind::Homepage(_)))
            {
              // if we've already got the Avatar, RepoFile & Homepage,
              // then the ReadmeImage is the best match
              found_best_match = true;
            }

            repo_icons.push(readme_image.clone());
          }
        }

        LoadedKind::PrefixedRepo(icons) => {
          if let Some(icons) = icons {
            repo_icons.extend(icons.clone());
          }
        }

        LoadedKind::Homepage(site_icons) => {
          if let Some(site_icons) = site_icons {
            repo_icons.extend(site_icons.clone());

            // if it contains AppIcon or SiteFavicon
            if site_icons.iter().any(|icon| {
              matches!(
                icon.kind,
                RepoIconKind::AppIcon { .. } | RepoIconKind::SiteFavicon { .. }
              )
            }) && previous_loads
              .iter()
              .any(|loaded| matches!(loaded, LoadedKind::Avatar(_)))
              && previous_loads
                .iter()
                .any(|loaded| matches!(loaded, LoadedKind::RepoFile(_)))
            {
              found_best_match = true;
            }
          }
        }
      }

      previous_loads.push(loaded);

      repo_icons.sort_by(|a, b| a.info.cmp(&b.info));
      repo_icons.sort_by(|a, b| a.kind.cmp(&b.kind));

      if best_matches_only && found_best_match {
        break;
      }
    }

    error?;

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
  /// let icons = RepoIcons::fetch("https://github-icons.com", "facebook", "react").await?;
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
    let mut endpoint = endpoint
      .into_url()?
      .join(&format!("{}/{}/all", owner, repo))?;

    if let Some(token) = get_token() {
      endpoint.set_query(Some(&format!("token={}", token)));
    }

    let repo_icons = reqwest::get(endpoint)
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

    self.best_match()
  }

  pub fn best_match(&self) -> &RepoIcon {
    self.0.first()
  }

  pub fn into_best_match(self) -> RepoIcon {
    self.0.into_iter().next().unwrap()
  }
}

impl IntoIterator for RepoIcons {
  type Item = RepoIcon;
  type IntoIter = std::vec::IntoIter<Self::Item>;

  fn into_iter(self) -> Self::IntoIter {
    self.0.into_iter()
  }
}
