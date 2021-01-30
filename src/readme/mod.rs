mod badges;
mod primary_heading;
pub mod readme_image;
mod repo_redirect;

pub use readme_image::*;

use self::{primary_heading::PrimaryHeading, repo_redirect::is_same_repo};
use crate::github_api_get;
use scraper::Html;
use serde::{de, Deserialize};
use std::error::Error;
use url::Url;

pub struct Readme {
  pub owner: String,
  pub repo: String,
  pub homepage: Option<Url>,
  link_base: Url,
  document: Html,
}

impl Readme {
  pub async fn load(owner: &str, repo: &str) -> Result<Self, Box<dyn Error>> {
    let (repo, readme_body) = try_join!(
      async {
        #[derive(Deserialize)]
        struct RepoOwner {
          login: String,
        }

        #[derive(Deserialize)]
        struct Repo {
          owner: RepoOwner,
          name: String,
          default_branch: String,
          #[serde(deserialize_with = "deserialize_url")]
          homepage: Option<Url>,
        }

        github_api_get!("repos/{}/{}", owner, repo)
          .send()
          .await?
          .json::<Repo>()
          .await
      },
      async {
        github_api_get!("repos/{}/{}/readme", owner, repo)
          .header("Accept", "application/vnd.github.html")
          .send()
          .await?
          .error_for_status()?
          .text()
          .await
      }
    )?;

    Ok(Readme::new(
      &repo.owner.login,
      &repo.name,
      &readme_body,
      &repo.default_branch,
      repo.homepage,
    ))
  }

  pub fn new(
    owner: &str,
    repo: &str,
    body: &str,
    default_branch: &str,
    homepage: Option<Url>,
  ) -> Self {
    let document = Html::parse_document(&body);

    let link_base = Url::parse(&format!(
      "https://github.com/{}/{}/raw/{}/",
      owner, repo, default_branch
    ))
    .unwrap();

    Self {
      owner: owner.to_lowercase(),
      repo: repo.to_lowercase(),
      homepage,
      document,
      link_base,
    }
  }

  pub async fn images(&self) -> Vec<ReadmeImage> {
    let primary_heading = &mut PrimaryHeading::new(&self.document);

    let mut images = Vec::new();
    for element_ref in self.document.select(selector!("img[src]")) {
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

    images
  }

  /// Check if a given url is a project link.
  pub async fn is_link_to_project(&self, url: &Url) -> Option<ProjectLink> {
    let domain = url.domain()?.to_lowercase();

    // check for github pages
    let re = regex!(r"^([^.])+\.github\.(com|io)$");
    if let Some(res) = re.captures(&domain) {
      let user = &res[1];

      // USERNAME.github.io
      if let Some(repo_res) = re.captures(&domain) {
        if &repo_res[1] == user {
          return Some(ProjectLink::Website);
        }
      }

      // USERNAME.github.io/REPO
      if let Some(res) = regex!("^/([^/]+)").captures(url.path()) {
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

    if self.is_link_to_repo(url).await {
      return Some(ProjectLink::Repo);
    };

    None
  }

  /// Check if a given url is a repo link.
  pub async fn is_link_to_repo(&self, url: &Url) -> bool {
    let domain = if let Some(domain) = url.domain() {
      domain.to_lowercase()
    } else {
      return false;
    };

    if domain == "github.com" || domain == "raw.githubusercontent.com" || domain == "raw.github.com"
    {
      if let Some(res) = regex!("^/([^/]+)/([^/]+)").captures(url.path()) {
        return self.is_same_repo_as(&res[1], &res[2]).await;
      }
    }

    false
  }

  async fn is_same_repo_as(&self, user: &str, repo: &str) -> bool {
    let user = user.to_lowercase();
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
