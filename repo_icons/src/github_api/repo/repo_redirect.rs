use super::repo::{Repo, User};
use cached::proc_macro::cached;
use cached::SizedCache;
use instant::Instant;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(untagged)]
enum UserResponse {
  Message { message: String },
  User(User),
}

/// check if two repos are the same, following
/// redirects (in case the user/repo was renamed)
/// user/repo pairs should be transformed to lowercase!
pub async fn is_same_repo(repo: (&str, &str), other_repo: (&str, &str)) -> bool {
  let repo = (repo.0.to_lowercase(), repo.1.to_lowercase());
  let other_repo = (other_repo.0.to_lowercase(), repo.1.to_lowercase());

  if repo == other_repo {
    return true;
  }

  // if neither user nor repo are the same
  // then ignore it (even though it could be true)
  if repo.0 != other_repo.0 && repo.1 != other_repo.1 {
    return false;
  }

  let other_repo_res = get_redirected_repo(&other_repo.0, &other_repo.1)
    .await
    .map(|(owner, repo, _)| (owner, repo));

  let other_repo = other_repo_res.unwrap_or(other_repo);

  if other_repo == repo {
    return true;
  }

  let repo_res = get_redirected_repo(&repo.0, &repo.1)
    .await
    .map(|(owner, repo, _)| (owner, repo));
  let repo = repo_res.unwrap_or(repo);

  if repo == other_repo {
    return true;
  }

  false
}

#[cached(
  sync_writes = true,
  type = "SizedCache<String, Result<(String, bool), String>>",
  create = "{ SizedCache::with_size(100) }",
  convert = r#"{ format!("{}/{}", owner.to_lowercase(), repo.to_lowercase()) }"#
)]
pub async fn get_redirected_user(owner: &str, repo: &str) -> Result<(String, bool), String> {
  match get_redirected_repo(owner, repo).await {
    Ok((owner, _, is_org)) => Ok((owner, is_org)),
    Err(_) => {
      let url = format!("users/{}", owner);
      let start = Instant::now();

      let user = async {
        gh_api_get!("{}", url)
          .send()
          .await?
          .json::<UserResponse>()
          .await
      }
      .await
      .map_err(|e| format!("{}: {:?}", url, e));

      info!("{}: {:?}", url, start.elapsed());

      match user? {
        UserResponse::Message { message } => Err(message),
        UserResponse::User(user) => Ok((owner.to_lowercase(), user.r#type == "Organization")),
      }
    }
  }
}

async fn get_redirected_repo(owner: &str, repo: &str) -> Result<(String, String, bool), String> {
  let repo = Repo::load(owner, repo).await?;

  Ok((
    repo.owner.login.to_lowercase(),
    repo.name.to_lowercase(),
    repo.owner.r#type == "Organization",
  ))
}
