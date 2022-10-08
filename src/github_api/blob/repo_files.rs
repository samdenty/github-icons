use cached::proc_macro::cached;
use std::error::Error;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileType {
  Blob,
  Tree,
  Commit,
}

#[derive(Debug, Clone, Deserialize)]
pub struct File {
  pub path: String,
  pub r#type: FileType,
  pub sha: String,
}

#[derive(Deserialize)]
struct Commit {
  sha: String,
}

#[derive(Deserialize)]
struct Trees {
  tree: Vec<File>,
}

#[cached]
async fn get_repo_files_cached(
  owner: String,
  repo: String,
  tree_sha: String,
) -> Result<Vec<File>, String> {
  let res = gh_api_get!(
    "repos/{}/{}/git/trees/{}?recursive=1",
    owner,
    repo,
    tree_sha
  )
  .send()
  .await
  .map_err(|e| format!("{:?}", e).to_string())?
  .json::<Trees>()
  .await
  .map_err(|e| format!("{:?}", e).to_string())?;

  Ok(res.tree)
}

pub async fn get_repo_files(
  owner: &str,
  repo: &str,
) -> Result<(String, Vec<File>), Box<dyn Error>> {
  let res = gh_api_get!("repos/{}/{}/commits", owner, repo)
    .send()
    .await?
    .json::<Vec<Commit>>()
    .await?;

  let commit_sha = res
    .into_iter()
    .next()
    .ok_or(format!("no commits found for repo {}/{}!", owner, repo))?
    .sha;

  let files = get_repo_files_cached(
    owner.to_lowercase(),
    repo.to_lowercase(),
    commit_sha.clone(),
  )
  .await?;

  Ok((commit_sha, files))
}
