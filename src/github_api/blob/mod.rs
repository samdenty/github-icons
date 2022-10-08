mod repo_files;

use crate::RepoBlob;
use fancy_regex::{escape, Regex};
use repo_files::{get_repo_files, File, FileType};
use std::error::Error;

fn get_weight(owner: &str, repo: &str, file: &File) -> u8 {
  let owner = owner.to_lowercase();
  let repo = repo.to_lowercase();

  let fullpath = file.path.to_lowercase();
  let (path, filename) = fullpath.rsplit_once('/').unwrap_or(("", &fullpath));

  let mut weight = 0;
  let mut matches_icon = false;

  if filename.contains(&owner) {
    weight += 1;
    matches_icon = true;
  }

  if filename.contains(&repo) {
    weight += 2;
    matches_icon = true;
  }

  let exactly_repo_name =
    Regex::new(&format!("^{}(?:[-_]logo.*)?\\.[^.]+$", escape(&repo))).unwrap();
  if exactly_repo_name.is_match(&filename).unwrap() {
    weight += 1;
  }

  let fixtures = regex!("(e2e|fixtures|test(s)?)/");
  if !fixtures.is_match(&file.path).unwrap() {
    if filename.contains("favicon") {
      weight += 2;
      matches_icon = true;
    }

    let logo = regex!("logo(?!ut|n|s)");
    if logo.is_match(&fullpath).unwrap() {
      weight += 1;
      matches_icon = true;

      let exactly_logo = regex!("^logo\\.[^.]+$");
      if exactly_logo.is_match(&filename).unwrap() {
        weight += 2;
      }
    }

    if matches_icon {
      let public = regex!("(public|static|resources|assets|www)/");
      if public.is_match(&fullpath).unwrap() {
        weight += 1;
      }

      if path.contains("server") || fullpath.contains("website") {
        weight += 1;
      }
    }
  }

  weight
}

pub async fn get_blob(owner: &str, repo: &str) -> Result<Option<RepoBlob>, Box<dyn Error>> {
  let (commit_sha, files) = get_repo_files(owner, repo).await?;

  let mut results = files
    .into_iter()
    .filter(|file| {
      matches!(file.r#type, FileType::Blob)
        && (file.path.ends_with(".png")
          || file.path.ends_with(".jpg")
          || file.path.ends_with(".jpeg")
          || file.path.ends_with(".ico")
          || file.path.ends_with(".svg"))
    })
    .map(|file| {
      let weight = get_weight(owner, repo, &file);
      (file, weight)
    })
    .filter(|(_, weight)| *weight > 0)
    .collect::<Vec<_>>();

  results.sort_by(|(_, a_weight), (_, b_weight)| b_weight.cmp(&a_weight));

  let result = results.get(0).cloned().map(|(file, weight)| {
    let final_results = results
      .iter()
      .cloned()
      .filter(|(_, other_weight)| weight == *other_weight)
      .collect::<Vec<_>>();

    final_results
      .iter()
      .cloned()
      .find(|(file, _)| file.path.ends_with(".svg"))
      .or_else(|| {
        final_results
          .into_iter()
          .find(|(file, _)| file.path.ends_with(".png"))
      })
      .unwrap_or((file, weight))
  });

  Ok(result.map(|(file, _)| RepoBlob {
    owner: owner.to_string(),
    repo: repo.to_string(),
    commit_sha,

    sha: file.sha,
    path: file.path,
  }))
}
