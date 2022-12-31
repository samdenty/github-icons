use super::is_same_repo;
use crate::blacklist::is_blacklisted_homepage;
use cached::proc_macro::cached;
use cached::SizedCache;
use serde::{de, Deserialize};
use std::time::Instant;
use url::Url;

#[derive(Clone, Deserialize)]
pub struct User {
  pub login: String,
  pub r#type: String,
}

#[derive(Clone, Deserialize)]
pub struct Repo {
  pub owner: User,
  pub name: String,
  pub default_branch: String,
  pub private: bool,
  #[serde(deserialize_with = "deserialize_homepage")]
  pub homepage: Option<Url>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RepoResponse {
  Repo(Repo),
  Message { message: String },
}

impl Repo {
  pub async fn load(owner: &str, repo: &str) -> Result<Self, String> {
    get_repo_cached(owner, repo).await
  }
}

#[cached(
  sync_writes = true,
  type = "SizedCache<String, Result<Repo, String>>",
  create = "{ SizedCache::with_size(100) }",
  convert = r#"{ format!("{}/{}", owner.to_lowercase(), repo.to_lowercase()) }"#
)]
async fn get_repo_cached(owner: &str, repo: &str) -> Result<Repo, String> {
  let url = format!("repos/{}/{}", owner, repo);
  let start = Instant::now();

  let response = async {
    gh_api_get!("{}", url)
      .send()
      .await?
      .json::<RepoResponse>()
      .await
  }
  .await
  .map_err(|e| format!("{}: {:?}", url, e));

  info!("{}: {:?}", url, start.elapsed());

  match response? {
    RepoResponse::Repo(repo) => Ok(repo),
    RepoResponse::Message { message } => Err(message),
  }
}

pub fn qualify_repo_raw_url(owner: &str, repo: &str, path: &str) -> Result<Url, url::ParseError> {
  let link_base = Url::parse(&format!("https://github.com/{}/{}/raw/HEAD/", owner, repo)).unwrap();

  let mut path = path.to_string();
  if path.starts_with("/") {
    path = format!(".{}", path);
  }

  link_base.join(&path)
}

/// Check if a given url points to a file located inside the repo.
pub async fn get_branch_and_path(owner: &str, repo: &str, url: &Url) -> Option<(String, String)> {
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
        let other_owner = &res[1];
        let other_repo = &res[2];

        if is_same_repo((owner, repo), (other_owner, other_repo)).await {
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

fn deserialize_homepage<'de, D: de::Deserializer<'de>>(d: D) -> Result<Option<Url>, D::Error> {
  Deserialize::deserialize(d).map(|url: Option<&str>| {
    url.and_then(|url| {
      if url.is_empty() {
        return None;
      }

      if let Ok(homepage) =
        Url::parse(url.as_ref()).or_else(|_| Url::parse(&format!("http://{}", url)))
      {
        if is_blacklisted_homepage(&homepage) {
          None
        } else {
          Some(homepage)
        }
      } else {
        None
      }
    })
  })
}
