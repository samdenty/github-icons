use cached::proc_macro::cached;
use serde::Deserialize;

#[derive(Deserialize)]
struct RepoOwner {
  login: String,
}

#[derive(Deserialize)]
struct Repo {
  name: String,
  owner: RepoOwner,
}

/// check if two repos are the same, following
/// redirects (in case the user/repo was renamed)
/// user/repo pairs should be transformed to lowercase!
pub async fn is_same_repo(repo: (&str, &str), other_repo: (&str, &str)) -> bool {
  if repo == other_repo {
    return true;
  }

  // if neither user nor repo are the same
  // then ignore it (even though it could be true)
  if repo.0 != other_repo.0 && repo.1 != other_repo.1 {
    return false;
  }

  let other_repo_res = get_repo_redirect(other_repo.0.into(), other_repo.1.into())
    .await
    .map(|(user, repo)| (user.to_lowercase(), repo.to_lowercase()));
  let other_repo = other_repo_res
    .as_ref()
    .map(|(user, repo)| (&user[..], &repo[..]))
    .unwrap_or(other_repo);

  if other_repo == repo {
    return true;
  }

  let repo_res = get_repo_redirect(repo.0.into(), repo.1.into())
    .await
    .map(|(user, repo)| (user.to_lowercase(), repo.to_lowercase()));
  let repo = repo_res
    .as_ref()
    .map(|(user, repo)| (&user[..], &repo[..]))
    .unwrap_or(repo);

  if repo == other_repo {
    return true;
  }

  false
}

#[cached]
async fn get_repo_redirect(owner: String, repo: String) -> Option<(String, String)> {
  #[cfg(target_arch = "wasm32")]
  let req = gh_api_get!("repos/{}/{}", owner, repo);

  #[cfg(not(target_arch = "wasm32"))]
  let req = {
    use gh_api::gh_client;
    use reqwest::{header::LOCATION, redirect::Policy};

    let client = gh_client(None).redirect(Policy::none()).build().ok()?;
    let res = gh_api_get!(client, "repos/{}/{}", owner, repo)
      .send()
      .await
      .ok()?;

    if res.status() != 301 {
      return None;
    }

    let location = res.headers().get(LOCATION)?.to_str().ok()?;
    client.get(location)
  };

  let repo = req.send().await.ok()?.json::<Repo>().await.ok()?;

  Some((repo.owner.login, repo.name))
}
