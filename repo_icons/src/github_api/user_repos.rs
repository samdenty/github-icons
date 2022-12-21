use cached::proc_macro::cached;
use std::error::Error;

#[derive(Deserialize)]
struct Repo {
  name: String,
}

#[derive(Deserialize)]
enum Response {
  Repos(Vec<Repo>),
  Message { message: String },
}

#[cached]
async fn get_user_repos_cached(user: String) -> Result<Vec<String>, String> {
  let res = gh_api_get!("users/{}/repos?per_page=100", user)
    .send()
    .await
    .map_err(|e| format!("{:?}", e).to_string())?
    .json::<Response>()
    .await
    .map_err(|e| format!("{:?}", e).to_string())?;

  match res {
    Response::Repos(repos) => Ok(repos.into_iter().map(|r| r.name.to_lowercase()).collect()),
    Response::Message { message } => Err(message),
  }
}

pub async fn get_user_repos(user: &str) -> Result<Vec<String>, Box<dyn Error>> {
  get_user_repos_cached(user.to_lowercase())
    .await
    .map_err(|e| e.into())
}
