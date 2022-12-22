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
enum CommitResponse {
  Commits(Vec<Commit>),
  Message { message: String },
}

#[derive(Deserialize)]
enum TreesResponse {
  Trees { tree: Vec<File> },
  Message { message: String },
}

#[cached]
async fn get_repo_files_cached(
  owner: String,
  repo: String,
  tree_sha: String,
) -> Result<Vec<File>, String> {
  let url = format!(
    "repos/{}/{}/git/trees/{}?recursive=1",
    owner, repo, tree_sha
  );

  let res = gh_api_get!("{}", url)
    .send()
    .await
    .map_err(|e| format!("{}: {:?}", url, e).to_string())?
    .json::<TreesResponse>()
    .await
    .map_err(|e| format!("{}: {:?}", url, e).to_string())?;

  match res {
    TreesResponse::Trees { tree } => Ok(tree),
    TreesResponse::Message { message } => Err(message),
  }
}

pub async fn get_repo_files(
  owner: &str,
  repo: &str,
) -> Result<(String, Vec<File>), Box<dyn Error>> {
  let url = format!("repos/{}/{}/commits?per_page=1", owner, repo);

  let res = gh_api_get!("{}", url)
    .send()
    .await
    .map_err(|e| format!("{}: {:?}", url, e))?
    .json::<CommitResponse>()
    .await
    .map_err(|e| format!("{}: {:?}", url, e))?;

  let commits = match res {
    CommitResponse::Commits(commits) => commits,
    CommitResponse::Message { message } => return Err(message.into()),
  };

  let commit_sha = commits
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
