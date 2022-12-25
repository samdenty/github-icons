use super::get_redirected_user;
use cached::proc_macro::cached;
use std::error::Error;

#[derive(Deserialize)]
struct Repo {
  name: String,
  fork: bool,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Response {
  Repos(Vec<Repo>),
  Message { message: String },
}

#[cached]
async fn get_user_repos_cached(user: String) -> Result<Vec<String>, String> {
  let url = format!("users/{}/repos?per_page=100", user);

  let res = gh_api_get!("{}", url)
    .send()
    .await
    .map_err(|e| format!("{}: {:?}", url, e))?
    .json::<Response>()
    .await
    .map_err(|e| format!("{}: {:?}", url, e))?;

  match res {
    Response::Repos(repos) => Ok(
      repos
        .into_iter()
        .filter(|repo| !repo.fork)
        .map(|repo| repo.name.to_lowercase())
        .collect(),
    ),
    Response::Message { message } => Err(message),
  }
}

pub async fn get_user_repos(owner: &str, repo: &str) -> Result<Vec<String>, Box<dyn Error>> {
  let (user, _) = get_redirected_user(owner.to_lowercase(), repo.to_lowercase()).await?;

  get_user_repos_cached(user).await.map_err(|e| e.into())
}
