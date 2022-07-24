use cached::proc_macro::cached;

#[derive(Deserialize)]
struct Repo {
  name: String,
}

#[cached]
pub async fn get_user_repos(user: String) -> Option<Vec<String>> {
  let res = gh_api_get!("users/{}/repos?per_page=100", user)
    .send()
    .await
    .ok()?
    .json::<Vec<Repo>>()
    .await
    .ok()?;

  Some(res.into_iter().map(|r| r.name.to_lowercase()).collect())
}
