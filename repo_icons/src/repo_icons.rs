use crate::{
  blacklist::{is_badge_url, is_blacklisted_homepage},
  get_token,
  github_api::{self, Readme, Repo},
  RepoIcon, RepoIconKind,
};
use async_recursion::async_recursion;
use futures::{
  future::{select_all, try_join_all},
  Future, FutureExt,
};
use futures_timer::Delay;
use instant::Duration;
use itertools::Itertools;
use reqwest::IntoUrl;
use serde::{ser::SerializeStruct, Deserialize, Deserializer, Serialize, Serializer};
use site_icons::SiteIcons;
use std::{
  cmp::{max, min},
  collections::HashMap,
  convert::TryInto,
  error::Error,
  ops::{Deref, DerefMut},
  pin::Pin,
};
use vec1::Vec1;

const NO_ICONS_FOUND: &str = "No icons found for repo";

#[derive(Debug)]
pub struct RepoIconsResult {
  pub errors: Option<Vec1<String>>,
  pub icons: Result<RepoIcons, String>,
}

impl Serialize for RepoIconsResult {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut state = serializer.serialize_struct("RepoIconsResult", 2)?;
    state.serialize_field("errors", &self.errors)?;
    state.serialize_field("icons", &self.icons.as_ref().ok())?;
    state.end()
  }
}

impl<'de> Deserialize<'de> for RepoIconsResult {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    #[derive(Deserialize)]
    struct Fields {
      errors: Option<Vec1<String>>,
      icons: Option<RepoIcons>,
    }

    let Fields { errors, icons } = Fields::deserialize(deserializer)?;

    Ok(RepoIconsResult {
      errors,
      icons: icons.ok_or_else(|| NO_ICONS_FOUND.to_string()),
    })
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepoIcons(Vec1<RepoIcon>);

#[derive(Debug, Clone)]
enum LoadedKind {
  Avatar(RepoIcon),
  RepoFile(Option<Vec1<RepoIcon>>),
  ReadmeImage(Option<RepoIcon>),
  Homepage(Option<Vec1<RepoIcon>>),
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
  pub async fn load(owner: &str, repo: &str, best_matches_only: bool) -> RepoIconsResult {
    let mut repo_icons = Vec::new();

    let mut futures: Vec<Pin<Box<dyn Future<Output = Result<LoadedKind, Box<dyn Error>>>>>> = vec![
      async {
        let icon = RepoIcon::load_user_avatar(owner, repo).await?;
        Ok(LoadedKind::Avatar(icon))
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
                .map(|blob| RepoIcon::load_repo_file(blob, is_icon_field)),
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
        let mut icons =
          SiteIcons::new_with_blacklist(|url| is_blacklisted_homepage(url) || is_badge_url(url));

        let Repo { homepage, .. } = Repo::load(owner, repo).await?;

        let entries = match &homepage {
          Some(homepage) => {
            select_all(vec![
              icons
                .load_website(homepage.clone(), best_matches_only)
                .boxed_local(),
              Delay::new(Duration::from_secs(2))
                .map(|_| Ok(Vec::new()))
                .boxed_local(),
            ])
            .await
            .0?
          }
          None => Vec::new(),
        };

        Ok(LoadedKind::Homepage(
          entries
            .into_iter()
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
        let readme = Readme::load(owner, repo).await;
        let image =
          readme.and_then(|images| images.into_iter().find(|image| image.in_primary_heading));

        Ok(LoadedKind::ReadmeImage(match image {
          Some(image) => Some(
            RepoIcon::load_with_headers(image.src, image.headers, RepoIconKind::ReadmeImage)
              .await?,
          ),
          None => None,
        }))
      }
      .boxed_local(),
    ];

    let mut previous_loads = Vec::new();
    let mut found_best_match = false;

    let mut errors = Vec::new();

    while !futures.is_empty() {
      let (loaded, index, _) = select_all(&mut futures).await;
      futures.remove(index);

      let loaded = match loaded {
        Err(err) => {
          errors.push(err.to_string());
          continue;
        }
        Ok(loaded) => loaded,
      };

      match &loaded {
        LoadedKind::RepoFile(file_icons) => {
          if let Some(mut file_icons) = file_icons.clone() {
            for file_icon in &mut file_icons {
              if let Ok(Repo { private, .. }) = Repo::load(&owner, &repo).await {
                file_icon.set_repo_private(private);
              }

              if matches!(file_icon.kind, RepoIconKind::IconField { .. }) {
                found_best_match = true;
              }
            }

            // this is to ensure it isn't a Framework
            let has_repo_file = file_icons
              .iter()
              .any(|file_icon| matches!(file_icon.kind, RepoIconKind::RepoFile { .. }));

            repo_icons.extend(file_icons);

            if has_repo_file
              && previous_loads
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
                if matches!(blob_kind, RepoIconKind::RepoFile { .. }) {
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

    let repo_icons = repo_icons
      .into_iter()
      .unique_by(|icon| icon.url.clone())
      .collect::<Vec<_>>();

    let icons: Result<Vec1<RepoIcon>, _> = repo_icons
      .try_into()
      .map_err(|_| NO_ICONS_FOUND.to_string());

    RepoIconsResult {
      icons: icons.map(|icons| RepoIcons(icons)),
      errors: errors.try_into().ok(),
    }
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

impl Deref for RepoIcons {
  type Target = Vec1<RepoIcon>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for RepoIcons {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}
