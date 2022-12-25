mod primary_heading;
pub mod readme_image;
mod repo_redirect;

pub use readme_image::*;
use vec1::Vec1;

use self::{primary_heading::PrimaryHeading, repo_redirect::is_same_repo};
use crate::blacklist::is_blacklisted_homepage;
use scraper::Html;
use serde::{de, Deserialize};
use std::{convert::TryInto, error::Error};
use url::Url;

#[derive(Clone)]
pub struct Readme {
  pub owner: String,
  pub repo: String,
  pub homepage: Option<Url>,
  pub private: bool,
  link_base: Url,
}

impl Readme {
  pub async fn load(owner: &str, repo: &str) -> Result<Self, String> {
    #[derive(Deserialize)]
    struct RepoOwner {
      login: String,
    }

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Response {
      Repo {
        owner: RepoOwner,
        name: String,
        default_branch: String,
        private: bool,
        #[serde(deserialize_with = "deserialize_url")]
        homepage: Option<Url>,
      },
      Message {
        message: String,
      },
    }

    let url = format!("repos/{}/{}", owner, repo);

    let response = gh_api_get!("{}", url)
      .send()
      .await
      .map_err(|e| format!("{}: {:?}", url, e))?
      .json::<Response>()
      .await
      .map_err(|e| format!("{}: {:?}", url, e))?;

    match response {
      Response::Repo {
        owner,
        name,
        private,
        default_branch,
        homepage,
      } => Ok(Readme::new(
        &owner.login,
        &name,
        private,
        &default_branch,
        homepage,
      )),
      Response::Message { message } => Err(message),
    }
  }

  fn new(
    owner: &str,
    repo: &str,
    private: bool,
    default_branch: &str,
    homepage: Option<Url>,
  ) -> Self {
    let link_base = Url::parse(&format!(
      "https://github.com/{}/{}/raw/{}/",
      owner, repo, default_branch
    ))
    .unwrap();

    Self {
      owner: owner.to_lowercase(),
      repo: repo.to_lowercase(),
      private,
      homepage: homepage.and_then(|homepage| {
        if is_blacklisted_homepage(&homepage) {
          None
        } else {
          Some(homepage)
        }
      }),
      link_base,
    }
  }

  pub async fn load_body(&self) -> Option<Vec1<ReadmeImage>> {
    let body = gh_api_get!("repos/{}/{}/readme", self.owner, self.repo)
      .header("Accept", "application/vnd.github.html")
      .send()
      .await
      .ok()?
      .error_for_status()
      .ok()?
      .text()
      .await
      .ok()?;

    let document = Html::parse_document(&body);

    let primary_heading = &mut PrimaryHeading::new(&document);

    let mut images = Vec::new();
    for element_ref in document.select(selector!("img[src]")) {
      if let Some(image) = ReadmeImage::get(self, &element_ref, primary_heading).await {
        images.push(image);
      }
    }

    let mut iter = images.iter_mut().enumerate().peekable();
    while let Some((idx, image)) = iter.next() {
      if image.in_primary_heading
        && (idx == 0
          || iter
            .peek()
            .map(|(_, image)| !image.in_primary_heading)
            .unwrap_or(true))
      {
        image.edge_of_primary_heading = true;
      };
    }

    images.sort();

    warn!(
      "{:#?}",
      images
        .iter()
        .map(|img| (img.src.clone(), img.weight()))
        .collect::<Vec<_>>()
    );

    images.try_into().ok()
  }

  /// Check if a given url is a project link.
  pub async fn is_link_to_project(&self, url: &Url) -> Option<ProjectLink> {
    let domain = url.domain()?.to_lowercase();

    // check for github pages
    let re = regex!(r"^([^.])+\.github\.(com|io)$");
    if let Some(res) = re.captures(&domain).unwrap() {
      let user = &res[1];

      // USERNAME.github.io
      if let Some(repo_res) = re.captures(&domain).unwrap() {
        if &repo_res[1] == user {
          return Some(ProjectLink::Website);
        }
      }

      // USERNAME.github.io/REPO
      if let Some(res) = regex!("^/([^/]+)").captures(url.path()).unwrap() {
        let repo = &res[1];
        if self.is_same_repo_as(user, repo).await {
          return Some(ProjectLink::Website);
        }
      }
    }

    if self
      .homepage
      .as_ref()
      .and_then(|u| u.domain().map(|d| d.to_lowercase()))
      .map(|d| domain == d)
      .unwrap_or(false)
    {
      return Some(ProjectLink::Website);
    }

    if self.get_branch_and_path(url).await.is_some() {
      return Some(ProjectLink::Repo);
    };

    None
  }

  /// Check if a given url points to a file located inside the repo.
  pub async fn get_branch_and_path(&self, url: &Url) -> Option<(String, String)> {
    let domain = if let Some(domain) = url.domain() {
      domain.to_lowercase()
    } else {
      return None;
    };

    match &domain[..] {
      "raw.githubusercontent.com" | "raw.github.com" | "github.com" => {
        let re = if domain == "github.com" {
          regex!("^/([^/]+)/([^/]+)/[^/]+/([^/]+)/(.+)")
        } else {
          regex!("^/([^/]+)/([^/]+)/([^/]+)/(.+)")
        };

        if let Some(res) = re.captures(url.path()).unwrap() {
          let user = &res[1];
          let repo = &res[2];

          if self.is_same_repo_as(user, repo).await {
            let branch = &res[3];
            let path = &res[4];
            return Some((branch.into(), path.into()));
          };
        }
      }
      _ => {}
    }

    None
  }

  pub fn qualify_url(&self, path: &str) -> Result<Url, Box<dyn Error>> {
    let mut path = path.to_string();
    if path.starts_with("/") {
      path = format!(".{}", path);
    }

    Ok(self.link_base.join(&path)?)
  }

  async fn is_same_repo_as(&self, owner: &str, repo: &str) -> bool {
    let user = owner.to_lowercase();
    let repo = repo.to_lowercase();
    is_same_repo((&self.owner, &self.repo), (&user, &repo)).await
  }
}

fn deserialize_url<'de, D: de::Deserializer<'de>>(d: D) -> Result<Option<Url>, D::Error> {
  Deserialize::deserialize(d).map(|url: Option<&str>| {
    url.and_then(|url| {
      if url.is_empty() {
        return None;
      }
      Url::parse(url.as_ref())
        .or_else(|_| Url::parse(&format!("http://{}", url)))
        .ok()
    })
  })
}
