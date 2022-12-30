use cached::proc_macro::cached;
use std::{error::Error, time::Instant};

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
#[serde(untagged)]
enum TreesResponse {
  Trees { sha: String, tree: Vec<File> },
  Message { message: String },
}

#[cached]
async fn get_repo_files_cached(owner: String, repo: String) -> Result<(String, Vec<File>), String> {
  let url = format!("repos/{}/{}/git/trees/HEAD?recursive=1", owner, repo);

  let start = Instant::now();
  let res = async {
    gh_api_get!("{}", url)
      .send()
      .await?
      .json::<TreesResponse>()
      .await
  }
  .await
  .map_err(|e| format!("{}: {:?}", url, e).to_string());

  info!("{}: {:?}", url, start.elapsed());

  match res? {
    TreesResponse::Trees { sha, tree } => Ok((sha, tree)),
    TreesResponse::Message { message } => Err(message),
  }
}

pub async fn get_repo_files(
  owner: &str,
  repo: &str,
) -> Result<(String, Vec<File>), Box<dyn Error>> {
  let result = get_repo_files_cached(owner.to_lowercase(), repo.to_lowercase()).await?;

  Ok(result)
}
