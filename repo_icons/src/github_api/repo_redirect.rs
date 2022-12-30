use cached::proc_macro::cached;
use serde::Deserialize;
use std::time::Instant;

#[derive(Deserialize)]
struct User {
  login: String,
  r#type: String,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum UserResponse {
  Message { message: String },
  User(User),
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RepoResponse {
  Message { message: String },
  Repo { name: String, owner: User },
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

  let other_repo_res = get_redirected_repo(other_repo.0.clone(), other_repo.1.clone())
    .await
    .map(|(owner, repo, _)| (owner, repo));

  let other_repo = other_repo_res.unwrap_or(other_repo);

  if other_repo == repo {
    return true;
  }

  let repo_res = get_redirected_repo(repo.0.clone(), repo.1.clone())
    .await
    .map(|(owner, repo, _)| (owner, repo));
  let repo = repo_res.unwrap_or(repo);

  if repo == other_repo {
    return true;
  }

  false
}

pub async fn get_redirected_user(owner: String, repo: String) -> Result<(String, bool), String> {
  match get_redirected_repo(owner.clone(), repo).await {
    Ok((owner, _, is_org)) => Ok((owner, is_org)),
    Err(_) => {
      let start = Instant::now();

      let url = format!("users/{}", owner);
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
        UserResponse::User(user) => Ok((owner, user.r#type == "Organization")),
      }
    }
  }
}

#[cached]
pub async fn get_redirected_repo(
  owner: String,
  repo: String,
) -> Result<(String, String, bool), String> {
  let url = format!("repos/{}/{}", owner, repo);
  let start = Instant::now();
  let repo = async {
    gh_api_get!("{}", url)
      .send()
      .await?
      .json::<RepoResponse>()
      .await
  }
  .await
  .map_err(|e| format!("{}: {:?}", url, e));

  info!("{}: {:?}", url, start.elapsed());

  match repo? {
    RepoResponse::Message { message } => Err(message),
    RepoResponse::Repo { name, owner } => Ok((
      owner.login.to_lowercase(),
      name.to_lowercase(),
      owner.r#type == "Organization",
    )),
  }
}
